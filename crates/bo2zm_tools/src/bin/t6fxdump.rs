<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/bo2zm_tools/src/bin/t6fxdump.rs
//! bo2mp: one Black Ops II effect part by part, read-only.
//! `t6fxdump <name part> [bo2 root]` reads the multiplayer zones (or
//! `T6FXDUMP_ZONES=a,b`) and prints, for each effect whose name contains the
//! part (and the effects it spawns): every element's type, flags, counts,
//! delays and life, its colour and size graph as stored, what it draws; then
//! each material it draws with: technique set, blend, colour map.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::process::ExitCode;

use asset_t6::{DrawState, FxVisualRef, ZoneCapture};
use fastfile_t6::layout::{FxElemDef as e, FxElemVisStateSample as vs, FxElemVisualState as v};

const DEFAULT_BO2: &str = r"D:\SteamLibrary\steamapps\common\Call of Duty Black Ops II";
const ZONES: [&str; 5] = ["code_pre_gfx_mp", "code_post_gfx_mp", "common_mp", "patch_mp", "mp_nuketown_2020"];
=======
//! bo2zm: one Black Ops II effect's elements, read-only. `t6fxdump <effect
//! name> [bo2 root]` prints per element its type, flags, life, spawn, fade
//! ranges and each visual state sample's colour and size (base / amplitude).

use std::path::PathBuf;
use std::process::ExitCode;

use fastfile_t6::layout::FxElemDef as e;

const DEFAULT_BO2: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Call of Duty Black Ops II";
const ZONES: [&str; 4] = ["common_zm", "patch_zm", "zm_nuked", "zm_nuked_patch"];
>>>>>>> /tmp/claude-0/zmhead/crates/bo2zm_tools/src/bin/t6fxdump.rs

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(want) = args.next() else {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/bo2zm_tools/src/bin/t6fxdump.rs
        eprintln!("usage: t6fxdump <effect name part> [bo2 root]");
        return ExitCode::FAILURE;
    };
    let root = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_BO2.to_owned()));
    let zones: Vec<String> = std::env::var("T6FXDUMP_ZONES")
        .map(|z| z.split(',').map(str::to_owned).collect())
        .unwrap_or_else(|_| ZONES.iter().map(|z| (*z).to_owned()).collect());
    let dir = root.join("zone").join("all");
    let mut caps: Vec<ZoneCapture> = Vec::new();
    for z in &zones {
        match asset_t6::capture_zone(&dir.join(format!("{z}.ff"))) {
            Ok(c) => caps.push(c),
            Err(err) => eprintln!("{z}: {err}"),
        }
    }
    let mut fx = HashMap::new();
    for c in &caps {
        for f in c.fx.iter().filter(|f| !f.name.starts_with(',') && !f.name.is_empty()) {
            fx.insert(f.name.clone(), f);
        }
    }
    let mut todo: Vec<String> = fx.keys().filter(|n| n.contains(&want)).cloned().collect();
    todo.sort();
    todo.reverse();
    let mut seen = BTreeSet::new();
    let mut materials = BTreeSet::new();
    while let Some(name) = todo.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(f) = fx.get(&name) else {
            println!("\n== {name}: NOT FOUND");
            continue;
        };
        println!(
            "\n== {name}: flags {:#06x} looping {} one-shot {} emitted {} looping life {} ms, non-looping life {} ms",
            f.flags, f.looping_count, f.one_shot_count, f.emission_count, f.msec_looping_life, f.msec_non_looping_life
        );
        for (i, el) in f.elems.iter().enumerate() {
            let pair = |off: usize| format!("{}+{}", el.i32_at(off), el.i32_at(off + 4));
            let b = |off: usize| el.raw.get(off).copied().unwrap_or(0);
            let u16_at = |off: usize| u16::from_le_bytes([b(off), b(off + 1)]);
            println!(
                "  #{i} type {} flags {:#010x} spawn {} delay {} ms life {} ms alphaFade {} ms lighting {} sort {} gravity {}+{} fade-in {}..{} fade-out {}..{}",
                el.elem_type(),
                el.i32_at(e::flags),
                pair(e::spawn),
                pair(e::spawnDelayMsec),
                pair(e::lifeSpanMsec),
                u16_at(e::alphaFadeTimeMsec),
                b(e::lightingFrac),
                b(e::sortOrder),
                el.f32_at(e::gravity),
                el.f32_at(e::gravity + 4),
                el.f32_at(e::fadeInRange),
                el.f32_at(e::fadeInRange + 4),
                el.f32_at(e::fadeOutRange),
                el.f32_at(e::fadeOutRange + 4),
            );
            for (k, s) in el.vis_samples.chunks_exact(vs::SIZE).enumerate() {
                let state = |at: usize| {
                    let f32_at = |off: usize| f32::from_le_bytes([s[at + off], s[at + off + 1], s[at + off + 2], s[at + off + 3]]);
                    format!(
                        "rgba {:?} size {:.1}x{:.1} scale {:.2} rot {:.2}/{:.2}",
                        &s[at + v::color..at + v::color + 4],
                        f32_at(v::size),
                        f32_at(v::size + 4),
                        f32_at(v::scale),
                        f32_at(v::rotationDelta),
                        f32_at(v::rotationTotal),
                    )
                };
                println!("     vis {k}: base {} | amp {}", state(vs::base), state(vs::amplitude));
            }
            for vis in &el.visuals {
                match vis {
                    FxVisualRef::Material(m) => {
                        println!("     material {m}");
                        materials.insert(m.clone());
                    }
                    FxVisualRef::Mark([a, b]) => {
                        println!("     mark {a} | {b}");
                        materials.insert(a.clone());
                    }
                    FxVisualRef::Model(m) => println!("     model {m}"),
                    FxVisualRef::Effect(c) => {
                        println!("     runs {c}");
                        todo.push(c.clone());
                    }
                    FxVisualRef::Sound(s) => println!("     sound {s}"),
                    FxVisualRef::Light(_) => println!("     light"),
                    FxVisualRef::None => {}
                }
            }
            for (what, child) in [("on impact", &el.effect_on_impact), ("on death", &el.effect_on_death), ("emitted", &el.effect_emitted)] {
                if !child.is_empty() {
                    println!("     {what} {child}");
                    todo.push(child.clone());
                }
            }
        }
    }
    println!("\n== materials");
    for m in &materials {
        let found = caps.iter().find_map(|c| c.materials.iter().find(|x| x.name == *m && !x.textures.is_empty()).map(|x| (c, x)));
        let Some((c, mat)) = found else {
            println!("{m}: NOT FOUND");
            continue;
        };
        let ts = mat.technique_set.map_or("-".to_owned(), |t| c.technique_sets[t.index].name.clone());
        let state = [6usize, 5, 4, 3, 2, 1, 0].iter().find_map(|&t| {
            let entry = *mat.state_bits_entry.get(t)?;
            mat.state_bits.get(usize::from(entry)).map(|b| (t, DrawState::decode(*b)))
        });
        let blend = state.map_or("no state".to_owned(), |(t, s)| {
            format!("slot{t} {}/{} op{} at{:?}", s.src_blend, s.dst_blend, s.blend_op, s.alpha_test)
        });
        let tex = mat.textures.iter().find(|t| t.name_hash == 0xa0ab_1041).or(mat.textures.first());
        let img = tex.and_then(|t| t.image).map_or("-", |k| c.images[k.index].name.as_str());
        println!("{m} | {ts} | {blend} | {img}");
    }
=======
        eprintln!("usage: t6fxdump <effect name> [bo2 root]");
        return ExitCode::FAILURE;
    };
    let root = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_BO2.to_owned()));
    let dir = root.join("zone").join("all");
    // T6FXDUMP_ZONES=a,b: read these zones instead (another map's effects).
    let zones: Vec<String> = std::env::var("T6FXDUMP_ZONES").map_or_else(
        |_| ZONES.iter().map(|z| (*z).to_owned()).collect(),
        |v| v.split(',').map(str::to_owned).collect(),
    );
    for z in &zones {
        let Ok(c) = asset_t6::capture_zone(&dir.join(format!("{z}.ff"))) else { continue };
        for fx in c.fx.iter().filter(|f| f.name == want) {
            println!("== {z}: {} ({} elems)", fx.name, fx.elems.len());
            for (n, el) in fx.elems.iter().enumerate() {
                let b = |o: usize| el.raw.get(o).copied().unwrap_or(0);
                println!(
                    "#{n} type {} flags {:#010x} life {}+{} spawn {} {} delay {}+{} fadeIn {:?} fadeOut {:?} visInt {} visuals {:?}",
                    b(e::elemType),
                    el.i32_at(e::flags),
                    el.i32_at(e::lifeSpanMsec),
                    el.i32_at(e::lifeSpanMsec + 4),
                    el.i32_at(e::spawn),
                    el.i32_at(e::spawn + 4),
                    el.i32_at(e::spawnDelayMsec),
                    el.i32_at(e::spawnDelayMsec + 4),
                    [el.f32_at(e::fadeInRange), el.f32_at(e::fadeInRange + 4)],
                    [el.f32_at(e::fadeOutRange), el.f32_at(e::fadeOutRange + 4)],
                    b(e::visStateIntervalCount),
                    el.visuals.iter().take(2).collect::<Vec<_>>(),
                );
                let r = |o: usize| [el.f32_at(o), el.f32_at(o + 4)];
                println!(
                    "   origin {:?} {:?} {:?} offR {:?} offH {:?} angles {:?} {:?} {:?} grav {:?} velInt {}",
                    r(e::spawnOrigin), r(e::spawnOrigin + 8), r(e::spawnOrigin + 16),
                    r(e::spawnOffsetRadius), r(e::spawnOffsetHeight),
                    r(e::spawnAngles), r(e::spawnAngles + 8), r(e::spawnAngles + 16),
                    r(e::gravity), b(e::velIntervalCount),
                );
                let vs: Vec<&[u8]> = el.vel_samples.chunks(96).collect();
                for (k, s) in vs.iter().enumerate().filter(|(k, _)| *k == 0 || *k + 1 == vs.len()) {
                    let v = |o: usize| [0, 4, 8].map(|d| f32::from_le_bytes([s[o + d], s[o + d + 1], s[o + d + 2], s[o + d + 3]]));
                    println!(
                        "   v{k}: local vel {:?} delta {:?} world vel {:?} delta {:?} | amp local delta {:?} world delta {:?}",
                        v(0), v(12), v(24), v(36), v(60), v(84)
                    );
                }
                for (k, s) in el.vis_samples.chunks(48).enumerate() {
                    let f = |o: usize| f32::from_le_bytes([s[o], s[o + 1], s[o + 2], s[o + 3]]);
                    println!(
                        "   s{k}: col {:?}/{:?} size {:.0},{:.0}/{:.0},{:.0} scale {:.2}/{:.2}",
                        &s[0..4], &s[24..28], f(12), f(16), f(36), f(40), f(20), f(44)
                    );
                }
            }
        }
    }
>>>>>>> /tmp/claude-0/zmhead/crates/bo2zm_tools/src/bin/t6fxdump.rs
    ExitCode::SUCCESS
}
