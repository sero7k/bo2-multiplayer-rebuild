//! A Black Ops III weapon's record: `t7_weapon <zone.ff> [weapon name]`.
//! Without a name, lists the zone's weapons; with one (`ar_standard` or
//! `ar_standard_mp`), prints every field of the variant and its
//! `WeaponDef` that names an asset or a string, the animation table, and
//! for each model its LOD meshes and materials (each material's technique
//! set and images, with the `.xpak` entries of streamed images).
use fastfile_t7::walk::capture::{Capture, FieldValue, WeaponRec};
use fastfile_t7::walk::{Stop, walk};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("usage: t7_weapon <zone.ff> [weapon name]");
        std::process::exit(2);
    };
    let zone = match fastfile_t7::Zone::open(std::path::Path::new(path)) {
        Ok(z) => z,
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(1);
        }
    };
    let w = walk(&zone);
    if !matches!(w.stop, Stop::Done { .. }) {
        eprintln!("walk stopped early: {:?} (records up to there)", w.stop);
    }
    let cap = &w.capture;
    match args.get(2) {
        None => list(cap),
        Some(name) => {
            let found: Vec<&WeaponRec> = cap
                .weapons
                .iter()
                .filter(|r| {
                    r.name == *name || r.base_name == *name || r.name == format!("{name}_mp")
                })
                .collect();
            if found.is_empty() {
                eprintln!(
                    "no weapon {name} in {path} ({} weapons; run without a name to list them)",
                    cap.weapons.len()
                );
                std::process::exit(1);
            }
            for r in found {
                show(&zone, cap, r);
            }
        }
    }
}

fn list(cap: &Capture) {
    println!("{} weapons:", cap.weapons.len());
    for r in &cap.weapons {
        let shared = r
            .def_shared_with
            .as_ref()
            .map_or(String::new(), |o| format!(" (WeaponDef of {o})"));
        println!(
            "  {:<38} {:<34} \"{}\"{shared}",
            r.name, r.base_name, r.display_name
        );
    }
}

/// Picks the fields a group of the record shows.
type Pick<'a> = &'a dyn Fn(&FieldValue) -> bool;

fn show(zone: &fastfile_t7::Zone, cap: &Capture, r: &WeaponRec) {
    println!(
        "{} (WeaponDef \"{}\") \"{}\"",
        r.name, r.base_name, r.display_name
    );
    let mut models = Vec::new();
    let groups: [(&str, Pick); 5] = [
        (
            "models",
            &|v| matches!(v, FieldValue::Asset { ty, .. } if *ty == fastfile_t7::AssetType::XModel),
        ),
        (
            "images",
            &|v| matches!(v, FieldValue::Asset { ty, .. } if *ty == fastfile_t7::AssetType::Image),
        ),
        (
            "effects",
            &|v| matches!(v, FieldValue::Asset { ty, .. } if matches!(ty, fastfile_t7::AssetType::Fx | fastfile_t7::AssetType::TagFx)),
        ),
        (
            "other assets",
            &|v| matches!(v, FieldValue::Asset { ty, .. } if !matches!(ty, fastfile_t7::AssetType::XModel | fastfile_t7::AssetType::Image | fastfile_t7::AssetType::Fx | fastfile_t7::AssetType::TagFx)),
        ),
        ("strings (sound aliases, names, hints)", &|v| {
            matches!(v, FieldValue::Str(_))
        }),
    ];
    for (title, pick) in groups {
        // The default cosmetic variant fills most of the 44 slots: left out.
        let default_acv = |v: &FieldValue| matches!(v, FieldValue::Asset { name, .. } if name == ",defaultattachmentcosmeticvariant");
        let rows: Vec<_> = r.fields.iter().filter(|(_, v)| pick(v) && !default_acv(v)).collect();
        if rows.is_empty() {
            continue;
        }
        println!("  {title}:");
        for (field, v) in rows {
            match v {
                FieldValue::Asset { ty, name } => {
                    println!("    {field:<34} {:<16} {name}", ty.name());
                    if *ty == fastfile_t7::AssetType::XModel && !models.contains(name) {
                        models.push(name.clone());
                    }
                }
                FieldValue::Str(s) => println!("    {field:<34} \"{s}\""),
            }
        }
    }
    if !r.anims.is_empty() {
        println!("  animations ({} of 197 slots):", r.anims.len());
        for (i, slot, a) in &r.anims {
            println!("    {i:3} {slot:<28} {a}");
        }
    }
    if !r.hide_tags.is_empty() {
        let names: Vec<String> = r
            .hide_tags
            .iter()
            .map(|&i| {
                zone.assets
                    .script_strings
                    .get(i as usize)
                    .cloned()
                    .flatten()
                    .unwrap_or_else(|| format!("#{i}"))
            })
            .collect();
        println!("  hideTags: {}", names.join(" "));
    }
    for m in models {
        print_model(cap, &m, "  ");
    }
}

fn print_model(cap: &Capture, name: &str, indent: &str) {
    if name.starts_with(',') {
        println!("{indent}model {name}: placeholder (the model is loaded by another zone)");
        return;
    }
    let Some(m) = cap.model(name) else {
        println!("{indent}model {name}: not loaded in this zone");
        return;
    };
    println!(
        "{indent}model {} ({} bones, {} LODs)",
        m.name,
        m.bones,
        m.lods.len()
    );
    for (i, lod) in m.lods.iter().enumerate() {
        let detail = cap.mesh(&lod.mesh).map_or(String::new(), |me| {
            let at = match (me.data, me.xpak) {
                (Some((pos, len)), _) => format!("data in zone at {pos:#x} ({len} bytes)"),
                (None, Some(x)) => {
                    format!("data in .xpak key {:#018x} ({} bytes)", x.key, me.data_size)
                }
                (None, None) => "no data".to_string(),
            };
            let surfs: Vec<String> = me
                .surfaces
                .iter()
                .map(|sf| {
                    format!(
                        "{}{}v/{}t",
                        if sf.name.is_empty() {
                            String::new()
                        } else {
                            format!("{} ", sf.name)
                        },
                        sf.vert_count,
                        sf.tri_count
                    )
                })
                .collect();
            format!(
                ": {} verts, {} indices, {at}; surfaces [{}]",
                me.vert_count,
                me.index_count,
                surfs.join(", ")
            )
        });
        println!("{indent}  lod{i} mesh {}{detail}", lod.mesh);
        for mat in &lod.materials {
            print_material(cap, mat, &format!("{indent}    "));
        }
    }
}

fn print_material(cap: &Capture, name: &str, indent: &str) {
    let Some(m) = cap.material(name).filter(|_| !name.starts_with(',')) else {
        println!(
            "{indent}material {name}{}",
            if name.starts_with(',') {
                " (placeholder)"
            } else {
                ""
            }
        );
        return;
    };
    println!(
        "{indent}material {} (techset {})",
        m.name,
        m.techset.as_deref().unwrap_or("-")
    );
    for (hash, img) in &m.textures {
        if img.starts_with(',') {
            println!("{indent}  {hash:08x} {img} (placeholder)");
            continue;
        }
        let detail = cap.image(img).map_or(String::new(), |i| {
            let parts: Vec<String> = i
                .xpak
                .iter()
                .map(|p| format!("{}x{} key {:#018x}", p.width, p.height, p.key))
                .collect();
            if parts.is_empty() {
                format!(
                    " {}x{} fmt {} in zone ({} bytes)",
                    i.width, i.height, i.format, i.pixels_len
                )
            } else {
                format!(
                    " {}x{} fmt {} xpak [{}]",
                    i.width,
                    i.height,
                    i.format,
                    parts.join(", ")
                )
            }
        });
        println!("{indent}  {hash:08x} {img}{detail}");
    }
}
