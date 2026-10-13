//! bo2mp: World at War's sound aliases, read from its `common_mp` the way the
//! sound bank reads them, with some guns' fire clips decoded.
//!
//! `IW4L_GAMES=<games folder> cargo run -p asset_audio --example
//! t4_sound_probe [alias ...]`; `T4_PROBE_DUMP=<dir>` also writes each named
//! alias's first clip as it is in the zone and as decoded (16-bit PCM), to
//! compare with another decoder.

use std::collections::{BTreeMap, BTreeSet};

use asset_audio::{AssetNamespace, SoundCatalog, decode_t4_wav, parse_wav};

const FIRE: [&str; 6] = [
    "weap_springfield_fire_plr",
    "weap_kar98k_fire_plr",
    "weap_mp40_fire_plr",
    "weap_thompson_fire_plr",
    "bayonet_stab",
    "weap_m1garand_fire_plr",
];

fn main() {
    let games = asset_audio::games_root_from_env().expect("IW4L_GAMES");
    let Some(path) = asset_audio::waw_common_mp(&games) else {
        println!("World at War is not installed under {}", games.0.display());
        return;
    };
    let started = std::time::Instant::now();
    let data = t4::fastfile::load(&path).expect("read the zone");
    let zone =
        t4::zone::Zone::parse(&data, t4::zone::ParseOptions::default()).expect("parse the zone");
    let read = started.elapsed();
    let bank = asset_audio::waw_sounds_from_zone(&path, &zone);
    println!(
        "{}: read in {:.1} s, catalog in {:.2} s: {} alias lists, {} loaded clips, main {:?}",
        path.display(),
        read.as_secs_f64(),
        (started.elapsed() - read).as_secs_f64(),
        bank.sounds.len(),
        bank.loaded.len(),
        asset_audio::waw_main(&games)
    );
    let again = asset_audio::waw_weapon_sounds(&path).expect("cached");
    assert!(
        std::sync::Arc::ptr_eq(&bank, &again),
        "the second read is the cache's"
    );

    // Every sound a World at War weapon names, and whether it plays.
    let mut named = BTreeSet::new();
    for (_, w) in zone.of_type(t4::zone::AssetType::Weapon) {
        collect_sounds(&w.root, &mut named);
    }
    let (mut playable, mut silent, mut missing) = (0, Vec::new(), Vec::new());
    for alias in &named {
        match bank.sound_in(AssetNamespace::T4, alias) {
            None => missing.push(alias.as_str()),
            Some(_) if clip_count(&bank, alias) > 0 => playable += 1,
            Some(_) => silent.push(alias.as_str()),
        }
    }
    println!(
        "weapon-named aliases: {} — {playable} with a clip, {} without ({:?}), {} not in the zone ({:?})",
        named.len(),
        silent.len(),
        &silent[..silent.len().min(8)],
        missing.len(),
        &missing[..missing.len().min(8)]
    );

    // Taken into a bank the way the match's is composed (behind other
    // games' sounds): the guns' fire aliases still find their clips.
    let mut composed = SoundCatalog::default();
    composed.absorb_missing_aliases_unresolved((*bank).clone());
    composed.finalize();
    for alias in FIRE {
        let clip = composed
            .loaded_for_alias(AssetNamespace::T4, alias)
            .map(|p| p.name.as_str());
        let other = composed.sound_in(AssetNamespace::Iw4, alias).is_some();
        println!("composed bank: {alias} -> {clip:?} (in another namespace: {other})");
    }

    let wanted: Vec<String> = std::env::args().skip(1).collect();
    let wanted: Vec<&str> = if wanted.is_empty() {
        FIRE.to_vec()
    } else {
        wanted.iter().map(String::as_str).collect()
    };
    let dump = std::env::var_os("T4_PROBE_DUMP").map(std::path::PathBuf::from);
    for alias in wanted {
        let Some(sound) = bank.sound_in(AssetNamespace::T4, alias) else {
            println!("{alias}: not in the bank");
            continue;
        };
        println!("{alias}: {} variants", sound.aliases.len());
        for (v, row) in sound.aliases.iter().enumerate() {
            let pcm = bank.pcm_for_variant(AssetNamespace::T4, alias, v);
            let line = match pcm.map(|p| (p, decode_t4_wav(p.encoded_bytes()))) {
                Some((p, Ok(d))) => {
                    let peak = d
                        .samples
                        .iter()
                        .map(|s| s.unsigned_abs())
                        .max()
                        .unwrap_or(0);
                    let tag = parse_wav(p.encoded_bytes()).map_or(0, |w| w.tag);
                    if v == 0
                        && let Some(dir) = &dump
                    {
                        let _ = std::fs::create_dir_all(dir);
                        let _ = std::fs::write(
                            dir.join(format!("{alias}.zone.wav")),
                            p.encoded_bytes(),
                        );
                        let _ =
                            std::fs::write(dir.join(format!("{alias}.decoded.wav")), pcm16_wav(&d));
                    }
                    format!(
                        "{} [format {tag:#x}, {} ch, {} Hz] {} samples ({} frames, header said {}), peak {peak}",
                        p.name,
                        d.channels,
                        d.rate,
                        d.samples.len(),
                        d.frames(),
                        p.samples
                    )
                }
                Some((p, Err(e))) => format!("{}: DOES NOT DECODE: {e}", p.name),
                None => format!("no loaded clip ({:?} {:?})", row.file_kind(), row.file_name),
            };
            println!(
                "  [{v}] vol {:.2}-{:.2} pitch {:.2}-{:.2} secondary {:?}: {line}",
                row.vol_min, row.vol_max, row.pitch_min, row.pitch_max, row.secondary
            );
        }
    }

    // The zone's loaded sounds the bank left out (a format not read).
    let mut left_out: BTreeMap<String, usize> = BTreeMap::new();
    for (_, l) in zone.of_type(t4::zone::AssetType::LoadedSound) {
        let name = l.root.string("name").unwrap_or("");
        let data = l.root.node("sound").map_or(&[][..], |s| s.bytes("data"));
        if data.is_empty() || bank.loaded_index_in(AssetNamespace::T4, name).is_some() {
            continue;
        }
        let why = match parse_wav(data) {
            Ok(w) => match decode_t4_wav(data) {
                Ok(d) => format!(
                    "format {:#x} {} ch {} Hz block {}: decodes ({} samples) but was left out",
                    w.tag,
                    w.channels,
                    w.rate,
                    w.block_align,
                    d.samples.len()
                ),
                Err(e) => format!(
                    "format {:#x} {} ch {} Hz block {}: {e}",
                    w.tag, w.channels, w.rate, w.block_align
                ),
            },
            Err(e) => e,
        };
        *left_out.entry(why).or_default() += 1;
    }
    println!("loaded sounds left out: {left_out:#?}");

    // Every loaded clip, decoded.
    let started = std::time::Instant::now();
    let mut by_tag: BTreeMap<u16, (usize, usize, u64)> = BTreeMap::new();
    let mut failures = Vec::new();
    for clip in &bank.loaded {
        let tag = parse_wav(clip.encoded_bytes()).map_or(0, |w| w.tag);
        let e = by_tag.entry(tag).or_default();
        match decode_t4_wav(clip.encoded_bytes()) {
            Ok(d) => {
                e.0 += 1;
                e.2 += d.samples.len() as u64;
                if clip.samples != 0 && d.frames() as u32 != clip.samples {
                    failures.push(format!(
                        "{}: {} frames, header said {}",
                        clip.name,
                        d.frames(),
                        clip.samples
                    ));
                }
            }
            Err(err) => {
                e.1 += 1;
                failures.push(format!("{}: {err}", clip.name));
            }
        }
    }
    println!(
        "decoded every loaded clip in {:.1} s: by WAVE format (ok, failed, samples) {by_tag:?}",
        started.elapsed().as_secs_f64()
    );
    for f in failures.iter().take(10) {
        println!("  {f}");
    }
}

fn clip_count(bank: &SoundCatalog, alias: &str) -> usize {
    let Some(sound) = bank.sound_in(AssetNamespace::T4, alias) else {
        return 0;
    };
    (0..sound.aliases.len())
        .filter(|&v| {
            bank.pcm_for_variant(AssetNamespace::T4, alias, v).is_some()
                || bank
                    .streamed_for_variant(AssetNamespace::T4, alias, v)
                    .is_some()
        })
        .count()
}

fn collect_sounds(node: &t4::zone::GNode, out: &mut BTreeSet<String>) {
    for (_, value) in &node.fields {
        if let t4::zone::GVal::Nodes(children) = value {
            for child in children {
                if child.ty == "SndAliasCustom" {
                    if let Some(name) = child
                        .node("name")
                        .and_then(|n| n.string("soundName"))
                        .filter(|n| !n.is_empty())
                    {
                        out.insert(name.to_owned());
                    }
                } else {
                    collect_sounds(child, out);
                }
            }
        }
    }
}

fn pcm16_wav(d: &asset_audio::T4Pcm) -> Vec<u8> {
    let data: Vec<u8> = d.samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    let ch = u32::from(d.channels);
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(ch as u16).to_le_bytes());
    out.extend_from_slice(&d.rate.to_le_bytes());
    out.extend_from_slice(&(d.rate * ch * 2).to_le_bytes());
    out.extend_from_slice(&((ch * 2) as u16).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    out
}
