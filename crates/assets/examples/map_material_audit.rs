//! bo2mp: a map's world and prop materials, each with why its colour map
//! would or would not be drawn: `map_material_audit <iw3|t4|t5> <zone.ff>...`
//! prints one JSON object per map (docs/MAP_MATERIAL_AUDIT.md is made from
//! them). Offline, read-only: CoD4 / World at War zones are read with the
//! `iw3` / `t4` crates and checked against the game's `main/*.iwd`; Black
//! Ops zones are loaded by the T5 lane and its catalog's undecoded colour
//! maps listed.
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// CoD4 / World at War surface types; `surfaceTypeBits` bit `n` is type
/// `n + 1` (`default` has no bit).
const SURFACES: &[&str] = &[
    "default", "bark", "brick", "carpet", "cloth", "concrete", "dirt", "flesh", "foliage", "glass", "grass",
    "gravel", "ice", "metal", "mud", "paper", "plaster", "rock", "sand", "snow", "water", "wood", "asphalt",
    "ceramic", "plastic", "rubber", "cushion", "fruit", "paintedmetal", "player", "tallgrass",
];

fn surface_of(bits: u32) -> String {
    if bits == 0 {
        return "none".into();
    }
    (0..32)
        .filter(|b| bits & (1 << b) != 0)
        .map(|b| SURFACES.get(b as usize + 1).map_or_else(|| format!("type{b}"), |s| (*s).to_owned()))
        .collect::<Vec<_>>()
        .join("+")
}

/// A rough kind of surface from the material / colour image names and the
/// technique set, for grouping the remap work.
fn kind_of(material: &str, image: &str, techset: &str, surface: &str) -> &'static str {
    let n = format!("{} {}", material.to_ascii_lowercase(), image.to_ascii_lowercase());
    let has = |w: &str| n.contains(w);
    if techset.contains("sky") || has("sky") {
        "sky"
    } else if has("decal") || has("graffiti") || has("poster") || has("blood") || has("burntstain") {
        "decal"
    } else if has("foliage") || has("leaf") || has("leaves") || has("tree") || has("bush") || has("grass_tuft")
        || has("vine") || has("fern") || has("palm") || has("ivy") || surface.contains("foliage")
    {
        "foliage"
    } else if has("water") || surface == "water" {
        "water"
    } else if has("glass") || has("window") || surface == "glass" {
        "glass"
    } else if has("terrain") || has("ground") || has("dirt") || has("mud") || has("sand") || has("snow")
        || has("rubble") || has("gravel") || surface.contains("dirt") || surface.contains("grass")
        || surface.contains("mud") || surface.contains("sand") || surface.contains("snow") || surface.contains("gravel")
    {
        "terrain"
    } else if has("floor") || has("tile") || has("asphalt") || has("road") || has("cobble") || has("carpet") {
        "floor"
    } else if has("roof") || has("ceiling") {
        "roof/ceiling"
    } else if has("trim") || has("ornate") || has("column") || has("beam") || has("frame") || has("door")
        || has("stair") || has("rail") || has("pipe") || has("sign") || has("edge")
    {
        "trim/detail"
    } else if has("wall") || has("brick") || has("plaster") || has("concrete") || has("facade") || has("stone")
        || has("wood") || has("metal") || has("corrug")
    {
        "wall"
    } else if has("global_") || has("caulk") || has("clip") || has("shadow") || has("hint") || has("portal") {
        "tool/global"
    } else {
        "other"
    }
}

fn read_iw3(game: &str, path: &Path) -> Result<iw3::zone::Zone, String> {
    match game {
        "t4" => std::panic::catch_unwind(|| t4::load_map(path))
            .map_err(|_| "t4 reader panicked".to_owned())?
            .map(|m| m.zone)
            .map_err(|e| format!("{e:#}")),
        _ => iw3::fastfile::load(path)
            .and_then(|d| iw3::zone::Zone::parse(&d, iw3::zone::ParseOptions::default()))
            .map_err(|e| format!("{e:#}")),
    }
}

/// The game's `main` and whether it holds IWDs.
fn main_dir(path: &Path) -> (Option<PathBuf>, usize) {
    let main = asset_transport::game_root_for_zone(path).ok().map(|r| r.join("main"));
    let iwds = main
        .as_ref()
        .and_then(|m| std::fs::read_dir(m).ok())
        .map_or(0, |d| d.flatten().filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("iwd"))).count());
    (main, iwds)
}

/// Where a colour image would come from and whether it can be read.
fn image_status(image: &iw3::zone::Image, main: Option<&Path>, iwds: usize, cache: &mut HashMap<String, String>) -> String {
    if image.name.starts_with(',') || image.name.starts_with('$') {
        return "engine_reference".into();
    }
    if image.load_def.as_ref().is_some_and(|d| !d.data.is_empty()) {
        return "in_zone".into();
    }
    if iwds == 0 {
        return "iwd_absent".into();
    }
    let Some(main) = main else { return "iwd_absent".into() };
    cache
        .entry(image.name.clone())
        .or_insert_with(|| {
            let file = format!("images/{}.iwi", image.name.trim_start_matches(','));
            if asset_transport::read_main_iwd_file(main, &file).is_none() {
                return "not_in_iwds".into();
            }
            match asset_material::decode_ui_image_from_main(main, &image.name) {
                Ok(Some(_)) => "ok".into(),
                Ok(None) => "unsupported_format".into(),
                Err(e) => format!("decode_error: {e}"),
            }
        })
        .clone()
}

fn audit_legacy(game: &str, path: &Path) -> Value {
    let map = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let zone = match read_iw3(game, path) {
        Ok(z) => z,
        Err(e) => return json!({"game": game, "map": map, "status": "broken", "error": e}),
    };
    let Some(world) = zone.gfx_world() else {
        return json!({"game": game, "map": map, "status": "broken",
            "error": format!("no GfxWorld (stopped at {:?})", zone.stats.stopped_at)});
    };
    let dir = path.parent().unwrap_or(Path::new("."));
    // The shared zones the lane consults for referenced materials.
    let commons: Vec<(String, iw3::zone::Zone)> = ["localized_common_mp.ff", "common_mp.ff"]
        .iter()
        .map(|n| dir.join(n))
        .filter(|p| p.is_file())
        .filter_map(|p| Some((p.file_name()?.to_string_lossy().into_owned(), read_iw3(game, &p).ok()?)))
        .collect();
    let mut common_names: HashMap<String, (usize, usize)> = HashMap::new();
    for (zi, (_, z)) in commons.iter().enumerate() {
        for (i, a) in z.assets.iter().enumerate() {
            if let iw3::zone::Asset::Material(m) = a
                && m.technique_set.is_some()
            {
                common_names.entry(m.name.trim_start_matches(',').to_owned()).or_insert((zi, i));
            }
        }
    }
    let (main, iwds) = main_dir(path);
    let mut cache = HashMap::new();
    let decal = world.decal_surfs.clone();

    // World surfaces: material id -> (tris, surfaces, decal?)
    let mut used: BTreeMap<usize, (u64, u32, bool)> = BTreeMap::new();
    let mut no_material = 0u32;
    for (si, s) in world.surfaces.iter().enumerate() {
        match s.material {
            Some(m) => {
                let e = used.entry(m).or_default();
                e.0 += u64::from(s.tri_count);
                e.1 += 1;
                e.2 |= decal.contains(&(si as u32));
            }
            None => no_material += 1,
        }
    }
    let mut rows = Vec::new();
    let mut walk = |mid: usize, origin: &str, tris: u64, surfs: u32, is_decal: bool, rows: &mut Vec<Value>| {
        let Some(material) = zone.material(mid) else {
            rows.push(json!({"material": format!("#{mid}"), "origin": origin, "status": "unresolved_material", "tris": tris}));
            return;
        };
        let reference = material.name.starts_with(',') || material.technique_set.is_none();
        let (mzone, mat, via) = if reference {
            match common_names.get(material.name.trim_start_matches(',')) {
                Some(&(zi, i)) => match commons[zi].1.material(i) {
                    Some(m) => (&commons[zi].1, m, Some(commons[zi].0.clone())),
                    None => (&zone, material, None),
                },
                None => (&zone, material, None),
            }
        } else {
            (&zone, material, None)
        };
        let techset = mat.technique_set.and_then(|t| mzone.technique_set(t)).map_or("", |t| t.name.as_str()).to_owned();
        let colours: Vec<&iw3::zone::Image> = mat
            .textures
            .iter()
            .filter(|t| t.semantic == iw3::zone::TextureSemantic::Color)
            .filter_map(|t| t.image.and_then(|i| mzone.image(i)))
            .collect();
        let normals = mat.textures.iter().filter(|t| t.semantic == iw3::zone::TextureSemantic::Normal).count();
        let surface = surface_of(mat.surface_type_bits);
        let first = colours.first();
        let image_name = first.map_or(String::new(), |i| i.name.clone());
        let status = if reference && via.is_none() {
            "reference_unresolved".to_owned()
        } else if let Some(img) = first {
            image_status(img, main.as_deref(), iwds, &mut cache)
        } else {
            "no_colour_map".to_owned()
        };
        // Layers the fallback pass drops: only the first colour map is drawn.
        let layers = colours.len();
        let mut kind = kind_of(&material.name, &image_name, &techset, &surface);
        if is_decal && kind != "sky" {
            kind = "decal";
        }
        rows.push(json!({
            "material": material.name, "origin": origin, "via": via, "techset": techset,
            "colour": image_name, "colour_layers": layers, "normal_maps": normals,
            "surface": surface, "kind": kind, "status": status, "tris": tris, "surfaces": surfs,
        }));
    };
    for (&mid, &(tris, surfs, d)) in &used {
        walk(mid, "world", tris, surfs, d, &mut rows);
    }
    // Placed props' materials.
    let mut prop_mats: BTreeMap<usize, u32> = BTreeMap::new();
    for sm in &world.static_models {
        if let Some(m) = sm.model.and_then(|id| zone.xmodel(id)) {
            for mid in m.materials.iter().flatten() {
                *prop_mats.entry(*mid).or_default() += 1;
            }
        }
    }
    for (&mid, &count) in &prop_mats {
        if !used.contains_key(&mid) {
            walk(mid, "prop", 0, count, false, &mut rows);
        }
    }
    let sky = world.sky_image.and_then(|i| zone.image(i)).map(|i| i.name.clone());
    json!({
        "game": game, "map": map, "zone": path.display().to_string(),
        "zone_assets": [zone.stats.top_level_parsed, zone.stats.top_level_total],
        "stopped_at": zone.stats.stopped_at,
        "iwds_in_main": iwds, "main": main.map(|m| m.display().to_string()),
        "commons": commons.iter().map(|c| c.0.clone()).collect::<Vec<_>>(),
        "world_surfaces": world.surfaces.len(), "surfaces_without_material": no_material,
        "sky_image": sky, "rows": rows,
    })
}

fn audit_t5(path: &Path) -> Value {
    let map = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let image = match asset_transport::open_zone(path) {
        Ok(i) => i,
        Err(e) => return json!({"game": "t5", "map": map, "status": "broken", "error": format!("zone does not open: {e}")}),
    };
    let progress = asset_transport::progress::LoadProgress::new(0);
    let mut visions = BTreeMap::new();
    let loaded = assets::lane(image.game).load_world(path, &image, &progress, Default::default(), Default::default(), &mut visions);
    let cat = &loaded.materials;
    let (main, iwds) = main_dir(path);
    let mut rows = Vec::new();
    for m in cat.materials.iter() {
        let colour = m.textures.iter().find(|t| t.semantic == asset_material::TS_COLOR_MAP);
        let img = colour.and_then(|t| t.image).and_then(|i| cat.images.get(i));
        let decoded = img.is_some_and(|i| i.decoded.is_some() || i.pending_decode.is_some() || !i.payload.is_empty());
        let status = match (colour, img) {
            (None, _) => "no_colour_map",
            (Some(_), None) => "unresolved_image",
            (Some(_), Some(_)) if decoded => "ok",
            (Some(_), Some(i)) if i.name.is_reference() => "engine_reference",
            _ if iwds == 0 => "iwd_absent",
            _ => "not_in_iwds_or_unsupported",
        };
        let name = m.name.as_str().to_owned();
        let image_name = img.map_or(String::new(), |i| i.name.as_str().to_owned());
        let techset = m.technique_set.as_str().to_owned();
        rows.push(json!({
            "material": name, "origin": "zone", "techset": techset, "colour": image_name,
            "kind": kind_of(&name, &image_name, &techset, ""), "status": status,
            "routed": m.route.is_some(),
        }));
    }
    let gaps: Vec<String> = loaded.gaps.iter().map(|g| g.reason.clone()).collect();
    let report: Vec<String> = loaded
        .report
        .iter()
        .filter(|l| l.contains("IWD") || l.contains("material") || l.contains("lightmap") || l.contains("world"))
        .cloned()
        .collect();
    json!({
        "game": "t5", "map": map, "zone": path.display().to_string(),
        "iwds_in_main": iwds, "main": main.map(|m| m.display().to_string()),
        "world_drawn": loaded.world.draw.as_ref().map_or(0, |d| d.batches.len()),
        "gaps": gaps, "report": report, "rows": rows,
    })
}

fn main() {
    let mut args = std::env::args().skip(1);
    let game = args.next().expect("game prefix: iw3|t4|t5");
    for path in args {
        let path = Path::new(&path);
        let started = std::time::Instant::now();
        let mut v = match game.as_str() {
            "t5" => audit_t5(path),
            g => audit_legacy(g, path),
        };
        v["seconds"] = json!(started.elapsed().as_secs_f32());
        println!("{v}");
    }
}
