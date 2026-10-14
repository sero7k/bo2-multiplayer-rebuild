//! bo2mp: what the guest-gun lane reads of one Black Ops III gun:
//! `t7_probe <core_common.ff> <weapon> [anim ...]`. Prints the view and
//! world models' skeletons (bone names, parents, bind poses), the
//! animations' headers and data sizes, and the weapon's attached models.

use std::path::Path;

use fastfile_t7::walk::capture::FieldValue;
use fastfile_t7::walk::{Options, walk_with};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(path), Some(weapon)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: t7_probe <zone.ff> <weapon> [anim ...]");
        std::process::exit(2);
    };
    let t = std::time::Instant::now();
    let zone = fastfile_t7::Zone::open(Path::new(path)).expect("zone");
    eprintln!("opened in {:.1}s ({} MB)", t.elapsed().as_secs_f32(), zone.image.len() >> 20);
    let w = walk_with(&zone, Options { keep_anims: true, ..Options::default() });
    eprintln!("walked in {:.1}s: {:?}; {} anims kept", t.elapsed().as_secs_f32(), w.stop, w.capture.anims.len());
    let strings = &zone.assets.script_strings;
    let ss = |i: u32| strings.get(i as usize).cloned().flatten().unwrap_or_else(|| format!("<{i}>"));
    if weapon == "--list" {
        let i32_at = |a: usize| i32::from_le_bytes(zone.image[a..a + 4].try_into().unwrap());
        let zm = std::env::var("T7_LIST_ZM").is_ok();
        for r in w.capture.weapons.iter().filter(|r| if zm { !r.name.ends_with("_mp") } else { r.name.ends_with("_mp") }) {
            let def = r.def_at.or_else(|| {
                let o = r.def_shared_with.as_deref()?;
                w.capture.weapons.iter().find(|x| x.name == o)?.def_at
            });
            let (ty, class, inv, fire) = def.map_or((-1, -1, -1, -1), |d| (i32_at(d + 0x6c), i32_at(d + 0x70), i32_at(d + 0x7c), i32_at(d + 0x80)));
            let gun = r.field("gunModel").map(|v| format!("{v:?}")).unwrap_or_default();
            println!("{} base {} display {} type {ty} class {inv}/{class} fire {fire} {gun}", r.name, r.base_name, r.display_name);
        }
        for z in std::env::var("T7_LOC").ok().into_iter().flat_map(|v| v.split(';').map(str::to_owned).collect::<Vec<_>>()) {
            let lz = fastfile_t7::Zone::open(Path::new(&z)).expect("loc zone");
            let lw = walk_with(&lz, Options { keep_anims: true, ..Options::default() });
            let n = lw.capture.localize.iter().filter(|(k, _)| k.starts_with("WEAPON_")).count();
            println!("localize {z}: {:?}; {} strings, {n} WEAPON_", lw.stop, lw.capture.localize.len());
            for (k, v) in lw.capture.localize.iter().filter(|(k, _)| k.starts_with("WEAPON_")).take(400) {
                println!("  {k} = {v}");
            }
        }
        return;
    }
    let Some(r) = w.capture.weapon(weapon) else {
        eprintln!("no weapon {weapon}");
        return;
    };
    println!("weapon {} def_at {:x?} header {:#x}", r.name, r.def_at, r.header_at);
    for (f, v) in &r.fields {
        if f.starts_with("attach") || f == "gunModel" || f == "worldModel" {
            match v {
                FieldValue::Asset { name, .. } => println!("  {f} = {name}"),
                FieldValue::Str(s) => println!("  {f} = \"{s}\""),
            }
        }
    }
    println!("  hide tags: {:?}", r.hide_tags.iter().map(|&t| ss(t)).collect::<Vec<_>>());
    let models: Vec<String> = r
        .fields
        .iter()
        .filter_map(|(f, v)| match v {
            FieldValue::Asset { name, .. } if f == "gunModel" || f == "worldModel" || f.starts_with("attachViewModel[") || f.starts_with("attachWorldModel[") => Some(name.clone()),
            _ => None,
        })
        .collect();
    for m in models {
        let Some(m) = w.capture.model(&m) else { continue };
        let k = &m.skeleton;
        println!("model {} bones {} roots {} lods {}", m.name, m.bones, k.num_root_bones, m.lods.len());
        for i in 0..m.bones {
            let parent = if i < k.num_root_bones as usize {
                "-".to_string()
            } else {
                let step = k.parent_list.get(i - k.num_root_bones as usize).copied().unwrap_or(0) as usize;
                format!("{}", i.saturating_sub(step))
            };
            let q = k.quats.get(i.wrapping_sub(k.num_root_bones as usize)).copied();
            let tr = k.trans.get(i.wrapping_sub(k.num_root_bones as usize)).copied();
            println!(
                "  {i:2} {:<24} parent {parent:>3} part {:3} base {:?} rest q {:?} t {:?}",
                ss(k.bone_names.get(i).copied().unwrap_or(0)),
                k.part_classification.get(i).copied().unwrap_or(0),
                k.base_mat.get(i).map(|b| b.map(|x| (x * 1000.0).round() / 1000.0)),
                q,
                tr
            );
        }
        // The binormal sign: per tangent top-bits value, how often
        // cross(n, t) agrees with the UV-derived bitangent.
        if let Some(lod) = m.lods.iter().max_by_key(|l| w.capture.mesh(&l.mesh).map_or(0, |me| me.vert_count))
            && let Some(me) = w.capture.mesh(&lod.mesh)
        {
            let paks = fastfile_t7::xpak::XpakSet::open_dir(Path::new(path).parent().unwrap()).map(|x| x.0).unwrap_or_default();
            let data = match (me.data, me.xpak) {
                (Some((at, len)), _) => zone.image[at..at + len].to_vec(),
                (None, Some(x)) => paks.read(x.key).unwrap_or_default(),
                _ => Vec::new(),
            };
            if let Ok(g) = fastfile_t7::mesh::decode(me, &data) {
                let mut tally = [[0usize; 2]; 4];
                for sf in &g.surfaces {
                    for t in sf.indices.chunks_exact(3) {
                        let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
                        let sub = |x: [f32; 3], y: [f32; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
                        let (e1, e2) = (sub(g.positions[b], g.positions[a]), sub(g.positions[c], g.positions[a]));
                        let (du1, dv1) = (g.uvs[b][0] - g.uvs[a][0], g.uvs[b][1] - g.uvs[a][1]);
                        let (du2, dv2) = (g.uvs[c][0] - g.uvs[a][0], g.uvs[c][1] - g.uvs[a][1]);
                        let r = du1 * dv2 - du2 * dv1;
                        if r.abs() < 1e-8 {
                            continue;
                        }
                        let bt: [f32; 3] = std::array::from_fn(|k| (e2[k] * du1 - e1[k] * du2) / r);
                        let (n, tg) = (g.normals[a], g.tangents[a]);
                        let cr = [n[1] * tg[2] - n[2] * tg[1], n[2] * tg[0] - n[0] * tg[2], n[0] * tg[1] - n[1] * tg[0]];
                        let d = cr[0] * bt[0] + cr[1] * bt[1] + cr[2] * bt[2];
                        tally[usize::from(g.tangent_w[a] & 3)][usize::from(d < 0.0)] += 1;
                    }
                }
                println!("  binormal: per tangent w (agree, disagree with cross(n, t)): {tally:?}");
            }
        }
        for (li, lod) in m.lods.iter().enumerate() {
            if let Some(me) = w.capture.mesh(&lod.mesh) {
                println!("  lod{li} {} verts {} surfaces {} materials {:?}", lod.mesh, me.vert_count, me.surfaces.len(), lod.materials);
            }
        }
    }
    // `T7_SND=<zone>`: that zone's aliases whose names start with `wpn_ar_standard`.
    if let Ok(z) = std::env::var("T7_SND") {
        let sz = fastfile_t7::Zone::open(Path::new(&z)).expect("snd zone");
        let sw = walk_with(&sz, Options { keep_anims: true, ..Options::default() });
        println!("sound zone {z}: {:?}, {} aliases", sw.stop, sw.capture.sound_aliases.len());
        let want = std::env::var("T7_SND_PREFIX").unwrap_or_else(|_| "wpn_ar_standard".into());
        // The banks beside the zone (`snd/all/<zone>.all.sabl`, `.sabs`).
        let dir = Path::new(&z).parent().unwrap().join("snd").join("all");
        let stem = Path::new(&z).file_stem().unwrap().to_string_lossy().into_owned();
        let banks: Vec<fastfile_t7::sab::SabFile> = ["sabl", "sabs"]
            .iter()
            .filter_map(|ext| match fastfile_t7::sab::SabFile::open(&dir.join(format!("{stem}.all.{ext}"))) {
                Ok(b) => Some(b),
                Err(e) => {
                    println!("bank {ext}: {e}");
                    None
                }
            })
            .collect();
        for b in &banks {
            let fmt: std::collections::BTreeMap<u8, usize> = b.entries.iter().fold(Default::default(), |mut m, e| {
                *m.entry(e.format).or_default() += 1;
                m
            });
            let bad = b.entries.iter().filter(|e| e.offset + u64::from(e.size) > b.header.file_size).count();
            println!("bank {} v{} {} entries, formats {fmt:?}, {bad} past the end, deps {:?}", b.path.display(), b.header.version, b.entries.len(), b.header.dependencies);
            for e in b.entries.iter().take(3) {
                println!("   {e:?}");
            }
        }
        let mut found = 0;
        for a in sw.capture.sound_aliases.iter().filter(|a| a.name.starts_with(&want)) {
            let hit = banks.iter().find_map(|b| b.find(a.asset_id).map(|e| (b, e)));
            if let Some((b, e)) = hit {
                found += 1;
                if found <= 3 {
                    let bytes = b.read(e).unwrap_or_default();
                    println!("  {} -> {:?} starts {:02x?}", a.name, e, &bytes[..bytes.len().min(8)]);
                }
            }
        }
        println!("  {found} of the {want} aliases found in the banks");
        for a in sw.capture.sound_aliases.iter().filter(|a| a.name.starts_with(&want)).take(40) {
            println!("  {} id {:#010x} file {} secondary {}", a.name, a.asset_id, a.file, a.secondary);
            for row in a.record.chunks(24) {
                println!("     {}", row.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "));
            }
        }
    }
    let mut anims: Vec<String> = args[3..].to_vec();
    if anims.is_empty() {
        anims = r.anims.iter().take(6).map(|a| a.2.clone()).collect();
    }
    for a in &anims {
        let Some(a) = w.capture.anim(a) else {
            println!("anim {a}: not kept");
            continue;
        };
        println!(
            "anim {} frames {} names {} notifies {:?}",
            a.name,
            a.numframes,
            a.names.len(),
            a.notifies.iter().map(|(n, t, p, q)| (ss(*n), *t, ss(*p), ss(*q))).collect::<Vec<_>>()
        );
        for row in a.header.chunks(16).take(16) {
            println!("   {}", row.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "));
        }
        println!(
            "   byte {:?} short {:?} int {:?} rshort {:?} rbyte {:?} rint {:?} idx {:?}",
            a.data_byte.map(|x| x.1),
            a.data_short.map(|x| x.1),
            a.data_int.map(|x| x.1),
            a.random_data_short.map(|x| x.1),
            a.random_data_byte.map(|x| x.1),
            a.random_data_int.map(|x| x.1),
            a.indices.map(|x| x.1)
        );
        println!("   bones {:?}", a.names.iter().map(|&n| ss(n)).collect::<Vec<_>>());
        if let Some((at, len)) = a.data_short {
            let sh: Vec<i16> = zone.image[at..at + len.min(64)].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
            println!("   shorts {sh:?}");
            for q in sh.chunks_exact(4).take(4) {
                let n16 = q.iter().map(|&v| (v as f32 / 32767.0).powi(2)).sum::<f32>().sqrt();
                println!("   quat {q:?} |i16| {n16:.3}");
            }
        }
        if let Some((at, len)) = a.data_int {
            let f: Vec<f32> = zone.image[at..at + len.min(48)].chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
            println!("   ints as f32 {f:?}");
        }
        if let Some((at, len)) = a.random_data_short {
            let sh: Vec<i16> = zone.image[at..at + len.min(32)].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
            println!("   rshorts {sh:?}");
        }
        if let Some((at, len)) = a.data_byte {
            println!("   bytes {:?}", &zone.image[at..at + len.min(40)]);
        }
    }
}
