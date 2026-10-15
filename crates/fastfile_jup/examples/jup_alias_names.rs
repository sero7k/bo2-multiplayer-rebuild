//! Names for Modern Warfare III's hashed sound aliases, by trying words:
//! every lower-case identifier found in some files (decompressed zones,
//! word lists), each with numbered and channel suffixes, and the
//! combinations of a pattern file (`a|b|c` alternatives per `{}` slot).
//!
//! ```text
//! cargo run --release -p fastfile_jup --example jup_alias_names -- <hashes.tsv> <out.tsv> [options]
//!   --scan <file>...     take identifiers from these files (raw bytes)
//!   --words <file>       one word per line
//!   --patterns <file>    lines like `dx_br_{ann|cmd}_{circle|gas}_{01|02}`
//! ```
//! `hashes.tsv`: the first column of each line is an alias hash in hex (as
//! `jup_sounds --tsv` writes). Hits are appended to `out.tsv` as
//! `hash<TAB>name`.
use std::collections::{HashMap, HashSet};
use std::io::Write;

use fastfile_jup::sound::alias_hash;

const SUFFIXES: &[&str] = &[
    "", "_l", "_r", "_lr", "_c", "_lp", "_loop", "_lp_l", "_lp_r", "_plr", "_npc", "_ui", "_2d",
    "_3d", "_st", "_stereo", "_mono", "_close", "_dist", "_far", "_near", "_int", "_ext", "_start",
    "_end", "_stop", "_in", "_out", "_on", "_off", "_open", "_close", "_alt", "_quad", "_lfe",
    "_main", "_intro", "_exit",
];

fn numbered(out: &mut Vec<String>, base: &str) {
    for s in SUFFIXES {
        out.push(format!("{base}{s}"));
    }
    for n in 0..=40 {
        out.push(format!("{base}_{n:02}"));
        out.push(format!("{base}_{n}"));
        out.push(format!("{base}_{n:03}"));
        out.push(format!("{base}{n:02}"));
    }
}

fn variants(token: &str) -> Vec<String> {
    let mut out = Vec::new();
    numbered(&mut out, token);
    // Without a trailing number (`_03`, `3`), renumbered.
    let t = token.trim_end_matches(|c: char| c.is_ascii_digit());
    let t = t.strip_suffix('_').unwrap_or(t);
    if t.len() != token.len() && !t.is_empty() {
        numbered(&mut out, t);
    }
    // Without a trailing suffix word.
    for s in SUFFIXES.iter().filter(|s| !s.is_empty()) {
        if let Some(b) = token.strip_suffix(s) {
            numbered(&mut out, b);
        }
    }
    out
}

fn identifiers(bytes: &[u8], into: &mut HashSet<String>) {
    let mut start = None;
    for (i, &b) in bytes.iter().enumerate() {
        let id = b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_';
        match (id, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                if i - s >= 4 && i - s <= 96 {
                    into.insert(String::from_utf8_lossy(&bytes[s..i]).into_owned());
                }
                start = None;
            }
            _ => {}
        }
    }
}

fn expand(pattern: &str, out: &mut Vec<String>) {
    let Some(open) = pattern.find('{') else {
        out.push(pattern.to_owned());
        return;
    };
    let Some(close) = pattern[open..].find('}').map(|c| open + c) else {
        out.push(pattern.to_owned());
        return;
    };
    for alt in pattern[open + 1..close].split('|') {
        expand(
            &format!("{}{alt}{}", &pattern[..open], &pattern[close + 1..]),
            out,
        );
    }
}

fn main() {
    let mut a = std::env::args().skip(1);
    let hashes_file = a.next().expect("hashes.tsv");
    let out_file = a.next().expect("out.tsv");
    let mut scans = Vec::new();
    let mut words = Vec::new();
    let mut patterns = Vec::new();
    let mut mode = "";
    for x in a {
        match x.as_str() {
            "--scan" | "--words" | "--patterns" => {
                mode = if x == "--scan" {
                    "s"
                } else if x == "--words" {
                    "w"
                } else {
                    "p"
                }
            }
            _ => match mode {
                "s" => scans.push(x),
                "w" => words.push(x),
                "p" => patterns.push(x),
                _ => panic!("argument {x} before --scan/--words/--patterns"),
            },
        }
    }
    let targets: HashSet<u64> = std::fs::read_to_string(&hashes_file)
        .expect("hashes")
        .lines()
        .filter_map(|l| {
            u64::from_str_radix(l.split('\t').next()?.trim().trim_start_matches('#'), 16).ok()
        })
        .collect();
    println!("{} hashes", targets.len());
    let mut found: HashMap<u64, String> = HashMap::new();
    let try_name = |n: &str, found: &mut HashMap<u64, String>| {
        let h = alias_hash(n);
        if targets.contains(&h) && !found.contains_key(&h) {
            found.insert(h, n.to_owned());
        }
    };
    let mut tokens: HashSet<String> = HashSet::new();
    for f in &scans {
        match std::fs::read(f) {
            Ok(b) => identifiers(&b, &mut tokens),
            Err(e) => eprintln!("{f}: {e}"),
        }
    }
    for f in &words {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}"));
        for l in text.lines() {
            let l = l.trim().to_ascii_lowercase();
            if !l.is_empty() {
                tokens.insert(l);
            }
        }
    }
    println!("{} tokens", tokens.len());
    for t in &tokens {
        for v in variants(t) {
            try_name(&v, &mut found);
        }
    }
    println!("{} hits from tokens", found.len());
    for f in &patterns {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}"));
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut names = Vec::new();
            expand(line, &mut names);
            for n in names {
                try_name(&n, &mut found);
            }
        }
    }
    println!("{} hits in all", found.len());
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&out_file)
        .expect("out");
    let mut v: Vec<_> = found.into_iter().collect();
    v.sort_by(|a, b| a.1.cmp(&b.1));
    for (h, n) in v {
        writeln!(f, "{h:016x}\t{n}").ok();
    }
}
