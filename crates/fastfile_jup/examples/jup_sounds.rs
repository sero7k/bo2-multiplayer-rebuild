//! Modern Warfare III's sound aliases in some zones: how many sounds and
//! aliases each has, which names of a word list hash to an alias, and
//! chosen aliases decoded to WAV.
//!
//! ```text
//! cargo run -p fastfile_jup --example jup_sounds -- "F:\Modern Warfare III 1.21" <zone>... [options]
//!   --names <file>     alias names to try (one per line); hits are listed
//!   --tsv <file>       write every alias: hash, name (if known), zone, variants, seconds, channels, package
//!   --wav <dir>        decode the hits of --names (or --alias) into <dir>/<alias>[_n].wav
//!   --alias <name|0xhash>  an alias to decode (repeatable)
//!   --lang <xx>        language packs to prefer (default eng)
//! ```
//! Reads only; WAVs go where you say.
use std::collections::HashMap;
use std::path::PathBuf;

use fastfile_jup::container;
use fastfile_jup::oodle;
use fastfile_jup::sound::alias_hash;
use fastfile_jup::sound_bank::SoundBank;

fn main() {
    let mut a = std::env::args().skip(1);
    let install = PathBuf::from(a.next().expect("install dir"));
    let mut zones = Vec::new();
    let (mut names_file, mut tsv, mut wav_dir) = (None, None, None);
    let mut picks: Vec<String> = Vec::new();
    let mut lang = "eng".to_owned();
    while let Some(x) = a.next() {
        match x.as_str() {
            "--names" => names_file = a.next(),
            "--tsv" => tsv = a.next(),
            "--wav" => wav_dir = a.next(),
            "--alias" => picks.extend(a.next()),
            "--lang" => lang = a.next().unwrap_or(lang),
            _ => zones.push(x),
        }
    }
    let oodle = oodle::load(&install).expect("the game's Oodle DLL");
    let zone_dir = install.join("zone");
    let mut bank = SoundBank::new();
    for z in &zones {
        let p = zone_dir.join(format!("{z}.ff"));
        match container::read_zone(&p, oodle.as_ref()) {
            Ok((_, bytes)) => {
                let (s, l) = bank.add_zone(z, &bytes);
                println!("zone {z}: {s} sounds, {l} alias links");
            }
            Err(e) => println!("zone {z}: {e}"),
        }
    }
    let n = bank.index_packages(&zone_dir, &lang).expect("zone dir");
    let inline = bank.sounds.iter().filter(|s| s.inline.is_some()).count();
    println!(
        "{} sounds ({inline} inline, {n} streams found in packages), {} aliases",
        bank.sounds.len(),
        bank.aliases.len()
    );
    let mut names: HashMap<u64, String> = HashMap::new();
    if let Some(f) = names_file {
        let text = std::fs::read_to_string(&f).expect("names file");
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let h = alias_hash(line);
            if bank.aliases.contains_key(&h) {
                names.insert(h, line.to_owned());
            }
        }
        let mut hits: Vec<&String> = names.values().collect();
        hits.sort();
        println!("{} names hit", hits.len());
        for h in hits {
            println!("HIT {h}");
            picks.push(h.clone());
        }
    }
    if let Some(f) = tsv {
        let mut out = String::new();
        let mut keys: Vec<&u64> = bank.aliases.keys().collect();
        keys.sort();
        for k in keys {
            let v = &bank.aliases[k];
            let s = &bank.sounds[v[0]];
            out.push_str(&format!(
                "{k:016x}\t{}\t{}\t{}\t{:.2}\t{}\t{}\n",
                names.get(k).map_or("", String::as_str),
                bank.zones[s.zone],
                v.len(),
                s.asset.seconds(),
                s.asset.channels,
                bank.package_of(s)
                    .and_then(|p| p.file_name())
                    .map_or("inline".into(), |n| n.to_string_lossy())
            ));
        }
        std::fs::write(&f, out).expect("write tsv");
    }
    if let Some(dir) = wav_dir {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("wav dir");
        for p in &picks {
            let (h, label) = match p.strip_prefix("0x") {
                Some(x) => (u64::from_str_radix(x, 16).unwrap_or(0), p.clone()),
                None => (alias_hash(p), p.clone()),
            };
            let Some(v) = bank.alias_by_hash(h) else {
                println!("{label}: no such alias");
                continue;
            };
            for (i, &si) in v.iter().enumerate() {
                let s = &bank.sounds[si];
                match bank.decode(s, oodle.as_ref()) {
                    Ok(pcm) => {
                        let peak = pcm
                            .samples
                            .iter()
                            .map(|x| x.unsigned_abs())
                            .max()
                            .unwrap_or(0);
                        let rms = (pcm
                            .samples
                            .iter()
                            .map(|&x| f64::from(x) * f64::from(x))
                            .sum::<f64>()
                            / pcm.samples.len().max(1) as f64)
                            .sqrt();
                        let file = if v.len() > 1 {
                            dir.join(format!("{label}_{i}.wav"))
                        } else {
                            dir.join(format!("{label}.wav"))
                        };
                        std::fs::write(&file, pcm.wav()).expect("write wav");
                        println!(
                            "{label}[{i}]: {:.2}s {}ch {} Hz (record {:.2}s) peak {peak} rms {rms:.0} -> {}",
                            pcm.seconds(),
                            pcm.channels,
                            pcm.rate,
                            s.asset.seconds(),
                            file.display()
                        );
                    }
                    Err(e) => println!("{label}[{i}]: {e}"),
                }
            }
        }
    }
}
