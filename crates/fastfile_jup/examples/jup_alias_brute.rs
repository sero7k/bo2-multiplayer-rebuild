//! Names for Modern Warfare III's hashed sound aliases by meeting in the
//! middle: every prefix a pattern file spells (`{a|b}` alternatives,
//! `{@words}` for each word of a word file) is hashed forward, and every
//! target hash is walked backwards over every tail of `--tail` characters
//! (FNV-1a undoes: h = (h' * prime⁻¹) ^ c). A tail that lands on a prefix's
//! state names the alias.
//!
//! ```text
//! cargo run --release -p fastfile_jup --example jup_alias_brute -- <hashes.tsv> <out.tsv> <patterns> [--tail N] [--chars abc..] [--threads N]
//! ```
//! `hashes.tsv`: hex hash in the first column. Hits are appended to
//! `out.tsv` as `hash<TAB>name`. Default tail 4 over `a-z0-9`.
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

use fastfile_jup::sound::{ALIAS_HASH_OFFSET, FNV_PRIME};

fn inverse(p: u64) -> u64 {
    // Newton's iteration for the inverse of an odd number mod 2^64.
    let mut x = p;
    for _ in 0..6 {
        x = x.wrapping_mul(2u64.wrapping_sub(p.wrapping_mul(x)));
    }
    x
}

fn expand(pattern: &str, words: &HashMap<String, Vec<String>>, out: &mut Vec<String>) {
    let Some(open) = pattern.find('{') else {
        out.push(pattern.to_owned());
        return;
    };
    let Some(close) = pattern[open..].find('}').map(|c| open + c) else {
        out.push(pattern.to_owned());
        return;
    };
    let inner = &pattern[open + 1..close];
    let alts: Vec<String> = match inner.strip_prefix('@') {
        Some(file) => words
            .get(file)
            .cloned()
            .unwrap_or_else(|| panic!("no word file {file}")),
        None => inner.split('|').map(str::to_owned).collect(),
    };
    for alt in alts {
        expand(
            &format!("{}{alt}{}", &pattern[..open], &pattern[close + 1..]),
            words,
            out,
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (hashes_file, out_file, patterns_file) = (&args[0], &args[1], &args[2]);
    let mut tail = 4usize;
    let mut chars: Vec<u8> = (b'a'..=b'z').chain(b'0'..=b'9').collect();
    let mut threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--tail" => tail = args[i + 1].parse().expect("tail"),
            "--chars" => chars = args[i + 1].bytes().collect(),
            "--threads" => threads = args[i + 1].parse().expect("threads"),
            x => panic!("unknown {x}"),
        }
        i += 2;
    }
    let targets: Vec<u64> = std::fs::read_to_string(hashes_file)
        .expect("hashes")
        .lines()
        .filter_map(|l| {
            u64::from_str_radix(l.split('\t').next()?.trim().trim_start_matches('#'), 16).ok()
        })
        .collect();
    // Word files named by `{@file}`.
    let text = std::fs::read_to_string(patterns_file).expect("patterns");
    let mut words: HashMap<String, Vec<String>> = HashMap::new();
    for line in text.lines() {
        let mut rest = line;
        while let Some(at) = rest.find("{@") {
            let end = rest[at..].find('}').map_or(rest.len(), |e| at + e);
            let file = &rest[at + 2..end];
            words.entry(file.to_owned()).or_insert_with(|| {
                std::fs::read_to_string(file)
                    .unwrap_or_else(|e| panic!("{file}: {e}"))
                    .lines()
                    .map(|w| w.trim().to_ascii_lowercase())
                    .filter(|w| !w.is_empty())
                    .collect()
            });
            rest = &rest[end..];
        }
    }
    let mut prefixes: HashMap<u64, String> = HashMap::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let mut names = Vec::new();
        expand(line, &words, &mut names);
        for n in names {
            let mut h = ALIAS_HASH_OFFSET;
            for b in n.bytes() {
                h = (h ^ u64::from(b)).wrapping_mul(FNV_PRIME);
            }
            prefixes.entry(h).or_insert(n);
        }
    }
    println!(
        "{} targets, {} prefixes, tail {tail} over {} chars, {threads} threads",
        targets.len(),
        prefixes.len(),
        chars.len()
    );
    let inv = inverse(FNV_PRIME);
    assert_eq!(inv.wrapping_mul(FNV_PRIME), 1);
    let found: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
    let chunk = targets.len().div_ceil(threads.max(1));
    std::thread::scope(|s| {
        for part in targets.chunks(chunk.max(1)) {
            let (prefixes, chars, found) = (&prefixes, &chars, &found);
            s.spawn(move || {
                let n = chars.len();
                let mut idx = vec![0usize; tail];
                for &t in part {
                    // Walk every tail backwards: the last character first.
                    idx.iter_mut().for_each(|x| *x = 0);
                    'outer: loop {
                        let mut h = t;
                        for k in (0..tail).rev() {
                            h = h.wrapping_mul(inv) ^ u64::from(chars[idx[k]]);
                        }
                        if let Some(p) = prefixes.get(&h) {
                            let tail_s: String = idx.iter().map(|&k| chars[k] as char).collect();
                            found.lock().unwrap().push((t, format!("{p}{tail_s}")));
                        }
                        // next tail
                        let mut k = 0;
                        loop {
                            if k == tail {
                                break 'outer;
                            }
                            idx[k] += 1;
                            if idx[k] < n {
                                break;
                            }
                            idx[k] = 0;
                            k += 1;
                        }
                    }
                }
            });
        }
    });
    let found = found.into_inner().unwrap();
    println!("{} hits", found.len());
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_file)
        .expect("out");
    for (h, n) in &found {
        writeln!(f, "{h:016x}\t{n}").ok();
        println!("HIT {n}");
    }
}
