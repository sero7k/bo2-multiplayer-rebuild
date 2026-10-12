//! The music director's decisions, as plain functions of time: which context
//! is playing, which track comes next (a shuffle that never repeats the track
//! that just ended), and every voice's gain along its fade. No Bevy here; the
//! systems in `director.rs` carry out the [`Action`]s and feed back what the
//! decoders and sinks report, and the tests below drive it with a fake clock.

use std::collections::HashSet;
use std::f64::consts::FRAC_PI_2;

/// An index into the director's track table.
pub type TrackId = usize;

/// What the game is doing, as far as music goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MusicContext {
    /// Loading, a match without ambient music, or no front end: nothing plays.
    #[default]
    Silent,
    /// The front end's main screens.
    Menu,
    /// The custom-game lobby and its map/mode pickers.
    Lobby,
    /// In a match: low ambient music, when the settings allow it.
    Match,
}

impl MusicContext {
    pub const PLAYING: [MusicContext; 3] =
        [MusicContext::Menu, MusicContext::Lobby, MusicContext::Match];

    /// Its playlist's slot; `None` for silence.
    pub fn slot(self) -> Option<usize> {
        match self {
            MusicContext::Silent => None,
            MusicContext::Menu => Some(0),
            MusicContext::Lobby => Some(1),
            MusicContext::Match => Some(2),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MusicContext::Silent => "silent",
            MusicContext::Menu => "menu",
            MusicContext::Lobby => "lobby",
            MusicContext::Match => "match",
        }
    }
}

/// The longest any front-end music may still be fading once a match is on.
pub const MATCH_CUTOFF: f64 = 0.25;

/// Fade lengths, in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FadeTimings {
    /// A track starting from silence.
    pub fade_in: f64,
    /// A context change (menu -> lobby, -> loading, -> match) fading out
    /// what was playing.
    pub fade_out: f64,
    /// One track handing over to the next in the same playlist.
    pub crossfade: f64,
    /// How long before a track's handover the next one starts preparing.
    pub prefetch: f64,
    /// A track that has not prepared after this long is dropped.
    pub prepare_timeout: f64,
}

impl Default for FadeTimings {
    fn default() -> Self {
        Self {
            fade_in: 2.0,
            fade_out: 1.5,
            crossfade: 4.0,
            prefetch: 30.0,
            prepare_timeout: 90.0,
        }
    }
}

/// An equal-power fade from `from` to `to` over `[start, start + dur]`: a
/// rise follows a quarter sine and a fall a quarter cosine, so two voices
/// crossing over keep a constant power (sin² + cos² = 1) and neither end of
/// either fade has a step in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Envelope {
    from: f32,
    to: f32,
    start: f64,
    dur: f64,
}

impl Envelope {
    pub fn hold(value: f32) -> Self {
        Self {
            from: value,
            to: value,
            start: 0.0,
            dur: 0.0,
        }
    }

    pub fn ramp(from: f32, to: f32, start: f64, dur: f64) -> Self {
        Self {
            from,
            to,
            start,
            dur: dur.max(0.0),
        }
    }

    pub fn at(&self, now: f64) -> f32 {
        if self.dur <= 0.0 || now >= self.start + self.dur {
            return self.to;
        }
        if now <= self.start {
            return self.from;
        }
        let x = (now - self.start) / self.dur;
        if self.to >= self.from {
            self.from + (self.to - self.from) * (x * FRAC_PI_2).sin() as f32
        } else {
            self.to + (self.from - self.to) * (x * FRAC_PI_2).cos() as f32
        }
    }

    /// Heads for `to` from wherever the fade is now. `full` is the time a
    /// whole 0 <-> 1 fade takes; a part-way fade takes its share of it, so a
    /// track half faded in fades out in half the time and never jumps.
    pub fn retarget(&mut self, now: f64, to: f32, full: f64) {
        let here = self.at(now);
        let share = f64::from((to - here).abs()).clamp(0.0, 1.0);
        *self = Self::ramp(here, to, now, full * share);
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    /// When it reaches its target.
    pub fn settles_at(&self) -> f64 {
        self.start + self.dur
    }
}

/// A small xorshift; seeded by the caller (the clock in the game, a constant
/// in tests).
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n <= 1 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }
}

/// One context's tracks. Shuffled, it deals them from a bag: every track once
/// before any twice, and a new bag never opens with the track that just
/// played. In order, it walks the list. Either way a track that cannot play
/// is passed over.
#[derive(Clone, Debug, Default)]
pub struct Playlist {
    tracks: Vec<TrackId>,
    shuffle: bool,
    bag: Vec<TrackId>,
    cursor: usize,
    last: Option<TrackId>,
}

impl Playlist {
    pub fn new(tracks: Vec<TrackId>, shuffle: bool) -> Self {
        let mut seen = HashSet::new();
        let tracks = tracks.into_iter().filter(|t| seen.insert(*t)).collect();
        Self {
            tracks,
            shuffle,
            ..Self::default()
        }
    }

    pub fn tracks(&self) -> &[TrackId] {
        &self.tracks
    }

    pub fn contains(&self, track: TrackId) -> bool {
        self.tracks.contains(&track)
    }

    pub fn last(&self) -> Option<TrackId> {
        self.last
    }

    /// Adds a track at the end (and to the current bag); `false` if it was
    /// already listed.
    pub fn push(&mut self, track: TrackId) -> bool {
        if self.contains(track) {
            return false;
        }
        self.tracks.push(track);
        if !self.bag.is_empty() {
            self.bag.insert(0, track);
        }
        true
    }

    /// Note a track as played (so the next pick avoids it).
    pub fn played(&mut self, track: TrackId) {
        self.last = Some(track);
    }

    pub fn next(&mut self, rng: &mut Rng, usable: impl Fn(TrackId) -> bool) -> Option<TrackId> {
        let open: Vec<TrackId> = self.tracks.iter().copied().filter(|t| usable(*t)).collect();
        if open.is_empty() {
            return None;
        }
        let pick = if self.shuffle {
            self.bag.retain(|t| usable(*t));
            if self.bag.is_empty() {
                let mut bag = open.clone();
                for i in (1..bag.len()).rev() {
                    let j = rng.below(i + 1);
                    bag.swap(i, j);
                }
                // Dealt from the end: keep the last-played track off the top.
                if bag.len() > 1 && bag.last().copied() == self.last {
                    let j = rng.below(bag.len() - 1);
                    let top = bag.len() - 1;
                    bag.swap(j, top);
                }
                self.bag = bag;
            }
            self.bag.pop()?
        } else {
            let n = self.tracks.len();
            let mut found = None;
            for step in 0..n {
                let t = self.tracks[(self.cursor + step) % n];
                if usable(t) && (Some(t) != self.last || open.len() == 1) {
                    self.cursor = (self.cursor + step + 1) % n;
                    found = Some(t);
                    break;
                }
            }
            found?
        };
        Some(pick)
    }
}

/// What the director asks of the game this tick.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Start decoding/opening a track; answer with [`Core::ready`] or
    /// [`Core::failed`].
    Prepare { track: TrackId, context: MusicContext },
    /// Spawn the prepared track as `voice` (silent: its gain comes from
    /// [`Core::gain`] each frame).
    Start { voice: u64, track: TrackId, context: MusicContext, fade: f64 },
    /// Despawn a voice; its fade is over.
    Stop { voice: u64, track: TrackId },
    /// A track that never prepared in time; it is dropped like a failure.
    TimedOut { track: TrackId },
    /// A context has nothing it can play.
    Empty { context: MusicContext },
}

#[derive(Clone, Debug)]
pub struct Voice {
    pub id: u64,
    pub track: TrackId,
    pub context: MusicContext,
    env: Envelope,
    started: f64,
    length: Option<f64>,
    retiring: bool,
}

impl Voice {
    pub fn retiring(&self) -> bool {
        self.retiring
    }
}

#[derive(Clone, Debug)]
struct Want {
    track: TrackId,
    context: MusicContext,
    since: f64,
    /// Prepared, with its length when known.
    ready: Option<Option<f64>>,
}

/// The director's state.
#[derive(Clone, Debug)]
pub struct Core {
    pub timings: FadeTimings,
    context: MusicContext,
    playlists: [Playlist; 3],
    voices: Vec<Voice>,
    want: Option<Want>,
    dead: HashSet<TrackId>,
    rng: Rng,
    next_voice: u64,
    reported_empty: Option<MusicContext>,
}

impl Core {
    pub fn new(timings: FadeTimings, playlists: [Playlist; 3], seed: u64) -> Self {
        Self {
            timings,
            context: MusicContext::Silent,
            playlists,
            voices: Vec::new(),
            want: None,
            dead: HashSet::new(),
            rng: Rng::new(seed),
            next_voice: 1,
            reported_empty: None,
        }
    }

    pub fn context(&self) -> MusicContext {
        self.context
    }

    pub fn voices(&self) -> &[Voice] {
        &self.voices
    }

    pub fn playlist(&self, context: MusicContext) -> Option<&Playlist> {
        context.slot().map(|s| &self.playlists[s])
    }

    pub fn is_dead(&self, track: TrackId) -> bool {
        self.dead.contains(&track)
    }

    /// The track being prepared, if any.
    pub fn wanted(&self) -> Option<TrackId> {
        self.want.as_ref().map(|w| w.track)
    }

    fn active(&self) -> Option<&Voice> {
        self.voices.iter().find(|v| !v.retiring)
    }

    fn retire(voice: &mut Voice, now: f64, full: f64) {
        voice.retiring = true;
        voice.env.retarget(now, 0.0, full);
    }

    /// The game moved to `context`. What is playing fades out, unless its
    /// track belongs to the new context's playlist too, in which case it
    /// simply carries on.
    pub fn set_context(&mut self, context: MusicContext, now: f64) {
        if context == self.context {
            return;
        }
        self.context = context;
        self.reported_empty = None;
        let keeps = |v: &Voice| {
            context
                .slot()
                .is_some_and(|_| self.playlist(context).is_some_and(|p| p.contains(v.track)))
        };
        let mut kept = false;
        let fade_out = self.timings.fade_out;
        let fade_in = self.timings.fade_in;
        let keep_flags: Vec<bool> = self.voices.iter().map(|v| !v.retiring && keeps(v)).collect();
        for (voice, keep) in self.voices.iter_mut().zip(keep_flags) {
            if keep && !kept {
                kept = true;
                voice.context = context;
                voice.env.retarget(now, 1.0, fade_in);
            } else if !voice.retiring {
                Self::retire(voice, now, fade_out);
            }
        }
        if self.want.as_ref().is_some_and(|w| w.context != context) {
            self.want = None;
        }
        // Front-end music never carries into a match: whatever of it is
        // still fading goes within a quarter second.
        if context == MusicContext::Match {
            for voice in &mut self.voices {
                if voice.context != MusicContext::Match
                    && voice.env.settles_at() > now + MATCH_CUTOFF
                {
                    voice.retiring = true;
                    voice.env.retarget(now, 0.0, MATCH_CUTOFF);
                }
            }
        }
    }

    /// Adds `track` to `context`'s playlist (a menu bed asked for by a
    /// script menu); `false` if it was already there.
    pub fn add_to_playlist(&mut self, context: MusicContext, track: TrackId) -> bool {
        context
            .slot()
            .is_some_and(|slot| self.playlists[slot].push(track))
    }

    /// Tick at `now` (seconds, any monotonic origin).
    pub fn tick(&mut self, now: f64) -> Vec<Action> {
        let mut out = Vec::new();
        // Faded-out voices go.
        self.voices.retain(|v| {
            let done = v.retiring && now >= v.env.settles_at() && v.env.at(now) <= 0.0;
            if done {
                out.push(Action::Stop {
                    voice: v.id,
                    track: v.track,
                });
            }
            !done
        });
        // The playing track nearing its end hands over.
        let crossfade = self.timings.crossfade;
        let prefetch = self.timings.prefetch;
        let mut prepare_next = false;
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.retiring)
            && let Some(length) = voice.length
        {
            let ends = voice.started + length;
            let handover = ends - crossfade.min(length * 0.5);
            if now >= handover {
                voice.retiring = true;
                voice.env = Envelope::ramp(voice.env.at(now), 0.0, now, (ends - now).max(0.0));
                prepare_next = true;
            } else if now >= handover - prefetch {
                prepare_next = true;
            }
        }
        if self.context == MusicContext::Silent {
            self.want = None;
            return out;
        }
        if let Some(want) = &self.want
            && want.ready.is_none()
            && now - want.since > self.timings.prepare_timeout
        {
            let track = want.track;
            self.want = None;
            self.dead.insert(track);
            out.push(Action::TimedOut { track });
        }
        let idle = self.active().is_none();
        if self.want.is_none() && (idle || prepare_next) {
            let slot = self.context.slot().unwrap_or(0);
            let dead = &self.dead;
            let current = self.active().map(|v| v.track);
            let playlist = &mut self.playlists[slot];
            // Don't pick the track that is still playing as its own successor
            // when there is any other.
            let picked = playlist
                .next(&mut self.rng, |t| !dead.contains(&t) && Some(t) != current)
                .or_else(|| playlist.next(&mut self.rng, |t| !dead.contains(&t)));
            match picked {
                Some(track) => {
                    self.want = Some(Want {
                        track,
                        context: self.context,
                        since: now,
                        ready: None,
                    });
                    out.push(Action::Prepare {
                        track,
                        context: self.context,
                    });
                }
                None if idle && self.reported_empty != Some(self.context) => {
                    self.reported_empty = Some(self.context);
                    out.push(Action::Empty {
                        context: self.context,
                    });
                }
                None => {}
            }
        }
        // A prepared track starts once nothing else is the active one.
        if self.active().is_none()
            && let Some(want) = &self.want
            && let Some(length) = want.ready
        {
            let fade = if self.voices.is_empty() {
                self.timings.fade_in
            } else {
                self.timings.crossfade
            };
            let id = self.next_voice;
            self.next_voice += 1;
            let track = want.track;
            let context = want.context;
            self.want = None;
            if let Some(slot) = context.slot() {
                self.playlists[slot].played(track);
            }
            self.voices.push(Voice {
                id,
                track,
                context,
                env: Envelope::ramp(0.0, 1.0, now, fade),
                started: now,
                length,
                retiring: false,
            });
            out.push(Action::Start {
                voice: id,
                track,
                context,
                fade,
            });
        }
        out
    }

    /// The track asked for is prepared. `false` when it is no longer wanted
    /// (the context moved on); the caller drops what it prepared.
    pub fn ready(&mut self, track: TrackId, length: Option<f64>) -> bool {
        match &mut self.want {
            Some(want) if want.track == track => {
                want.ready = Some(length.filter(|l| l.is_finite() && *l > 0.0));
                true
            }
            _ => false,
        }
    }

    /// The track asked for cannot play; it is out of every playlist for the
    /// rest of the session.
    pub fn failed(&mut self, track: TrackId) {
        self.dead.insert(track);
        if self.want.as_ref().is_some_and(|w| w.track == track) {
            self.want = None;
        }
    }

    /// A voice's sink ran dry (or vanished) before its fade was over.
    pub fn voice_ended(&mut self, voice: u64) {
        self.voices.retain(|v| v.id != voice);
    }

    /// A voice's fade gain at `now` (0 for a voice the director does not know).
    pub fn gain(&self, voice: u64, now: f64) -> f32 {
        self.voices
            .iter()
            .find(|v| v.id == voice)
            .map_or(0.0, |v| v.env.at(now).clamp(0.0, 1.0))
    }

    /// Everything stops at once (shutdown).
    pub fn stop_all(&mut self) -> Vec<Action> {
        self.want = None;
        self.voices
            .drain(..)
            .map(|v| Action::Stop {
                voice: v.id,
                track: v.track,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timings() -> FadeTimings {
        FadeTimings {
            fade_in: 2.0,
            fade_out: 1.0,
            crossfade: 4.0,
            prefetch: 10.0,
            prepare_timeout: 30.0,
        }
    }

    fn lists(menu: &[TrackId], lobby: &[TrackId], matc: &[TrackId]) -> [Playlist; 3] {
        [
            Playlist::new(menu.to_vec(), true),
            Playlist::new(lobby.to_vec(), true),
            Playlist::new(matc.to_vec(), true),
        ]
    }

    /// Runs the core to `until` in 50 ms steps, answering every Prepare at
    /// once with `length(track)`, and returns every action with its time.
    fn run(
        core: &mut Core,
        from: f64,
        until: f64,
        length: impl Fn(TrackId) -> Option<f64>,
    ) -> Vec<(f64, Action)> {
        let mut log = Vec::new();
        let mut t = from;
        while t <= until + 1e-9 {
            for action in core.tick(t) {
                if let Action::Prepare { track, .. } = action {
                    core.ready(track, length(track));
                }
                log.push((t, action));
            }
            t += 0.05;
        }
        log
    }

    #[test]
    fn envelope_is_equal_power_and_continuous() {
        let rise = Envelope::ramp(0.0, 1.0, 10.0, 4.0);
        let fall = Envelope::ramp(1.0, 0.0, 10.0, 4.0);
        assert_eq!(rise.at(9.0), 0.0);
        assert_eq!(rise.at(14.0), 1.0);
        assert_eq!(fall.at(14.5), 0.0);
        let mut prev = 0.0;
        for i in 0..=400 {
            let t = 10.0 + 4.0 * f64::from(i) / 400.0;
            let (a, b) = (rise.at(t), fall.at(t));
            assert!((a * a + b * b - 1.0).abs() < 1e-4, "power at {t}: {a} {b}");
            assert!((a - prev).abs() < 0.01, "step at {t}");
            prev = a;
        }
    }

    #[test]
    fn retarget_mid_fade_starts_where_it_is() {
        let mut env = Envelope::ramp(0.0, 1.0, 0.0, 2.0);
        let here = env.at(1.0);
        env.retarget(1.0, 0.0, 2.0);
        assert!((env.at(1.0) - here).abs() < 1e-6);
        // A part-way fade takes its share of the full time.
        assert!(env.settles_at() < 3.0 && env.settles_at() > 1.0);
        assert_eq!(env.at(env.settles_at()), 0.0);
    }

    #[test]
    fn shuffle_never_repeats_back_to_back_and_deals_every_track() {
        let mut list = Playlist::new(vec![0, 1, 2, 3, 4], true);
        let mut rng = Rng::new(7);
        let mut prev = None;
        let mut counts = [0usize; 5];
        for round in 0..500 {
            let t = list.next(&mut rng, |_| true).unwrap();
            assert_ne!(Some(t), prev, "repeat at {round}");
            list.played(t);
            counts[t] += 1;
            prev = Some(t);
        }
        // A bag deals each track once per five picks.
        assert!(counts.iter().all(|&c| c == 100), "{counts:?}");
    }

    #[test]
    fn shuffle_skips_unusable_tracks() {
        let mut list = Playlist::new(vec![0, 1, 2], true);
        let mut rng = Rng::new(3);
        for _ in 0..30 {
            let t = list.next(&mut rng, |t| t != 1).unwrap();
            assert_ne!(t, 1);
            list.played(t);
        }
        assert_eq!(list.next(&mut rng, |_| false), None);
    }

    #[test]
    fn single_track_playlist_repeats() {
        let mut list = Playlist::new(vec![9, 9], false);
        let mut rng = Rng::new(1);
        assert_eq!(list.tracks(), &[9]);
        assert_eq!(list.next(&mut rng, |_| true), Some(9));
        list.played(9);
        assert_eq!(list.next(&mut rng, |_| true), Some(9));
    }

    #[test]
    fn sequential_walks_in_order() {
        let mut list = Playlist::new(vec![4, 5, 6], false);
        let mut rng = Rng::new(1);
        let picks: Vec<_> = (0..6)
            .map(|_| {
                let t = list.next(&mut rng, |_| true).unwrap();
                list.played(t);
                t
            })
            .collect();
        assert_eq!(picks, vec![4, 5, 6, 4, 5, 6]);
    }

    #[test]
    fn tracks_crossfade_at_their_end() {
        let mut core = Core::new(timings(), lists(&[0, 1], &[], &[]), 11);
        core.set_context(MusicContext::Menu, 0.0);
        let log = run(&mut core, 0.0, 70.0, |_| Some(30.0));
        let starts: Vec<(f64, TrackId)> = log
            .iter()
            .filter_map(|(t, a)| match a {
                Action::Start { track, .. } => Some((*t, *track)),
                _ => None,
            })
            .collect();
        assert!(starts.len() >= 3, "{starts:?}");
        // The second starts when the first has its crossfade left.
        assert!((starts[1].0 - 26.0).abs() < 0.11, "{starts:?}");
        assert_ne!(starts[0].1, starts[1].1);
        assert_ne!(starts[1].1, starts[2].1);
        // Prepared ahead of time, not at the handover.
        let first_prepare_after_start = log
            .iter()
            .filter(|(t, a)| *t > 0.1 && matches!(a, Action::Prepare { .. }))
            .map(|(t, _)| *t)
            .next()
            .unwrap();
        assert!((first_prepare_after_start - 16.0).abs() < 0.11);
        // The first voice is stopped once its fade is over, at its end.
        let stop = log
            .iter()
            .find(|(_, a)| matches!(a, Action::Stop { voice: 1, .. }))
            .unwrap();
        assert!((stop.0 - 30.0).abs() < 0.11, "{stop:?}");
    }

    #[test]
    fn crossfade_keeps_power_and_never_steps() {
        let mut core = Core::new(timings(), lists(&[0, 1], &[], &[]), 5);
        core.set_context(MusicContext::Menu, 0.0);
        let mut t = 0.0;
        let mut last = (1u64, 0.0f32);
        while t < 40.0 {
            for action in core.tick(t) {
                if let Action::Prepare { track, .. } = action {
                    core.ready(track, Some(30.0));
                }
            }
            let a = core.gain(1, t);
            let b = core.gain(2, t);
            if (27.0..29.0).contains(&t) {
                let power = a * a + b * b;
                assert!((power - 1.0).abs() < 0.02, "power {power} at {t}");
            }
            assert!((a - last.1).abs() < 0.05, "voice 1 stepped at {t}");
            last = (1, a);
            t += 0.01;
        }
    }

    #[test]
    fn context_change_fades_out_and_starts_new_playlist() {
        let mut core = Core::new(timings(), lists(&[0], &[1], &[2]), 1);
        core.set_context(MusicContext::Menu, 0.0);
        run(&mut core, 0.0, 5.0, |_| Some(120.0));
        assert_eq!(core.gain(1, 5.0), 1.0);
        core.set_context(MusicContext::Lobby, 5.0);
        let log = run(&mut core, 5.0, 8.0, |_| Some(120.0));
        assert!(log.iter().any(|(_, a)| matches!(a, Action::Start { track: 1, .. })));
        let stop = log
            .iter()
            .find(|(_, a)| matches!(a, Action::Stop { voice: 1, .. }))
            .unwrap();
        assert!((stop.0 - 6.0).abs() < 0.11, "{stop:?}");
        // Crossfading: the lobby track started while the menu one faded.
        let start = log
            .iter()
            .find(|(_, a)| matches!(a, Action::Start { .. }))
            .unwrap();
        assert!(start.0 < 6.0);
    }

    #[test]
    fn shared_track_carries_on_across_contexts() {
        let mut core = Core::new(timings(), lists(&[0], &[0, 1], &[]), 2);
        core.set_context(MusicContext::Menu, 0.0);
        run(&mut core, 0.0, 3.0, |_| Some(120.0));
        assert_eq!(core.voices()[0].track, 0);
        core.set_context(MusicContext::Lobby, 3.0);
        let log = run(&mut core, 3.0, 6.0, |_| Some(120.0));
        assert!(log.is_empty(), "{log:?}");
        assert_eq!(core.voices().len(), 1);
        assert_eq!(core.voices()[0].context, MusicContext::Lobby);
        assert_eq!(core.gain(core.voices()[0].id, 6.0), 1.0);
    }

    #[test]
    fn silence_stops_everything_and_never_carries_into_match() {
        let mut core = Core::new(timings(), lists(&[0], &[], &[0]), 3);
        core.set_context(MusicContext::Menu, 0.0);
        run(&mut core, 0.0, 3.0, |_| Some(120.0));
        // Loading: silent.
        core.set_context(MusicContext::Silent, 3.0);
        let log = run(&mut core, 3.0, 10.0, |_| Some(120.0));
        assert!(log.iter().any(|(_, a)| matches!(a, Action::Stop { voice: 1, .. })));
        assert!(core.voices().is_empty());
        // The match's ambience is a new voice, even for the same track.
        core.set_context(MusicContext::Match, 10.0);
        let log = run(&mut core, 10.0, 11.0, |_| Some(120.0));
        assert!(log.iter().any(|(_, a)| matches!(
            a,
            Action::Start { voice: 2, context: MusicContext::Match, .. }
        )));
    }

    #[test]
    fn menu_music_is_cut_quickly_when_a_match_starts() {
        let mut t = timings();
        t.fade_out = 5.0;
        let mut core = Core::new(t, lists(&[0], &[], &[]), 12);
        core.set_context(MusicContext::Menu, 0.0);
        run(&mut core, 0.0, 3.0, |_| Some(120.0));
        // A very quick load: the menu track is still fading when play starts.
        core.set_context(MusicContext::Silent, 3.0);
        run(&mut core, 3.0, 3.5, |_| Some(120.0));
        assert!(core.gain(1, 3.5) > 0.5);
        core.set_context(MusicContext::Match, 3.5);
        assert!(core.gain(1, 3.5 + MATCH_CUTOFF) <= 0.0);
        let log = run(&mut core, 3.5, 4.0, |_| Some(120.0));
        assert!(log.iter().any(|(_, a)| matches!(a, Action::Stop { voice: 1, .. })));
        // The match playlist is empty: nothing else plays, once reported.
        assert!(core.voices().is_empty());
    }

    #[test]
    fn added_tracks_join_the_rotation() {
        let mut core = Core::new(timings(), lists(&[0], &[], &[]), 13);
        assert!(core.add_to_playlist(MusicContext::Menu, 5));
        assert!(!core.add_to_playlist(MusicContext::Menu, 5));
        assert!(!core.add_to_playlist(MusicContext::Silent, 6));
        assert!(core.playlist(MusicContext::Menu).unwrap().contains(5));
    }

    #[test]
    fn context_switch_while_preparing_drops_the_stale_track() {
        let mut core = Core::new(timings(), lists(&[0], &[1], &[]), 4);
        core.set_context(MusicContext::Menu, 0.0);
        let first = core.tick(0.0);
        assert_eq!(
            first,
            vec![Action::Prepare { track: 0, context: MusicContext::Menu }]
        );
        core.set_context(MusicContext::Lobby, 0.5);
        assert!(!core.ready(0, Some(10.0)));
        let next = core.tick(0.6);
        assert!(next.contains(&Action::Prepare { track: 1, context: MusicContext::Lobby }));
    }

    #[test]
    fn failed_tracks_are_skipped_and_empty_is_reported_once() {
        let mut core = Core::new(timings(), lists(&[0, 1], &[], &[]), 6);
        core.set_context(MusicContext::Menu, 0.0);
        let mut t = 0.0;
        let mut empties = 0;
        for _ in 0..20 {
            for action in core.tick(t) {
                match action {
                    Action::Prepare { track, .. } => core.failed(track),
                    Action::Empty { .. } => empties += 1,
                    _ => {}
                }
            }
            t += 0.1;
        }
        assert!(core.is_dead(0) && core.is_dead(1));
        assert_eq!(empties, 1);
        assert!(core.voices().is_empty());
    }

    #[test]
    fn prepare_timeout_drops_the_track() {
        let mut core = Core::new(timings(), lists(&[0, 1], &[], &[]), 8);
        core.set_context(MusicContext::Menu, 0.0);
        let first = core.tick(0.0);
        let Action::Prepare { track, .. } = first[0] else {
            panic!()
        };
        let later = core.tick(31.0);
        assert!(later.contains(&Action::TimedOut { track }));
        assert!(later.iter().any(|a| matches!(a, Action::Prepare { .. })));
    }

    #[test]
    fn unknown_length_waits_for_the_sink_to_end() {
        let mut core = Core::new(timings(), lists(&[0, 1], &[], &[]), 9);
        core.set_context(MusicContext::Menu, 0.0);
        let log = run(&mut core, 0.0, 100.0, |_| None);
        assert_eq!(
            log.iter().filter(|(_, a)| matches!(a, Action::Start { .. })).count(),
            1
        );
        core.voice_ended(1);
        let log = run(&mut core, 100.0, 101.0, |_| None);
        assert!(log.iter().any(|(_, a)| matches!(a, Action::Start { voice: 2, .. })));
        assert_eq!(core.gain(2, 100.0), 0.0);
    }
}
