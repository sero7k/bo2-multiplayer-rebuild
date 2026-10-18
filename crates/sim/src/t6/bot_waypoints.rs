//! bo2mp: Bot Warfare's waypoints (INeedGames' Bot Warfare, see NOTICE) for
//! the multiplayer maps of Call of Duty 4, World at War, Modern Warfare 2
//! and 3, as Black Ops II path nodes: a map from another game carries no
//! path nodes of its own, and these hand-placed ones route bots far better
//! than the walk graph the engine bakes.
//!
//! `assets/bot_waypoints/<game>.wpz`: zlib text, `#<map>` then one line a
//! waypoint, `x y z|children|type|pitch yaw roll|x y z` (the last two
//! optional: where a camping or explosive spot looks, what a javelin or
//! tube spot fires at).

use super::T6PathNode;
use super::nav::NODE_PATH;

const PACKS: &[(&str, &[u8])] = &[
    ("iw3", include_bytes!("../../../../assets/bot_waypoints/iw3.wpz")),
    ("iw4", include_bytes!("../../../../assets/bot_waypoints/iw4.wpz")),
    ("iw5", include_bytes!("../../../../assets/bot_waypoints/iw5.wpz")),
    ("t4", include_bytes!("../../../../assets/bot_waypoints/t4.wpz")),
];

/// Black Ops II's node kinds the waypoint types become.
const NODE_COVER_CROUCH: u32 = 3;
const NODE_COVER_PRONE: u32 = 5;

/// Call of Duty 4 maps whose Modern Warfare 2 rebuild has another zone name
/// (the geometry is the same): (MW2 zone, CoD4 zone).
const COD4_NAMES: &[(&str, &str)] = &[("mp_bog_sh", "mp_bog"), ("mp_cross_fire", "mp_crossfire")];

fn pack_text(game: &str) -> Option<String> {
    let (_, bytes) = PACKS.iter().find(|(g, _)| *g == game)?;
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib(bytes).ok()?;
    String::from_utf8(raw).ok()
}

/// The waypoint lines of `stem` in `game`'s pack.
fn map_lines(game: &str, stem: &str) -> Option<Vec<String>> {
    let text = pack_text(game)?;
    let header = format!("#{stem}");
    let mut lines = text.lines().skip_while(|l| *l != header);
    lines.next()?;
    let out: Vec<String> = lines.take_while(|l| !l.starts_with('#')).map(str::to_owned).collect();
    (!out.is_empty()).then_some(out)
}

fn vec3(s: &str) -> Option<[f32; 3]> {
    let mut it = s.split_whitespace().map(|v| v.parse::<f32>().ok());
    Some([it.next()??, it.next()??, it.next()??])
}

/// The path nodes for a map (`iw4:mp_rust`), when Bot Warfare has its
/// waypoints: its own game's, else (a Call of Duty 4 map rebuilt in Modern
/// Warfare 2) Call of Duty 4's.
pub(crate) fn path_nodes(map: &str) -> Option<(Vec<T6PathNode>, &'static str)> {
    let (game, stem) = map.split_once(':')?;
    let stem = stem.to_ascii_lowercase();
    let mut tries: Vec<(&'static str, String)> = Vec::new();
    if let Some((g, _)) = PACKS.iter().find(|(g, _)| *g == game) {
        tries.push((g, stem.clone()));
    }
    if game == "iw4" {
        let cod4 = COD4_NAMES.iter().find(|(m, _)| *m == stem).map_or(stem.as_str(), |(_, c)| c);
        tries.push(("iw3", cod4.to_owned()));
    }
    tries.into_iter().find_map(|(g, s)| map_lines(g, &s).map(|l| (nodes(&l), g)))
}

fn nodes(lines: &[String]) -> Vec<T6PathNode> {
    let mut out: Vec<T6PathNode> = lines
        .iter()
        .map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            let field = |i: usize| f.get(i).copied().unwrap_or("").trim();
            let ty = field(2).to_ascii_lowercase();
            let angles = vec3(field(3));
            let (kind, note) = match ty.as_str() {
                "crouch" => (NODE_COVER_CROUCH, "camp"),
                "prone" => (NODE_COVER_PRONE, "camp"),
                "claymore" => (NODE_PATH, "claymore"),
                "grenade" => (NODE_PATH, "grenade"),
                "javelin" => (NODE_PATH, "javelin"),
                "tube" => (NODE_PATH, "tube"),
                "climb" => (NODE_PATH, "climb"),
                "jump" | "mantle" => (NODE_PATH, "jump"),
                _ => (NODE_PATH, ""),
            };
            T6PathNode {
                ty: kind,
                origin: vec3(field(0)).unwrap_or([0.0; 3]),
                angle: angles.map_or(0.0, |a| a[1]),
                script_noteworthy: note.to_owned(),
                // (What a javelin or tube spot fires at.)
                script_linkname: field(4).to_owned(),
                links: field(1)
                    .split_whitespace()
                    .filter_map(|c| c.parse::<u16>().ok())
                    .map(|c| (c, 0.0, false))
                    .collect(),
                ..T6PathNode::default()
            }
        })
        .collect();
    let n = out.len();
    let origins: Vec<[f32; 3]> = out.iter().map(|o| o.origin).collect();
    for node in &mut out {
        node.links.retain(|l| usize::from(l.0) < n);
        for link in &mut node.links {
            let (a, b) = (node.origin, origins[usize::from(link.0)]);
            link.1 = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        }
    }
    // (A link one way only is walked both ways, as Bot Warfare walks it.)
    let pairs: Vec<(usize, u16, f32)> =
        out.iter().enumerate().flat_map(|(i, nd)| nd.links.iter().map(move |l| (i, l.0, l.1))).collect();
    for (i, j, d) in pairs {
        let back = &mut out[usize::from(j)].links;
        if !back.iter().any(|l| usize::from(l.0) == i) {
            back.push((i as u16, d, false));
        }
    }
    out
}
