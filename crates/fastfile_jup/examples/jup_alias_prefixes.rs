//! Shared prefixes of Modern Warfare III's hashed sound aliases, without
//! their text: every target hash is walked backwards over every tail of
//! `--tail` characters (FNV-1a undoes), and a state reached from several
//! targets is the hash of a prefix they share (`dx_br_xxxx_yyyy_zzzz_` for
//! a voice line set). Random collisions of 64-bit states are negligible.
//! The states are sorted in passes by their top bits to bound memory.
//!
//! ```text
//! cargo run --release -p fastfile_jup --example jup_alias_prefixes -- <hashes.tsv> <out.tsv> [--tail N] [--min K] [--passes P] [--chars ..]
//! ```
//! `out.tsv`: `prefix state<TAB>count<TAB>target:tail,...`, each target
//! under the shared state (reached from at least `--min`, default 3,
//! targets) reached from the most targets. `--sep` (default `_`)
//! is undone after the tail, so the states are of prefixes ending before it
//! (which also keeps FNV-1a's look-alike states, a low byte apart, out).
use std::io::Write;
use std::sync::Mutex;

use fastfile_jup::sound::FNV_PRIME;

fn inverse(p: u64) -> u64 {
    let mut x = p;
    for _ in 0..6 {
        x = x.wrapping_mul(2u64.wrapping_sub(p.wrapping_mul(x)));
    }
    x
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (hashes_file, out_file) = (&args[0], &args[1]);
    let mut tail = 4usize;
    let mut min = 3usize;
    let mut passes = 256u64;
    let mut chars: Vec<u8> = (b'a'..=b'z').chain(b'0'..=b'9').collect();
    let mut sep: Vec<u8> = b"_".to_vec();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--tail" => tail = args[i + 1].parse().expect("tail"),
            "--min" => min = args[i + 1].parse().expect("min"),
            "--passes" => passes = args[i + 1].parse().expect("passes"),
            "--chars" => chars = args[i + 1].bytes().collect(),
            "--sep" => sep = args[i + 1].bytes().collect(),
            x => panic!("unknown {x}"),
        }
        i += 2;
    }
    assert!(passes.is_power_of_two());
    let shift = 64 - passes.trailing_zeros();
    let targets: Vec<u64> = std::fs::read_to_string(hashes_file)
        .expect("hashes")
        .lines()
        .filter_map(|l| {
            u64::from_str_radix(l.split('\t').next()?.trim().trim_start_matches('#'), 16).ok()
        })
        .collect();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let inv = inverse(FNV_PRIME);
    let n = chars.len();
    let tails = n.pow(tail as u32);
    println!(
        "{} targets, {tails} tails each, {passes} passes, {threads} threads",
        targets.len()
    );
    // Per target: the shared state reached from the most targets (the
    // true prefix; FNV-1a's look-alikes, a few tail characters off, are
    // reached from subsets of them), with its tail.
    let mut best: Vec<(usize, u64, u32)> = vec![(0, 0, 0); targets.len()];
    let mut shared = 0usize;
    for pass in 0..passes {
        let states: Mutex<Vec<(u64, u32, u32)>> = Mutex::new(Vec::new());
        let chunk = targets.len().div_ceil(threads);
        std::thread::scope(|s| {
            for (ci, part) in targets.chunks(chunk.max(1)).enumerate() {
                let (states, chars, sep) = (&states, &chars, &sep);
                s.spawn(move || {
                    let mut local = Vec::new();
                    let mut tail_ix = vec![0usize; tail];
                    for (ti, &t) in part.iter().enumerate() {
                        let ti = (ci * chunk + ti) as u32;
                        tail_ix.iter_mut().for_each(|x| *x = 0);
                        for k in 0..tails {
                            // tail_ix[0] is the last character
                            let mut h = t;
                            for &c in &tail_ix {
                                h = h.wrapping_mul(inv) ^ u64::from(chars[c]);
                            }
                            for &c in sep.iter().rev() {
                                h = h.wrapping_mul(inv) ^ u64::from(c);
                            }
                            if h >> shift == pass {
                                local.push((h, ti, k as u32));
                            }
                            for x in tail_ix.iter_mut() {
                                *x += 1;
                                if *x < n {
                                    break;
                                }
                                *x = 0;
                            }
                        }
                    }
                    states.lock().unwrap().extend(local);
                });
            }
        });
        let mut v = states.into_inner().unwrap();
        v.sort_unstable();
        let mut i = 0;
        while i < v.len() {
            let mut j = i + 1;
            while j < v.len() && v[j].0 == v[i].0 {
                j += 1;
            }
            let mut ts: Vec<u32> = v[i..j].iter().map(|x| x.1).collect();
            ts.dedup();
            if ts.len() >= min {
                shared += 1;
                for &(st, ti, k) in &v[i..j] {
                    let b = &mut best[ti as usize];
                    if ts.len() > b.0 {
                        *b = (ts.len(), st, k);
                    }
                }
            }
            i = j;
        }
        if pass % 8 == 7 {
            println!(
                "pass {} of {passes}: {shared} shared states so far",
                pass + 1
            );
        }
    }
    let tail_text = |k: u32| {
        let mut rest = k as usize;
        let mut s = vec![0u8; tail];
        for c in s.iter_mut().rev() {
            *c = chars[rest % n];
            rest /= n;
        }
        String::from_utf8_lossy(&s).into_owned()
    };
    let mut out = std::fs::File::create(out_file).expect("out");
    let mut groups: std::collections::BTreeMap<u64, Vec<usize>> = Default::default();
    for (ti, &(c, st, _)) in best.iter().enumerate() {
        if c > 0 {
            groups.entry(st).or_default().push(ti);
        }
    }
    for (st, tis) in &groups {
        let list: Vec<String> = tis
            .iter()
            .map(|&ti| format!("{:016x}:{}", targets[ti], tail_text(best[ti].2)))
            .collect();
        writeln!(out, "{st:016x}\t{}\t{}", tis.len(), list.join(",")).ok();
    }
    println!(
        "{shared} shared states; {} prefixes best for {} targets",
        groups.len(),
        groups.values().map(Vec::len).sum::<usize>()
    );
}
