//! Prints what a map zone's placed static models read as:
//! `jup_smodels <zone image (.zone, decompressed)> [install]`.

use fastfile_jup::map::{ZoneImage, models, world};

fn main() {
    let path = std::env::args().nth(1).expect("usage: jup_smodels <file.zone> [install]");
    let install = std::env::args().nth(2);
    let t = std::time::Instant::now();
    let zone = ZoneImage::parse(std::fs::read(&path).expect("read")).expect("zone");
    let z = &zone.bytes;
    let (list, count) = world::find_surfaces(z).expect("surfaces");
    println!("surface list {list:#x} x{count} ({:.1}s)", t.elapsed().as_secs_f32());
    let mut report = Vec::new();
    let placed = models::read(&zone, list, &mut report);
    for l in &report {
        println!("{l}");
    }
    println!("read in {:.1}s", t.elapsed().as_secs_f32());
    let paks = install.as_ref().map(|i| fastfile_jup::map::materials::Packages::open(std::path::Path::new(i)));
    let oodle = install.as_ref().map(|i| fastfile_jup::oodle::load(std::path::Path::new(i)).expect("oodle"));
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for i in &placed.instances {
        for k in 0..3 {
            lo[k] = lo[k].min(i.origin[k]);
            hi[k] = hi[k].max(i.origin[k]);
        }
    }
    println!("instances {} bounds {lo:?} {hi:?}", placed.instances.len());
    // Per model: placements, LOD0 consistency.
    let mut uses: std::collections::HashMap<usize, usize> = Default::default();
    for i in &placed.instances {
        *uses.entry(i.smodel).or_default() += 1;
    }
    let (mut ok, mut bad, mut nosurfs, mut geo_ok, mut geo_bad, mut streamed, mut fetched) = (0, 0, 0, 0, 0, 0, 0);
    let mut first_err = Vec::new();
    let mut named: Vec<(usize, String, f32)> = Vec::new();
    let mut tried = 0;
    for (sm, n) in &uses {
        let Some(Some(e)) = placed.table.get(*sm) else { continue };
        let Some(d) = placed.models.get(e) else { continue };
        let l0 = d.lods[0];
        named.push((*n, d.name.clone(), d.lods.last().map_or(0.0, |l| l.distance)));
        let Some(&h) = placed.surfs_at.get(&l0.surfs) else {
            nosurfs += 1;
            continue;
        };
        match fastfile_jup::surfs::parse(z, h) {
            Ok(r) if r.surfaces.len() == l0.surfaces => {
                ok += 1;
                let blob: Option<Vec<u8>> = match r.streamed {
                    Some((key, len)) => {
                        streamed += 1;
                        if tried < 400 {
                            tried += 1;
                            let got = paks.as_ref().zip(oodle.as_ref()).and_then(|(p, o)| p.fetch(key, o.as_ref()));
                            if let Some(b) = &got {
                                fetched += 1;
                                if fetched <= 3 {
                                    println!("  fetched {} key {key:016x}: {} bytes (want {len})", d.name, b.len());
                                }
                            } else if first_err.len() < 10 {
                                first_err.push(format!("{}: key {key:016x} not in the packages", d.name));
                            }
                            got
                        } else {
                            None
                        }
                    }
                    None => Some(z[r.blob.0..r.blob.0 + r.blob.1].to_vec()),
                };
                let Some(blob) = blob else { continue };
                for s in 0..r.surfaces.len() {
                    match fastfile_jup::surfs::geometry_in(&blob, &r, s) {
                        Ok(_) => geo_ok += 1,
                        Err(e) => {
                            geo_bad += 1;
                            if first_err.len() < 10 {
                                first_err.push(format!("{}: {e}", d.name));
                            }
                        }
                    }
                }
            }
            Ok(r) => {
                bad += 1;
                if first_err.len() < 10 {
                    first_err.push(format!("{}: lod0 {} surfaces, mesh {}", d.name, l0.surfaces, r.surfaces.len()));
                }
            }
            Err(e) => {
                bad += 1;
                if first_err.len() < 10 {
                    first_err.push(format!("{}: {e}", d.name));
                }
            }
        }
    }
    println!(
        "models used {}: lod0 mesh fits {ok} ({streamed} streamed, {fetched} of {tried} fetched), not {bad}, no mesh header {nosurfs}; surfaces decoded {geo_ok}, failed {geo_bad}",
        uses.len()
    );
    for e in first_err {
        println!("  {e}");
    }
    named.sort_by(|a, b| b.0.cmp(&a.0));
    for (n, name, d) in named.iter().take(30) {
        println!("  {n:6} {name} (last lod {d})");
    }
    // Pairing diagnostic: mesh entries (by kind) against the headers found,
    // in windows, at small shifts.
    if std::env::var("JUP_PAIR").is_ok() {
        let heads = models::surfs_headers(z);
        let kind = |i: usize| {
            let o = zone.list_at + 16 * i + 8;
            u64::from_le_bytes(z[o..o + 8].try_into().unwrap())
        };
        let ents: Vec<usize> = (0..zone.types.len()).filter(|&i| zone.types[i] == 8).collect();
        let mut expect: std::collections::HashMap<usize, usize> = Default::default();
        for d in placed.models.values() {
            for l in &d.lods {
                expect.insert(l.surfs, l.surfaces);
            }
        }
        let counts: Vec<Option<usize>> =
            heads.iter().map(|&h| fastfile_jup::surfs::parse(z, h).map(|r| r.surfaces.len()).ok()).collect();
        let ends: Vec<usize> = heads.iter().map(|&h| fastfile_jup::surfs::parse(z, h).map_or(h, |r| r.end)).collect();
        let only_i: Vec<usize> = ents.iter().copied().filter(|&e| kind(e) == u64::MAX).collect();
        println!("pairing: {} entries, {} headers", only_i.len(), heads.len());
        for w in (0..only_i.len()).step_by(250) {
            let mut fits: Vec<(usize, i64)> = Vec::new();
            let mut n = 0;
            for d in -600i64..=600 {
                let mut fit = 0;
                n = 0;
                for k in w..(w + 250).min(only_i.len()) {
                    let Some(&want) = expect.get(&only_i[k]) else { continue };
                    n += 1;
                    let j = k as i64 + d;
                    if j >= 0 && counts.get(j as usize).copied().flatten() == Some(want) {
                        fit += 1;
                    }
                }
                fits.push((fit, d));
            }
            fits.sort_by(|a, b| b.cmp(a));
            println!("  {w:5} of {n}: best {:?}", &fits[..3]);
        }
        let mut gaps = Vec::new();
        for k in 0..heads.len().saturating_sub(1) {
            if heads[k + 1] != ends[k] {
                gaps.push((k, heads[k + 1] - ends[k]));
            }
        }
        println!("header gaps {}: {:?}", gaps.len(), &gaps[..gaps.len().min(60)]);
        for &(k, g) in gaps.iter().filter(|g| g.1 >= 0x130).take(8) {
            let at = ends[k];
            println!("gap after {k} at {at:#x} ({g} bytes): {:?}", fastfile_jup::surfs::parse(z, at).err());
            for row in 0..8 {
                let o = at + 16 * row;
                println!("   {o:08x} {:02x?}", &z[o..o + 16]);
            }
        }
        println!("first header {:#x}, last end {:#x}", heads[0], ends[ends.len() - 1]);
    }
    let mats = placed
        .models
        .values()
        .map(|d| (d.materials.iter().filter(|m| m.is_some()).count(), d.materials.len()))
        .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    println!("materials resolved {} of {}", mats.0, mats.1);
}
