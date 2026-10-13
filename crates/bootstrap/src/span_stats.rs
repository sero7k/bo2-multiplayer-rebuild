//! A profiling build's clock on every bevy system and schedule
//! (`--features span-stats`): where a frame's time goes, by name.
//!
//! Bevy's `trace` feature opens a span around each system run, each
//! schedule and each render graph node. This layer adds up how long each one
//! was open and, every few seconds, appends the biggest to a text file
//! (`IW4L_SPAN_STATS`, else `iw4l-artifacts/span-stats.txt`), with every
//! single run that took 20 ms or more. Small enough to keep for a whole match,
//! unlike a Chrome trace. Never on in a normal build: the clock costs the
//! frame it measures.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use bevy::log::BoxedLayer;
use bevy::log::tracing::field::{Field, Visit};
use bevy::log::tracing::{Subscriber, span};
use bevy::log::tracing_subscriber::Layer;
use bevy::log::tracing_subscriber::layer::Context;
use bevy::log::tracing_subscriber::registry::LookupSpan;
use bevy::prelude::App;

/// How often the totals go to the file.
const WINDOW: Duration = Duration::from_secs(10);
/// A single run this long is listed on its own.
const SPIKE: Duration = Duration::from_millis(20);
/// The most names listed per window (`IW4L_SPAN_TOP` lists more, to break a
/// schedule down into its small systems).
const TOP: usize = 60;

fn top() -> usize {
    std::env::var("IW4L_SPAN_TOP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(TOP)
}

/// Bevy's spans are at info level; the log keeps its usual quiet filter for
/// everything else.
pub(crate) const FILTER: &str = "warn,iw4l=info,bevy_ecs=info,bevy_app=info,bevy_render=info,\
     bevy_core_pipeline=info,bevy_winit=info,bevy_ui_render=info,bevy_sprite_render=info";

struct Label(&'static str);
struct Entered(Vec<Instant>);

#[derive(Default)]
struct Stat {
    total: Duration,
    runs: u64,
    max: Duration,
}

#[derive(Default)]
struct Shard {
    stats: HashMap<&'static str, Stat>,
    spikes: Vec<(f32, &'static str, f32)>,
}

static SHARDS: Mutex<Vec<Arc<Mutex<Shard>>>> = Mutex::new(Vec::new());
static NAMES: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
static START: OnceLock<Instant> = OnceLock::new();

thread_local! {
    static SHARD: Arc<Mutex<Shard>> = {
        let shard = Arc::new(Mutex::new(Shard::default()));
        lock(&SHARDS).push(shard.clone());
        shard
    };
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// One leaked copy per distinct name (systems and schedules are a fixed set).
fn intern(name: String) -> &'static str {
    let mut names = lock(&NAMES);
    let names = names.get_or_insert_with(HashSet::new);
    if let Some(&known) = names.get(name.as_str()) {
        return known;
    }
    let leaked: &'static str = Box::leak(name.into_boxed_str());
    names.insert(leaked);
    leaked
}

#[derive(Default)]
struct NameField(Option<String>);

impl Visit for NameField {
    fn record_str(&mut self, field: &Field, value: &str) {
        if matches!(field.name(), "name" | "plugin") {
            self.0 = Some(value.to_owned());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if matches!(field.name(), "name" | "plugin") {
            self.0 = Some(format!("{value:?}").trim_matches('"').to_owned());
        }
    }
}

struct SpanStats;

impl<S> Layer<S> for SpanStats
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let mut field = NameField::default();
        attrs.record(&mut field);
        let meta = attrs.metadata();
        let label = match field.0 {
            Some(name) => format!("{}: {name}", meta.name()),
            None => format!("{} ({})", meta.name(), meta.target()),
        };
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(Label(intern(label)));
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut ext = span.extensions_mut();
        match ext.get_mut::<Entered>() {
            Some(entered) => entered.0.push(Instant::now()),
            None => ext.insert(Entered(vec![Instant::now()])),
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        let now = Instant::now();
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut ext = span.extensions_mut();
        let Some(began) = ext.get_mut::<Entered>().and_then(|e| e.0.pop()) else {
            return;
        };
        let Some(label) = ext.get_mut::<Label>().map(|l| l.0) else {
            return;
        };
        drop(ext);
        let took = now - began;
        let at = START.get().map_or(0.0, |s| (began - *s).as_secs_f32());
        let _ = SHARD.try_with(|shard| {
            let mut shard = lock(shard);
            let stat = shard.stats.entry(label).or_default();
            stat.total += took;
            stat.runs += 1;
            stat.max = stat.max.max(took);
            if took >= SPIKE && shard.spikes.len() < 4096 {
                shard.spikes.push((at, label, took.as_secs_f32() * 1000.0));
            }
        });
    }
}

/// The layer, and the thread that writes its totals.
pub(crate) fn layer(_app: &mut App) -> Option<BoxedLayer> {
    let path = std::env::var_os("IW4L_SPAN_STATS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("iw4l-artifacts/span-stats.txt"));
    START.get_or_init(Instant::now);
    std::thread::Builder::new()
        .name("span stats".into())
        .spawn(move || write_windows(path))
        .ok()?;
    Some(Box::new(SpanStats))
}

fn write_windows(path: PathBuf) {
    let start = *START.get_or_init(Instant::now);
    let mut from = 0.0f32;
    let top = top();
    loop {
        std::thread::sleep(WINDOW);
        let to = start.elapsed().as_secs_f32();
        let mut stats: HashMap<&'static str, Stat> = HashMap::new();
        let mut spikes = Vec::new();
        for shard in lock(&SHARDS).iter() {
            let mut shard = lock(shard);
            for (label, stat) in shard.stats.drain() {
                let into = stats.entry(label).or_default();
                into.total += stat.total;
                into.runs += stat.runs;
                into.max = into.max.max(stat.max);
            }
            spikes.append(&mut shard.spikes);
        }
        let mut rows: Vec<_> = stats.into_iter().collect();
        rows.sort_by(|a, b| b.1.total.cmp(&a.1.total));
        spikes.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut out = format!("== {from:.0}-{to:.0} s\n");
        for (label, stat) in rows.iter().take(top) {
            let total = stat.total.as_secs_f64() * 1000.0;
            let _ = writeln!(
                out,
                "{total:>10.1} ms {:>7} runs {:>8.3} avg {:>8.2} max  {label}",
                stat.runs,
                total / stat.runs.max(1) as f64,
                stat.max.as_secs_f64() * 1000.0,
            );
        }
        for (at, label, ms) in &spikes {
            let _ = writeln!(out, "  spike {at:>8.2} s {ms:>8.1} ms  {label}");
        }
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(out.as_bytes());
        }
        from = to;
    }
}
