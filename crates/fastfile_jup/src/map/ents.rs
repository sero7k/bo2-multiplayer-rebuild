//! A map's entities and the spawn points a Black Ops II match needs.
//!
//! The zone's `map_ents` is compiled (keys by hash, values pooled); not
//! read yet. A plain-text copy of a map's entities (`{ "key" "value" }`
//! blocks, keys the zone has only by hash written `#xS<hash>`) is read when
//! one is beside the install (`<install>/jupiter/Assets/MapEnts/SpawnVars/
//! <map>.d3dbsp`, where the owner's tools leave them).
//!
//! Modern Warfare III keeps its multiplayer spawn points out of the
//! entities, so they are made here: spread over the floor the map's AI
//! path nodes stand on (else over the world's walkable triangles), as
//! Infinity Ward's class names (`mp_tdm_spawn`, the team starts, `mp_dm_spawn`,
//! `mp_global_intermission`).

use super::MapSources;

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

/// The path nodes' origins (where the map's AI walks: floor).
pub fn path_nodes(text: &str) -> Vec<[f32; 3]> {
    let mut v = origins_of(text, "node_pathnode");
    v.extend(origins_of(text, "node_3d_pathnode"));
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
