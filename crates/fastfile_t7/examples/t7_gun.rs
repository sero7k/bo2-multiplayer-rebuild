//! Everything one Black Ops III gun needs, and where each piece is:
//! `t7_gun <zone.ff> <weapon>` (e.g. `core_common.ff ar_standard_cp`).
//!
//! Walks the zone and prints the gun's manifest: its models -> LOD meshes
//! (vertex data in the zone or an `.xpak` entry) -> materials -> images
//! (pixels in the zone or `.xpak` parts, with the package that holds each),
//! the animation table (each animation loaded in this zone or not), sound
//! aliases, effects, attachments (their uniques' models, animations and
//! icons), cosmetic variants and camos.
//!
//! `T7_XPAK_DIR=<dir>`: where the `.xpak` packages are (default: the
//! zone's directory). `T7_GUN_OUT=<dir>`: also exports the view model's
//! images (`.dds` + `.png` of the top level) and every model's LOD 0 mesh
//! (`.obj`; `T7_GUN_LODS=all`: every LOD).
//! Names starting with "," are placeholders (the asset is in another zone).

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use fastfile_t7::AssetType as T;
use fastfile_t7::mesh;
use fastfile_t7::texture;
use fastfile_t7::walk::capture::{Capture, FieldValue, ImageRec, WeaponRec};
use fastfile_t7::walk::{Stop, walk};
use fastfile_t7::xpak::XpakSet;

struct Ctx<'a> {
    zone: &'a fastfile_t7::Zone,
    cap: &'a Capture,
    paks: XpakSet,
    /// Assets the walk read in this zone: (type, name).
    loaded: HashSet<(T, String)>,
    out: Option<PathBuf>,
    all_lods: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(path), Some(weapon)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: t7_gun <zone.ff> <weapon>");
        std::process::exit(2);
    };
    let zone = fastfile_t7::Zone::open(Path::new(path)).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(1);
    });
    let w = walk(&zone);
    if !matches!(w.stop, Stop::Done { .. }) {
        eprintln!("(walk stopped early: {:?}; records up to there)", w.stop);
    }
    let dir = std::env::var_os("T7_XPAK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(path).parent().unwrap_or(Path::new(".")).to_owned());
    let (paks, bad) = XpakSet::open_dir(&dir).unwrap_or_default();
    for (p, e) in bad {
        eprintln!("{}: {e}", p.display());
    }
    let loaded = w
        .walked
        .iter()
        .filter_map(|a| a.name.clone().map(|n| (a.ty, n)))
        .collect();
    let ctx = Ctx {
        zone: &zone,
        cap: &w.capture,
        paks,
        loaded,
        out: std::env::var_os("T7_GUN_OUT").map(PathBuf::from),
        all_lods: std::env::var("T7_GUN_LODS").is_ok_and(|v| v == "all"),
    };
    let Some(r) = w
        .capture
        .weapons
        .iter()
        .find(|r| r.name == *weapon)
        .or_else(|| w.capture.weapons.iter().find(|r| r.base_name == *weapon))
    else {
        eprintln!("no weapon {weapon} in {path} ({} weapons)", w.capture.weapons.len());
        std::process::exit(1);
    };
    if let Some(o) = &ctx.out {
        std::fs::create_dir_all(o).expect("out dir");
    }
    manifest(&ctx, r);
}

fn asset_fields<'a>(r: &'a WeaponRec, ty: T) -> Vec<(&'a str, &'a str)> {
    r.fields
        .iter()
        .filter_map(|(f, v)| match v {
            FieldValue::Asset { ty: t, name } if *t == ty => Some((f.as_str(), name.as_str())),
            _ => None,
        })
        .collect()
}

fn manifest(c: &Ctx, r: &WeaponRec) {
    println!("{} (WeaponDef \"{}\") \"{}\"", r.name, r.base_name, r.display_name);
    if let Some(o) = &r.def_shared_with {
        println!("  WeaponDef shared with {o}");
    }
    println!("  packages: {} .xpak files", c.paks.paks.len());

    // Models: the weapon's, its attachment uniques' and cosmetic variants'.
    let mut models: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (f, m) in asset_fields(r, T::XModel) {
        models.entry(m.to_string()).or_default().push(f.to_string());
    }
    let uniques = asset_fields(r, T::AttachmentUnique);
    let acvs = asset_fields(r, T::AttachmentCosmeticVariant);
    for (_, au) in &uniques {
        for (f, v) in c.cap.links(T::AttachmentUnique, au) {
            if let FieldValue::Asset { ty: T::XModel, name } = v {
                models.entry(name.clone()).or_default().push(format!("{au}.{f}"));
            }
        }
    }
    for (_, acv) in &acvs {
        for (f, v) in c.cap.links(T::AttachmentCosmeticVariant, acv) {
            if let FieldValue::Asset { ty: T::XModel, name } = v {
                models.entry(name.clone()).or_default().push(format!("{acv}.{f}"));
            }
        }
    }
    println!("\n== models ({})", models.len());
    let mut seen_images = BTreeSet::new();
    let mut totals = Totals::default();
    let view = asset_fields(r, T::XModel).iter().find(|(f, _)| *f == "gunModel").map(|(_, m)| m.to_string());
    for (m, used) in &models {
        println!("  model {m}  <- {}", used.join(", "));
        model(c, m, view.as_deref() == Some(m.as_str()), &mut seen_images, &mut totals);
    }
    println!(
        "  totals: {} images ({} streamed parts in packages, {} missing), {} meshes ({} streamed)",
        totals.images, totals.parts, totals.missing_parts, totals.meshes, totals.streamed_meshes
    );

    // Animations.
    println!("\n== animations ({} of 197 slots)", r.anims.len());
    let mut here = 0;
    for (i, slot, a) in &r.anims {
        let st = if c.loaded.contains(&(T::XAnimParts, a.clone())) {
            here += 1;
            "in zone"
        } else {
            "not in this zone"
        };
        println!("  {i:3} {slot:<28} {a}  [{st}]");
    }
    println!("  {here} of {} loaded in this zone", r.anims.len());

    // Sounds, effects and the rest of the weapon's fields.
    println!("\n== sound aliases and other strings");
    for (f, v) in &r.fields {
        if let FieldValue::Str(s) = v {
            println!("  {f:<36} \"{s}\"");
        }
    }
    println!("\n== effects");
    for ty in [T::Fx, T::TagFx] {
        for (f, n) in asset_fields(r, ty) {
            println!("  {f:<36} {:<6} {n}  [{}]", ty.name(), status(c, ty, n));
        }
    }
    println!("\n== other assets");
    for (f, v) in &r.fields {
        if let FieldValue::Asset { ty, name } = v
            && !matches!(ty, T::XModel | T::Image | T::Fx | T::TagFx | T::AttachmentUnique | T::Attachment | T::AttachmentCosmeticVariant)
        {
            println!("  {f:<36} {:<26} {name}  [{}]", ty.name(), status(c, *ty, name));
        }
    }
    for (f, n) in asset_fields(r, T::Image) {
        println!("  {f:<36} {:<26} {n}  [{}]", "image", status(c, T::Image, n));
    }

    // Attachments.
    println!("\n== attachments");
    for (f, a) in asset_fields(r, T::Attachment) {
        println!("  {f:<24} {a}");
    }
    println!("\n== attachment uniques ({})", uniques.len());
    for (f, au) in &uniques {
        let links = c.cap.links(T::AttachmentUnique, au);
        let anims = links.iter().filter(|(l, _)| l.starts_with("anim[")).count();
        println!("  {f:<24} {au}  ({anims} animation overrides)");
        for (l, v) in links.iter().filter(|(l, _)| !l.starts_with("anim[")) {
            match v {
                FieldValue::Asset { ty, name } => println!("      {l:<20} {:<26} {name}", ty.name()),
                FieldValue::Str(s) => println!("      {l:<20} \"{s}\""),
            }
        }
    }
    println!("\n== cosmetic variants ({})", acvs.len());
    for (f, acv) in &acvs {
        println!("  {f:<10} {acv}");
        for (l, v) in c.cap.links(T::AttachmentCosmeticVariant, acv) {
            match v {
                FieldValue::Asset { ty, name } => println!("      {l:<28} {:<8} {name}", ty.name()),
                FieldValue::Str(s) => println!("      {l:<28} \"{s}\""),
            }
        }
    }
    println!("\n== camos");
    let mut camos: BTreeSet<String> = asset_fields(r, T::WeaponCamo).iter().map(|(_, n)| n.to_string()).collect();
    for (_, au) in &uniques {
        for (_, v) in c.cap.links(T::AttachmentUnique, au) {
            if let FieldValue::Asset { ty: T::WeaponCamo, name } = v {
                camos.insert(name.clone());
            }
        }
    }
    for camo in &camos {
        let links = c.cap.links(T::WeaponCamo, camo);
        println!("  {camo}: {} camo slots read", links.len());
        for (l, v) in links {
            if let FieldValue::Asset { ty, name } = v {
                println!("      {l:<32} {:<8} {name}", ty.name());
            }
        }
    }
}

fn status(c: &Ctx, ty: T, name: &str) -> &'static str {
    if name.starts_with(',') {
        "placeholder: another zone"
    } else if name.starts_with("<ref") {
        "reference the walk cannot name"
    } else if c.loaded.contains(&(ty, name.to_string())) {
        "in zone"
    } else {
        "loaded by an earlier asset"
    }
}

#[derive(Default)]
struct Totals {
    images: usize,
    parts: usize,
    missing_parts: usize,
    meshes: usize,
    streamed_meshes: usize,
}

fn model(c: &Ctx, name: &str, export_images: bool, seen: &mut BTreeSet<String>, t: &mut Totals) {
    if name.starts_with(',') {
        println!("    (placeholder: the model is in another zone)");
        return;
    }
    let Some(m) = c.cap.model(name) else {
        println!("    (not loaded in this zone)");
        return;
    };
    println!("    {} bones, {} LODs", m.bones, m.lods.len());
    for (i, lod) in m.lods.iter().enumerate() {
        t.meshes += 1;
        let Some(me) = c.cap.mesh(&lod.mesh) else {
            println!("    lod{i} mesh {} (not read)", lod.mesh);
            continue;
        };
        let at = match (me.data, me.xpak) {
            (Some((pos, len)), _) => format!("zone {pos:#x} ({len} bytes)"),
            (None, Some(x)) => {
                t.streamed_meshes += 1;
                let hit = c.paks.find_all(x.key);
                let pak = hit.first().map_or("NOT FOUND".to_string(), |(p, _)| pak_name(&p.path));
                format!("xpak {:#018x} in {pak} ({} bytes)", x.key, me.data_size)
            }
            (None, None) => "no data".into(),
        };
        println!(
            "    lod{i} mesh {}: {} verts ({} skinned), {} indices, {} surfaces, {at}",
            lod.mesh,
            me.vert_count,
            me.skin_vert_count,
            me.index_count,
            me.surfaces.len()
        );
        if let Some(o) = &c.out
            && (i == 0 || c.all_lods)
        {
            export_mesh(c, me, &lod.materials, o, &format!("{name}_lod{i}"));
        }
        for mat in &lod.materials {
            material(c, mat, export_images, seen, t);
        }
    }
}

fn pak_name(p: &Path) -> String {
    p.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned())
}

fn material(c: &Ctx, name: &str, export: bool, seen: &mut BTreeSet<String>, t: &mut Totals) {
    let Some(m) = c.cap.material(name).filter(|_| !name.starts_with(',')) else {
        println!("      material {name}{}", if name.starts_with(',') { " (placeholder)" } else { " (not read)" });
        return;
    };
    println!("      material {} (techset {})", m.name, m.techset.as_deref().unwrap_or("-"));
    for (hash, img) in &m.textures {
        let Some(im) = c.cap.image(img).filter(|_| !img.starts_with(',')) else {
            println!("        {hash:08x} {img} (placeholder)");
            continue;
        };
        let first = seen.insert(img.clone());
        if first {
            t.images += 1;
        }
        println!("        {hash:08x} {img} {}", image_line(c, im, first, t));
        if export && first && let Some(o) = &c.out {
            export_image(c, im, o);
        }
    }
}

fn image_line(c: &Ctx, im: &ImageRec, count: bool, t: &mut Totals) -> String {
    let f = format!("{}x{} {}", im.width, im.height, texture::dxgi_name(im.format));
    if im.xpak.is_empty() {
        return format!("{f}, in zone ({} bytes)", im.pixels_len);
    }
    let parts: Vec<String> = im
        .xpak
        .iter()
        .map(|p| {
            let hit = c.paks.find_all(p.key);
            if count {
                t.parts += 1;
                if hit.is_empty() {
                    t.missing_parts += 1;
                }
            }
            let pak = hit.first().map_or("NOT FOUND".to_string(), |(x, _)| pak_name(&x.path));
            format!("{}x{} {:#018x} {pak}", p.width, p.height, p.key)
        })
        .collect();
    format!("{f}, streamed [{}]", parts.join(", "))
}

fn export_image(c: &Ctx, im: &ImageRec, out: &Path) {
    match texture::image_levels(im, &c.zone.image, &c.paks) {
        Ok(l) => {
            let base = out.join(&im.name);
            std::fs::write(base.with_extension("dds"), texture::to_dds(&l)).expect("write dds");
            if let Some((w, h, px)) = texture::level_rgba(&l, 0) {
                write_png(&base.with_extension("png"), w, h, &px);
            }
            println!("          -> {}.dds ({}x{}, {} levels)", im.name, l.width, l.height, l.levels);
        }
        Err(e) => println!("          -> {}: {e}", im.name),
    }
}

fn write_png(path: &Path, w: u32, h: u32, rgba: &[u8]) {
    let f = std::fs::File::create(path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("png header").write_image_data(rgba).expect("png data");
}

fn export_mesh(c: &Ctx, me: &fastfile_t7::walk::capture::MeshRec, materials: &[String], out: &Path, base: &str) {
    let data = match (me.data, me.xpak) {
        (Some((at, len)), _) => c.zone.image.get(at..at + len).map(<[u8]>::to_vec).ok_or_else(|| "zone range".to_string()),
        (None, Some(x)) => c.paks.read(x.key).map_err(|e| e.to_string()),
        _ => Err("no data".into()),
    };
    let data = match data {
        Ok(d) => d,
        Err(e) => {
            println!("      -> {base}: {e}");
            return;
        }
    };
    if std::env::var_os("T7_GUN_RAW").is_some() {
        std::fs::write(out.join(format!("{base}.bin")), &data).expect("write bin");
        println!("      -> {base}.bin offsets {:x?} flags {:#x}", me.offsets, me.shared_flags);
        for sf in &me.surfaces {
            println!(
                "         surface {} flags {:#x} verts {}+{} tris {} from index {} rigid {:?}",
                sf.name, sf.flags, sf.base_vert, sf.vert_count, sf.tri_count, sf.base_index, sf.rigid
            );
        }
    }
    match mesh::decode(me, &data) {
        Ok(g) => {
            let obj = mesh::to_obj(&g, materials);
            std::fs::write(out.join(format!("{base}.obj")), obj).expect("write obj");
            println!(
                "      -> {base}.obj ({} verts, {} tris; {} skinned, {} with 2+ bones, {} bones used)",
                g.positions.len(),
                g.surfaces.iter().map(|s| s.indices.len() / 3).sum::<usize>(),
                me.skin_vert_count,
                g.weights.iter().filter(|w| w.len() > 1).count(),
                g.weights.iter().flatten().map(|w| w.0).collect::<BTreeSet<u16>>().len()
            );
        }
        Err(e) => println!("      -> {base}: {e}"),
    }
}
