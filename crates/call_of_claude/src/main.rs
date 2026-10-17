//! `call_of_claude`: makes the "Call of Claude" folder - only the files the
//! rebuild reads, copied out of each installed game (see `rules`).
//!
//! ```text
//! call_of_claude pack "D:\Call of Claude"            copy (again: only what changed)
//! call_of_claude pack "D:\Call of Claude" --dry-run  sizes only, nothing copied
//! call_of_claude pack <dest> --from iw=E:\iw_game_files   a game not in game_paths.cfg
//! ```
//!
//! The games' folders come from `game_paths.cfg` (beside this tool or the
//! game, or `IW4L_GAME_PATHS`), as the game's own Game Folders set them.
//! Each game lands in `<dest>/<key>` laid out as its install is; the
//! marker `call_of_claude.txt` at the top says what is in it. The game's
//! first launch then takes the folder in place of the games.

mod glob;
mod rules;

use std::path::{Path, PathBuf};

use asset_transport::{call_of_claude, game_paths};

struct Args {
    dest: PathBuf,
    dry_run: bool,
    from: Vec<(String, PathBuf)>,
}

fn args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    match it.next().as_deref() {
        Some("pack") => {}
        _ => return Err(usage()),
    }
    let dest = it.next().map(PathBuf::from).ok_or_else(usage)?;
    let mut out = Args { dest, dry_run: false, from: Vec::new() };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dry-run" => out.dry_run = true,
            "--from" => {
                let v = it.next().ok_or_else(usage)?;
                let (k, p) = v.split_once('=').ok_or_else(usage)?;
                out.from.push((k.trim().to_ascii_lowercase(), PathBuf::from(p.trim().trim_matches('"'))));
            }
            _ => return Err(usage()),
        }
    }
    Ok(out)
}

fn usage() -> String {
    "usage: call_of_claude pack <destination folder> [--dry-run] [--from <game>=<folder> ...]".to_owned()
}

/// One file to copy: where from, where to, its size.
struct Item {
    from: PathBuf,
    to: PathBuf,
    size: u64,
}

/// Every file under `dir`, as (path, path relative to `dir` with `/`).
fn walk(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            let Ok(t) = e.file_type() else { continue };
            if t.is_dir() {
                stack.push(p);
            } else if t.is_file()
                && let Ok(rel) = p.strip_prefix(dir)
            {
                let rel = rel.to_string_lossy().replace('\\', "/");
                out.push((p, rel));
            }
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// Whether the copy at `to` is already this file (same size, not older).
fn up_to_date(from: &Path, to: &Path) -> bool {
    let (Ok(a), Ok(b)) = (std::fs::metadata(from), std::fs::metadata(to)) else { return false };
    a.len() == b.len() && match (a.modified(), b.modified()) {
        (Ok(x), Ok(y)) => y >= x,
        _ => true,
    }
}

fn gb(n: u64) -> String {
    format!("{:.2} GB", n as f64 / 1e9)
}

fn main() {
    let a = match args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let mut sources: Vec<(String, PathBuf)> = game_paths::load();
    for (k, p) in &a.from {
        sources.retain(|(key, _)| key != k);
        sources.push((k.clone(), p.clone()));
    }
    sources.retain(|(_, p)| p.is_dir());
    if !sources.iter().any(|(k, _)| k == "bo2") {
        eprintln!("Black Ops II's folder is not set (game_paths.cfg at {}, or --from bo2=<folder>).", game_paths::config_path().display());
        std::process::exit(1);
    }
    if sources.iter().any(|(_, p)| p.starts_with(&a.dest)) {
        eprintln!("The destination is inside a game's folder; pick another.");
        std::process::exit(1);
    }
    let mut plan: Vec<(String, Vec<Item>, u64)> = Vec::new();
    let (mut kept, mut left) = (0u64, 0u64);
    for (key, dir) in &sources {
        let rules = rules::for_game(key);
        let mut items = Vec::new();
        let mut dropped = 0u64;
        for (path, rel) in walk(dir) {
            let size = std::fs::metadata(&path).map_or(0, |m| m.len());
            if rules.keeps(&rel) {
                items.push(Item { to: a.dest.join(key).join(&rel), from: path, size });
            } else {
                dropped += size;
            }
        }
        let size: u64 = items.iter().map(|i| i.size).sum();
        println!("{key:>7}: keep {:>10} in {:>6} files, leave {:>10}   ({})", gb(size), items.len(), gb(dropped), dir.display());
        kept += size;
        left += dropped;
        plan.push((key.clone(), items, size));
    }
    println!("  total: keep {}, leave {}", gb(kept), gb(left));
    if a.dry_run {
        return;
    }
    if let Err(e) = std::fs::create_dir_all(&a.dest) {
        eprintln!("cannot make {}: {e}", a.dest.display());
        std::process::exit(1);
    }
    let started = std::time::Instant::now();
    let (mut done, mut copied, mut failed) = (0u64, 0u64, 0usize);
    for (key, items, _) in &plan {
        for it in items {
            done += it.size;
            if up_to_date(&it.from, &it.to) {
                continue;
            }
            if let Some(parent) = it.to.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::copy(&it.from, &it.to) {
                Ok(n) => copied += n,
                Err(e) => {
                    failed += 1;
                    eprintln!("could not copy {}: {e}", it.from.display());
                }
            }
            let secs = started.elapsed().as_secs_f64().max(0.1);
            println!("[{:5.1}%] {key}: {} ({:.0} MB/s)", done as f64 * 100.0 / kept.max(1) as f64, it.to.strip_prefix(&a.dest).unwrap_or(&it.to).display(), copied as f64 / 1e6 / secs);
        }
    }
    let mut marker = String::from("Call of Claude: the files the BO2 rebuild reads, copied out of each game by call_of_claude.\nOne folder per game (its key in game_paths.cfg):\n");
    for (key, items, size) in &plan {
        marker.push_str(&format!("{key}: {} files, {}\n", items.len(), gb(*size)));
    }
    let _ = std::fs::write(a.dest.join(call_of_claude::MARKER), marker);
    println!("done: {} copied in {:.0} s, {failed} failed. Folder: {}", gb(copied), started.elapsed().as_secs_f64(), a.dest.display());
    if failed > 0 {
        std::process::exit(1);
    }
}
