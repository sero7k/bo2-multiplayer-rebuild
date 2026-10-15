//! A map's entities and the spawn points a Black Ops II match needs.
//!
//! The zone's `map_ents` is compiled ([`from_zone`]): after its header
//! (0xB70 bytes; +0 the map's name hash, +8 u32 counts: pairs, keys,
//! GUIDs, vectors, string bytes; +0x20 five "data follows" pointers) come
//! - the pairs, a u32 each: value (low 20 bits), key (8 bits, an index into
//!   the keys), kind (top 4 bits: 0 the string at that byte of the strings,
//!   2 the vector of that index, 9 the GUID of that index, 1 other);
//! - the keys, a u64 hash each (not the asset name hash; the names are
//!   [`KEY_NAMES`], read off Tokyo's plain-text entities);
//! - the GUIDs (16 bytes), the vectors (three f32), the strings (NUL-ended).
//!
//! An entity's pairs run together, its GUID (key hash `1444C67E4377FD04`)
//! first; worldspawn comes first of all. A plain-text copy of a map's
//! entities (`{ "key" "value" }` blocks, keys known only by hash written
//! `#xS<hash>`) is read when the zone's are not found
//! (`<install>/jupiter/Assets/MapEnts/SpawnVars/<map>.d3dbsp`, where the
//! owner's tools leave them).
//!
//! Modern Warfare III keeps its multiplayer spawn points out of the
//! entities, so they are made here: spread over the floor the map's AI
//! path nodes stand on (else over the world's walkable triangles), as
//! Infinity Ward's class names (`mp_tdm_spawn`, the team starts, `mp_dm_spawn`,
//! `mp_global_intermission`).

use super::MapSources;
use super::index::{f32_at, find_all, u32_at, u64_at};

/// The entity keys by their hash in `map_ents` (the ones Tokyo's
/// plain-text entities name; Rebirth Island's are the same hashes).
pub const KEY_NAMES: &[(u64, &str)] = &[
    (0x1444_c67e_4377_fd04, "guid"),
    (0x6206_1b8c_b866_6fc0, "classname"),
    (0x27cb_f5ba_dd0c_bdbd, "angles"),
    (0x3aa4_8489_eee7_efa6, "model"),
    (0xcefc_229f_f6de_3343, "origin"),
    (0x27df_edf9_25c2_90bf, "targetname"),
    (0xd876_8d83_64cb_841d, "script_gameobjectname"),
    (0xaac7_7f38_83fc_281f, "spawnflags"),
    (0xb7e5_0a5d_2194_35c4, "target"),
    (0x533e_de30_d361_2dc2, "script_exploder"),
    (0x7da9_7318_b2ce_af24, "script_noteworthy"),
    (0x2c49_b5a3_ad25_3abb, "script_label"),
    (0x0456_a256_fe62_44d4, "script_linkname"),
    (0x4c97_2e80_d661_f936, "height"),
    (0x6d39_b1d2_8346_ef7f, "nophysics"),
    (0xe59e_fdfa_a456_71af, "radius"),
    (0x8091_e2e6_68d4_b5d7, "animscript"),
    (0xb747_e098_ccc6_b2c4, "modelscale"),
    (0x79cc_bf88_b887_3e37, "script_goalyaw"),
    (0xdc32_a624_7e07_bd62, "script_linkto"),
    (0x1685_4c36_6566_abde, "script_bombmode_original"),
    (0x6dc9_d4b8_4b4c_2f7a, "script_team"),
    (0xdfc3_54f9_a229_3cda, "lodcullscale"),
    (0x514e_5f7f_4670_b33e, "prop_model"),
    (0x9f92_ef46_c7d1_83c2, "state"),
    (0x204a_539b_ebe7_6ba4, "mapentlookup"),
    (0x2a3a_1c33_73a5_0db0, "fogenabled"),
    (0x7c3c_01a2_1122_50c7, "index"),
    (0xfc00_43e9_5242_d5cb, "visionset"),
    (0xd3d6_e303_d72b_e8d3, "dmg"),
    (0x5319_e500_f2f2_4095, "screenshot"),
];

/// An entity: its keys and values, in the zone's order.
pub type Entity = Vec<(String, String)>;

const FOLLOW: u64 = 0xFFFF_FFFF_FFFF_FFFE;
const HEADER: usize = 0xB70;

/// Do the counts at `h` lay the arrays out to a string pool (worldspawn's
/// class among its first strings, a NUL at its end)?
fn fits(z: &[u8], h: usize, c: [usize; 5]) -> bool {
    let strings = h + HEADER + 4 * c[0] + 8 * c[1] + 16 * c[2] + 12 * c[3];
    strings + c[4] <= z.len()
        && z[strings + c[4] - 1] == 0
        && z[strings..strings + c[4].min(64)]
            .windows(11)
            .any(|w| w == b"worldspawn\0")
}

/// The zone's own entities (its `map_ents`, found by the map's name
/// hash); keys without a known name and GUIDs are left out.
pub fn from_zone(z: &[u8], name_hash: u64) -> Result<Vec<Entity>, String> {
    let counts =
        |h: usize| -> [usize; 5] { std::array::from_fn(|k| u32_at(z, h + 8 + 4 * k) as usize) };
    let at = find_all(z, &name_hash.to_le_bytes())
        .into_iter()
        .find(|&h| {
            let c = counts(h);
            (0..5).all(|k| u64_at(z, h + 0x20 + 8 * k) == FOLLOW)
                && (1..256).contains(&c[1])
                && (1..1 << 22).contains(&c[0])
                && c[4] > 0
                && fits(z, h, c)
        })
        .ok_or("no map_ents of the map's name in the zone")?;
    let c = counts(at);
    let pairs = at + HEADER;
    let keys = pairs + 4 * c[0];
    let vectors = keys + 8 * c[1] + 16 * c[2];
    let strings = vectors + 12 * c[3];
    let pool = &z[strings..strings + c[4]];
    let names: Vec<Option<&str>> = (0..c[1])
        .map(|k| {
            let h = u64_at(z, keys + 8 * k);
            KEY_NAMES.iter().find(|(x, _)| *x == h).map(|(_, n)| *n)
        })
        .collect();
    let mut out: Vec<Entity> = Vec::new();
    let mut cur: Entity = Vec::new();
    let mut has_guid = false;
    for k in 0..c[0] {
        let e = u32_at(z, pairs + 4 * k);
        let (value, key, kind) = (
            (e & 0xF_FFFF) as usize,
            ((e >> 20) & 0xFF) as usize,
            e >> 28,
        );
        let Some(Some(name)) = names.get(key) else {
            continue;
        };
        if *name == "guid" {
            if has_guid {
                out.push(std::mem::take(&mut cur));
            }
            has_guid = true;
            continue;
        }
        let text = match kind {
            0 => {
                let Some(s) = pool.get(value..) else { continue };
                let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
                String::from_utf8_lossy(&s[..end]).into_owned()
            }
            2 if value < c[3] => {
                let v = [0, 4, 8].map(|o| f32_at(z, vectors + 12 * value + o));
                format!("{} {} {}", v[0], v[1], v[2])
            }
            _ => continue,
        };
        cur.push(((*name).to_owned(), text));
    }
    out.push(cur);
    Ok(out)
}

/// Is the entity one a match gets? Not what needs the game's own brush
/// models (`?N`), volumes, lights, cameras, vehicles' structs or models
/// that are not loaded.
fn kept(e: &Entity) -> bool {
    let get = |k: &str| e.iter().find(|(x, _)| x == k).map(|(_, v)| v.as_str());
    let Some(class) = get("classname") else {
        return false;
    };
    if get("model").is_some_and(|m| m.starts_with('?')) {
        return false;
    }
    matches!(
        class,
        "worldspawn"
            | "script_struct"
            | "script_origin"
            | "info_player_start"
            | "mp_global_intermission"
    ) || class.starts_with("node_")
}

/// The entities a match gets, as Radiant's text (`{ "key" "value" }`).
pub fn to_text(ents: &[Entity]) -> String {
    let mut out = String::new();
    for e in ents.iter().filter(|e| kept(e)) {
        out.push_str("{\n");
        for (k, v) in e {
            out.push_str(&format!("\"{k}\" \"{v}\"\n"));
        }
        out.push_str("}\n");
    }
    out
}

/// The entity text, or why there is none.
pub fn read(src: &MapSources) -> Result<String, String> {
    let path = src
        .install
        .join("jupiter")
        .join("Assets")
        .join("MapEnts")
        .join("SpawnVars")
        .join(format!("{}.d3dbsp", src.name));
    let bytes = std::fs::read(&path).map_err(|_| {
        "no entity text beside the install (the zone's own are not read yet)".to_owned()
    })?;
    let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
    // Keys known only by hash are no keys any reader here knows; the
    // others get their quotes (`key "value"` -> `"key" "value"`).
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("#x") && !l.is_empty())
        .map(|l| match l.split_once(' ') {
            Some((k, v)) if !k.starts_with('"') => format!("\"{k}\" {v}\n"),
            _ => format!("{l}\n"),
        })
        .collect())
}

/// Every entity's `origin`, by class name.
fn origins_of(text: &str, class: &str) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    let mut origin = None;
    let mut is = false;
    for line in text.lines() {
        let l = line.trim();
        if l == "{" {
            origin = None;
            is = false;
        } else if l == "}" {
            if is && let Some(o) = origin.take() {
                out.push(o);
            }
        } else if let Some(v) = l.strip_prefix("\"classname\" ") {
            is = v.trim_matches('"') == class;
        } else if let Some(v) = l.strip_prefix("\"origin\" ") {
            let p: Vec<f32> = v
                .trim_matches('"')
                .split_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect();
            if p.len() == 3 {
                origin = Some([p[0], p[1], p[2]]);
            }
        }
    }
    out
}

/// `n` points of `pts` spread apart (farthest-point order from the one
/// nearest the middle).
fn spread(pts: &[[f32; 3]], n: usize) -> Vec<[f32; 3]> {
    if pts.is_empty() {
        return Vec::new();
    }
    let mid: [f32; 3] =
        std::array::from_fn(|k| pts.iter().map(|p| p[k]).sum::<f32>() / pts.len() as f32);
    let d2 = |a: &[f32; 3], b: &[f32; 3]| {
        (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + ((a[2] - b[2]) * 2.0).powi(2)
    };
    let first = (0..pts.len())
        .min_by(|&a, &b| d2(&pts[a], &mid).total_cmp(&d2(&pts[b], &mid)))
        .unwrap_or(0);
    let mut picked = vec![first];
    let mut near: Vec<f32> = pts.iter().map(|p| d2(p, &pts[first])).collect();
    while picked.len() < n.min(pts.len()) {
        let (i, _) = near
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap();
        picked.push(i);
        for (k, p) in pts.iter().enumerate() {
            near[k] = near[k].min(d2(p, &pts[i]));
        }
    }
    picked.into_iter().map(|i| pts[i]).collect()
}

/// The yaw from `from` toward `to`, in degrees.
fn yaw_to(from: [f32; 3], to: [f32; 3]) -> f32 {
    (to[1] - from[1]).atan2(to[0] - from[0]).to_degrees()
}

/// Spawn point entities for the floor points given (path nodes, or the
/// world's floor): 24 team spawns facing the middle, four starts a side at
/// the two ends farthest apart, 16 free-for-all spawns, the intermission
/// camera above the middle.
pub fn spawn_entities(floor: &[[f32; 3]]) -> String {
    if floor.is_empty() {
        return String::new();
    }
    let mid: [f32; 3] =
        std::array::from_fn(|k| floor.iter().map(|p| p[k]).sum::<f32>() / floor.len() as f32);
    let mut out = String::new();
    let mut put = |class: &str, o: [f32; 3], toward: [f32; 3]| {
        out.push_str(&format!(
            "{{\n\"classname\" \"{class}\"\n\"origin\" \"{} {} {}\"\n\"angles\" \"0 {} 0\"\n}}\n",
            o[0],
            o[1],
            o[2] + 1.0,
            yaw_to(o, toward).round()
        ));
    };
    let team = spread(floor, 24);
    for p in &team {
        put("mp_tdm_spawn", *p, mid);
    }
    for p in spread(floor, 16) {
        put("mp_dm_spawn", p, mid);
    }
    // The starts: the two team spawns farthest apart, each with its three
    // nearest floor points.
    let (mut a, mut b) = (0, 0);
    let mut far = -1.0f32;
    for i in 0..team.len() {
        for j in i + 1..team.len() {
            let d = (0..3)
                .map(|k| (team[i][k] - team[j][k]).powi(2))
                .sum::<f32>();
            if d > far {
                (far, a, b) = (d, i, j);
            }
        }
    }
    for (class, at) in [
        ("mp_tdm_spawn_allies_start", team[a]),
        ("mp_tdm_spawn_axis_start", team[b]),
    ] {
        let mut near: Vec<[f32; 3]> = floor.to_vec();
        near.sort_by(|p, q| {
            let d = |x: &[f32; 3]| (0..3).map(|k| (x[k] - at[k]).powi(2)).sum::<f32>();
            d(p).total_cmp(&d(q))
        });
        // Points at least 48 units apart.
        let mut chosen: Vec<[f32; 3]> = Vec::new();
        for p in near {
            if chosen
                .iter()
                .all(|c| (0..3).map(|k| (c[k] - p[k]).powi(2)).sum::<f32>() > 48.0 * 48.0)
            {
                chosen.push(p);
            }
            if chosen.len() == 4 {
                break;
            }
        }
        for p in chosen {
            put(class, p, mid);
        }
    }
    let hi = floor.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
    put(
        "mp_global_intermission",
        [mid[0], mid[1], hi + 600.0],
        [mid[0] + 1.0, mid[1], mid[2]],
    );
    out
}

/// The path nodes' origins (where the map's AI walks or takes cover:
/// floor).
pub fn path_nodes(text: &str) -> Vec<[f32; 3]> {
    let mut v = Vec::new();
    for class in [
        "node_pathnode",
        "node_3d_pathnode",
        "node_exposed",
        "node_cover_left",
        "node_cover_right",
        "node_cover_crouch",
        "node_cover_stand",
    ] {
        v.extend(origins_of(text, class));
    }
    v
}

#[cfg(test)]
mod tests {
    #[test]
    fn spawns_from_nodes() {
        let text = "{\n\"classname\" \"node_pathnode\"\n\"origin\" \"0 0 0\"\n}\n{\n\"classname\" \"node_pathnode\"\n\"origin\" \"1000 0 0\"\n}\n";
        let nodes = super::path_nodes(text);
        assert_eq!(nodes.len(), 2);
        let s = super::spawn_entities(&nodes);
        assert!(s.contains("mp_tdm_spawn_allies_start") && s.contains("mp_global_intermission"));
    }
}
