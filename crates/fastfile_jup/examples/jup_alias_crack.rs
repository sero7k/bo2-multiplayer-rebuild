//! Text for alias-prefix states (from `jup_alias_prefixes`) by meeting in
//! the middle: the left pattern is hashed forward into a table, and each
//! state is walked backwards through the right pattern's segments until it
//! meets one.
//!
//! ```text
//! cargo run --release -p fastfile_jup --example jup_alias_crack -- <states.tsv> <out.tsv> <left> <right>
//! ```
//! Patterns are segments: literal text, `{a|b}` alternatives, `{@file}` a
//! word per line of a file, `{?N}` every string of N characters of a-z0-9.
//! E.g. left `dx_{br|mp}_{?4}`, right `_{@g3.txt}_{@g4.txt}`. A state's
//! line in `states.tsv` starts with its hex hash; hits are written as
//! `state<TAB>text`.
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

use fastfile_jup::sound::{ALIAS_HASH_OFFSET, FNV_PRIME};

const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

fn inverse(p: u64) -> u64 {
    let mut x = p;
    for _ in 0..6 {
        x = x.wrapping_mul(2u64.wrapping_sub(p.wrapping_mul(x)));
    }
    x
}

/// A pattern's segments, each a list of alternatives.
fn segments(pattern: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut rest = pattern;
    while !rest.is_empty() {
        match rest.find('{') {
            Some(0) => {
                let close = rest.find('}').expect("unclosed {");
                let inner = &rest[1..close];
                let alts: Vec<String> = if let Some(file) = inner.strip_prefix('@') {
                    std::fs::read_to_string(file)
                        .unwrap_or_else(|e| panic!("{file}: {e}"))
                        .lines()
                        .map(|w| w.trim().to_ascii_lowercase())
                        .filter(|w| !w.is_empty())
                        .collect()
                } else if let Some(n) = inner.strip_prefix('?') {
                    let n: u32 = n.parse().expect("{?N}");
                    let total = CHARS.len().pow(n);
                    (0..total)
                        .map(|mut k| {
                            let mut s = vec![0u8; n as usize];
                            for c in s.iter_mut().rev() {
                                *c = CHARS[k % CHARS.len()];
                                k /= CHARS.len();
                            }
                            String::from_utf8(s).unwrap()
                        })
                        .collect()
                } else {
                    inner.split('|').map(str::to_owned).collect()
                };
                out.push(alts);
                rest = &rest[close + 1..];
            }
            Some(at) => {
                out.push(vec![rest[..at].to_owned()]);
                rest = &rest[at..];
            }
            None => {
                out.push(vec![rest.to_owned()]);
                rest = "";
            }
        }
    }
    out
}

fn forward(segs: &[Vec<String>], h: u64, text: &mut String, table: &mut HashMap<u64, String>) {
    let Some((first, rest)) = segs.split_first() else {
        table.entry(h).or_insert_with(|| text.clone());
        return;
    };
    for alt in first {
        let mut g = h;
        for b in alt.bytes() {
            g = (g ^ u64::from(b)).wrapping_mul(FNV_PRIME);
        }
        let len = text.len();
        text.push_str(alt);
        forward(rest, g, text, table);
        text.truncate(len);
    }
}

#[allow(clippy::too_many_arguments)]
fn backward(
    segs: &[Vec<String>],
    h: u64,
    inv: u64,
    table: &HashMap<u64, String>,
    tail: &mut Vec<String>,
    state: u64,
    hits: &Mutex<Vec<(u64, String)>>,
) {
    let Some((last, rest)) = segs.split_last() else {
        if let Some(left) = table.get(&h) {
            let right: String = tail.iter().rev().cloned().collect();
            hits.lock().unwrap().push((state, format!("{left}{right}")));
        }
        return;
    };
    for alt in last {
        let mut g = h;
        for b in alt.bytes().rev() {
            g = g.wrapping_mul(inv) ^ u64::from(b);
        }
        tail.push(alt.clone());
        backward(rest, g, inv, table, tail, state, hits);
        tail.pop();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (states_file, out_file, left, right) = (&args[0], &args[1], &args[2], &args[3]);
    let states: Vec<u64> = std::fs::read_to_string(states_file)
        .expect("states")
        .lines()
        .filter_map(|l| u64::from_str_radix(l.split('\t').next()?.trim(), 16).ok())
        .collect();
    let mut table = HashMap::new();
    forward(
        &segments(left),
        ALIAS_HASH_OFFSET,
        &mut String::new(),
        &mut table,
    );
    let right = segments(right);
    let combos: usize = right.iter().map(Vec::len).product();
    println!(
        "{} states, {} left strings, {combos} right strings each",
        states.len(),
        table.len()
    );
    let inv = inverse(FNV_PRIME);
    let hits = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|s| {
        for part in states.chunks(states.len().div_ceil(threads).max(1)) {
            let (table, right, hits) = (&table, &right, &hits);
            s.spawn(move || {
                for &st in part {
                    backward(right, st, inv, table, &mut Vec::new(), st, hits);
                }
            });
        }
    });
    let hits = hits.into_inner().unwrap();
    println!("{} hits", hits.len());
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_file)
        .expect("out");
    for (st, t) in &hits {
        writeln!(f, "{st:016x}\t{t}").ok();
        println!("HIT {st:016x} {t}");
    }
}
