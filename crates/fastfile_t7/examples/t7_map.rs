//! Black Ops III map zones: `t7_map <zone.ff>...` walks a map zone and
//! prints what the map reader (`fastfile_t7::map`) gets from it.
//! `T7_MAP_DUMP=materials|meshes|smodels|ents` prints those in full
//! (`ents` also from a zone whose walk stops after its map entities);
//! `T7_MAP_MESH=<name>` writes that mesh's data (`<name>.mesh.bin`, from
//! the zone or its `.xpak` entry) to the current directory.
use fastfile_t7::walk::{Options, Stop, walk_with};

fn main() {
    let dump = std::env::var("T7_MAP_DUMP").unwrap_or_default();
    for path in std::env::args().skip(1) {
        let path = std::path::PathBuf::from(path);
        if std::env::var_os("T7_MAP_LOAD").is_some() {
            let started = std::time::Instant::now();
            match fastfile_t7::map::T7Map::load(&fastfile_t7::map::MapSources::beside(&path)) {
                Ok(m) => {
                    println!("{}: loaded in {:.1}s", path.display(), started.elapsed().as_secs_f32());
                    for l in &m.report {
                        println!("  {l}");
                    }
                    if dump == "ents" {
                        println!("{}", m.entities);
                    }
                }
                Err(e) => println!("{}: {e}", path.display()),
            }
            continue;
        }
        let z = match fastfile_t7::Zone::open(&path) {
            Ok(z) => z,
            Err(e) => {
                println!("{}: {e}", path.display());
                continue;
            }
        };
        let w = walk_with(&z, Options::default());
        let cap = &w.capture;
        let done = matches!(w.stop, Stop::Done { .. });
        println!(
            "{}: walk {}; {} materials, {} images, {} models, {} meshes",
            path.display(),
            if done { "done".to_owned() } else { format!("stopped: {:?}", w.stop) },
            cap.materials.len(),
            cap.images.len(),
            cap.models.len(),
            cap.meshes.len()
        );
        for g in &cap.world.gfx_worlds {
            println!(
                "  gfxworld {}: {} vertices, positions {:x?}, attributes {:x?}, indices {:x?}, surfaces {:x?}, smodels {:x?}, brush models {:x?}, sky {:?}",
                g.name, g.vertex_count, g.positions, g.attributes, g.indices, g.surfaces, g.smodels, g.brush_models, g.sky_models
            );
        }
        for c in &cap.world.clip_maps {
            println!(
                "  clipmap {}: planes {:x?}, {} materials, brushes {:x?}, verts {:x?}, tris {:x?}, partitions {:x?}, trees {:x?}, cmodels {:x?}, {} static models, dyn defs {:x?}",
                c.name, c.planes, c.materials.len(), c.brushes, c.verts, c.tri_indices, c.partitions, c.aabb_trees, c.cmodels, c.static_models.len(), c.dyn_ent_defs
            );
        }
        for m in &cap.world.map_ents {
            println!("  map_ents {}: entity string {:x?}, trigger models {:x?}", m.name, m.entity_string, m.trigger_models);
            if dump == "ents"
                && let Some((at, n)) = m.entity_string
            {
                let raw = &z.image[at..at + n];
                let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
                println!("{}", String::from_utf8_lossy(&raw[..end]));
            }
        }
        if dump == "materials" {
            for m in &cap.materials {
                println!("  material {} techset {:?}", m.name, m.techset);
                for (h, i) in &m.textures {
                    println!("    {h:#010x} {i}");
                }
            }
        }
        if dump == "meshes" {
            for m in &cap.meshes {
                println!(
                    "  mesh {} flags {:#x} verts {} skin {} indices {} size {:#x} offsets {:x?} data {:x?} xpak {:x?} surfs {:?}",
                    m.name,
                    m.shared_flags,
                    m.vert_count,
                    m.skin_vert_count,
                    m.index_count,
                    m.data_size,
                    m.offsets,
                    m.data,
                    m.xpak.map(|x| (x.key, x.size)),
                    m.surfaces.iter().map(|s| (s.vert_count, s.tri_count, s.base_vert, s.base_index)).collect::<Vec<_>>()
                );
            }
        }
        if let Ok(name) = std::env::var("T7_MAP_MESH")
            && let Some(m) = cap.mesh(&name)
        {
            let data = match (m.data, m.xpak) {
                (Some((at, n)), _) => Some(z.image[at..at + n].to_vec()),
                (None, Some(x)) => {
                    let dir = path.parent().unwrap_or(std::path::Path::new("."));
                    let (set, _) = fastfile_t7::xpak::XpakSet::open_dir(dir).expect("zone dir");
                    set.read(x.key).ok()
                }
                _ => None,
            };
            if let Some(d) = data {
                let out = format!("{}.mesh.bin", name.replace(['/', '\\'], "_"));
                std::fs::write(&out, &d).expect("write mesh");
                println!("  wrote {out} ({:#x} bytes)", d.len());
            }
        }
    }
}
