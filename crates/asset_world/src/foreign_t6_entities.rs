//! bo2mp: a map from another game (MW2, MW3, CoD4, Black Ops) handed to
//! Black Ops II's scripts. Its entities are kept block for block and in
//! order (the session installs the map's own drawn script models and brush
//! models by their place in this text, and the BO2 scripts' entities are
//! matched back to them by that place), with Black Ops II's names where the
//! games differ:
//!
//! - Infinity Ward's Demolition (`dd`) is Black Ops' `dem`: spawn classes
//!   `mp_dd_spawn_*` become `mp_dem_spawn_*`, the bomb sites' targetname
//!   `dd_bombzone` becomes `bombzone_dem`, and `script_gameobjectname`
//!   gains `dem` / `bombzone_dem` beside `dd` / `dd_bombzone` (and `tdm`
//!   beside Infinity Ward's `war`), so BO2's `_gameobjects::main` keeps them
//!   in their mode and deletes them in every other.
//! - Spawn classes a BO2 game type asks for and the map lacks are added at
//!   the end of the text, copied from the nearest kind the map has (Team
//!   Deathmatch's from free-for-all's, the team starts from Domination's or
//!   Search and Destroy's ...), so no game type finds an empty list and
//!   spawns players at the world origin.

/// The game a foreign map's entities come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignEntityGame {
    /// Infinity Ward: CoD4, MW2, MW3.
    InfinityWard,
    /// Treyarch: Black Ops (its names are Black Ops II's).
    Treyarch,
}

/// The rewritten text and what changed, for the world report.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ForeignEntities {
    pub text: String,
    pub report: Vec<String>,
}

/// Spawn classes Black Ops II's game types read, each with where to copy it
/// from when the map has none (the first source kind the map has wins).
/// Applied in order: a later rule sees what an earlier one added.
const T6_SPAWN_FALLBACKS: &[(&str, &[&str])] = &[
    ("mp_dm_spawn", &["mp_tdm_spawn", "mp_dom_spawn"]),
    ("mp_tdm_spawn", &["mp_dm_spawn", "mp_dom_spawn"]),
    (
        "mp_tdm_spawn_allies_start",
        &[
            "mp_dom_spawn_allies_start",
            "mp_ctf_spawn_allies_start",
            "mp_sd_spawn_attacker",
            "mp_tdm_spawn",
        ],
    ),
    (
        "mp_tdm_spawn_axis_start",
        &[
            "mp_dom_spawn_axis_start",
            "mp_ctf_spawn_axis_start",
            "mp_sd_spawn_defender",
            "mp_tdm_spawn",
        ],
    ),
    ("mp_dom_spawn", &["mp_tdm_spawn"]),
    ("mp_dom_spawn_allies_start", &["mp_tdm_spawn_allies_start"]),
    ("mp_dom_spawn_axis_start", &["mp_tdm_spawn_axis_start"]),
    ("mp_ctf_spawn_allies", &["mp_tdm_spawn"]),
    ("mp_ctf_spawn_axis", &["mp_tdm_spawn"]),
    ("mp_ctf_spawn_allies_start", &["mp_tdm_spawn_allies_start"]),
    ("mp_ctf_spawn_axis_start", &["mp_tdm_spawn_axis_start"]),
    ("mp_sd_spawn_attacker", &["mp_tdm_spawn_allies_start"]),
    ("mp_sd_spawn_defender", &["mp_tdm_spawn_axis_start"]),
    (
        "mp_dem_spawn_attacker_start",
        &["mp_sd_spawn_attacker", "mp_tdm_spawn_allies_start"],
    ),
    (
        "mp_dem_spawn_defender_start",
        &["mp_sd_spawn_defender", "mp_tdm_spawn_axis_start"],
    ),
    ("mp_dem_spawn_attacker", &["mp_tdm_spawn"]),
    ("mp_dem_spawn_defender", &["mp_tdm_spawn"]),
    ("mp_dem_spawn_attacker_a", &["mp_dem_spawn_attacker"]),
    ("mp_dem_spawn_attacker_b", &["mp_dem_spawn_attacker"]),
    ("mp_dem_spawn_defender_a", &["mp_dem_spawn_defender"]),
    ("mp_dem_spawn_defender_b", &["mp_dem_spawn_defender"]),
];

/// Infinity Ward game type names in `script_gameobjectname` and the Black
/// Ops II names added beside them.
const IW_GAMEOBJECT_ALIASES: &[(&str, &[&str])] = &[
    ("dd", &["dem"]),
    ("dd_bombzone", &["dem", "bombzone_dem"]),
    ("war", &["tdm"]),
];

/// The key and value of a `"key" "value"` line.
fn quoted_pair(line: &str) -> Option<(&str, &str)> {
    let mut quotes = line.match_indices('"').map(|(index, _)| index);
    let a = quotes.next()?;
    let b = quotes.next()?;
    let c = quotes.next()?;
    let d = quotes.next()?;
    Some((&line[a + 1..b], &line[c + 1..d]))
}

fn rename_spawn_class(game: ForeignEntityGame, class: &str) -> Option<String> {
    match game {
        ForeignEntityGame::InfinityWard => class
            .strip_prefix("mp_dd_spawn_")
            .map(|suffix| format!("mp_dem_spawn_{suffix}")),
        ForeignEntityGame::Treyarch => None,
    }
}

fn alias_gameobject_names(game: ForeignEntityGame, names: &str) -> Option<String> {
    if game != ForeignEntityGame::InfinityWard {
        return None;
    }
    let mut tokens: Vec<&str> = names.split_whitespace().collect();
    let before = tokens.len();
    for token in names.split_whitespace() {
        if let Some((_, extra)) = IW_GAMEOBJECT_ALIASES
            .iter()
            .find(|(from, _)| from.eq_ignore_ascii_case(token))
        {
            for name in *extra {
                if !tokens.iter().any(|t| t.eq_ignore_ascii_case(name)) {
                    tokens.push(name);
                }
            }
        }
    }
    (tokens.len() != before).then(|| tokens.join(" "))
}

/// One spawn point's place, as the map wrote it.
#[derive(Clone, Debug)]
struct SpawnPlace {
    origin: String,
    angles: Option<String>,
}

/// `text` with Black Ops II's names and the spawn classes its game types
/// need (see the module comment).
pub fn foreign_entities_for_t6_rules(text: &str, game: ForeignEntityGame) -> ForeignEntities {
    let mut out = String::with_capacity(text.len() + text.len() / 8);
    let mut report = Vec::new();
    let mut renamed_spawns = 0usize;
    let mut renamed_zones = 0usize;
    let mut aliased_gameobjects = 0usize;
    // Spawn points by (Black Ops II) class, in map order.
    let mut spawns: Vec<(String, SpawnPlace)> = Vec::new();
    // The open block's lines, rewritten when it closes.
    let mut block: Option<Vec<String>> = None;
    let mut blocks = 0usize;

    let flush = |lines: Vec<String>,
                 out: &mut String,
                 spawns: &mut Vec<(String, SpawnPlace)>,
                 renamed_spawns: &mut usize,
                 renamed_zones: &mut usize,
                 aliased_gameobjects: &mut usize| {
        let mut class: Option<String> = None;
        let mut origin: Option<String> = None;
        let mut angles: Option<String> = None;
        let mut rewritten = Vec::with_capacity(lines.len());
        for line in lines {
            let Some((key, value)) = quoted_pair(&line) else {
                rewritten.push(line);
                continue;
            };
            let key_lc = key.to_ascii_lowercase();
            let new_value = match key_lc.as_str() {
                "classname" => {
                    let renamed = rename_spawn_class(game, value);
                    if renamed.is_some() {
                        *renamed_spawns += 1;
                    }
                    let v = renamed.unwrap_or_else(|| value.to_owned());
                    class = Some(v.clone());
                    (v != value).then_some(v)
                }
                "origin" => {
                    origin = Some(value.to_owned());
                    None
                }
                "angles" => {
                    angles = Some(value.to_owned());
                    None
                }
                "targetname"
                    if game == ForeignEntityGame::InfinityWard
                        && value.eq_ignore_ascii_case("dd_bombzone") =>
                {
                    *renamed_zones += 1;
                    Some("bombzone_dem".to_owned())
                }
                "script_gameobjectname" => {
                    let aliased = alias_gameobject_names(game, value);
                    if aliased.is_some() {
                        *aliased_gameobjects += 1;
                    }
                    aliased
                }
                _ => None,
            };
            match new_value {
                Some(v) => rewritten.push(format!("\"{key}\" \"{v}\"")),
                None => rewritten.push(line),
            }
        }
        if let (Some(class), Some(origin)) = (class, origin)
            && class.starts_with("mp_")
            && class.contains("_spawn")
        {
            spawns.push((class.to_ascii_lowercase(), SpawnPlace { origin, angles }));
        }
        out.push_str("{\n");
        for line in rewritten {
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("}\n");
    };

    for line in text.lines() {
        let trimmed = line.trim();
        match (trimmed, block.as_mut()) {
            ("{", None) => block = Some(Vec::new()),
            ("}", Some(_)) => {
                blocks += 1;
                flush(
                    block.take().unwrap_or_default(),
                    &mut out,
                    &mut spawns,
                    &mut renamed_spawns,
                    &mut renamed_zones,
                    &mut aliased_gameobjects,
                );
            }
            (_, Some(lines)) => lines.push(trimmed.to_owned()),
            // Text outside any block (none in a map's entity string) is
            // kept as it was.
            (_, None) => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    if let Some(lines) = block.take() {
        // An unclosed last block: closed, as the game's parser would.
        blocks += 1;
        flush(
            lines,
            &mut out,
            &mut spawns,
            &mut renamed_spawns,
            &mut renamed_zones,
            &mut aliased_gameobjects,
        );
    }

    // Spawn classes the map lacks, after its own blocks (their places in
    // the text, which the session's models are matched by, stay).
    let mut added: Vec<String> = Vec::new();
    for (want, sources) in T6_SPAWN_FALLBACKS {
        if spawns.iter().any(|(class, _)| class == want) {
            continue;
        }
        let Some((from, places)) = sources.iter().find_map(|source| {
            let places: Vec<SpawnPlace> = spawns
                .iter()
                .filter(|(class, _)| class == source)
                .map(|(_, place)| place.clone())
                .collect();
            (!places.is_empty()).then_some((*source, places))
        }) else {
            report.push(format!(
                "bo2 rules spawn gap: {want}: the map has none and none to copy from"
            ));
            continue;
        };
        for place in &places {
            out.push_str("{\n");
            out.push_str(&format!("\"classname\" \"{want}\"\n"));
            out.push_str(&format!("\"origin\" \"{}\"\n", place.origin));
            if let Some(angles) = &place.angles {
                out.push_str(&format!("\"angles\" \"{angles}\"\n"));
            }
            out.push_str("}\n");
        }
        added.push(format!("{want}={} from {from}", places.len()));
        spawns.extend(places.into_iter().map(|p| ((*want).to_owned(), p)));
    }

    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (class, _) in &spawns {
        *counts.entry(class.as_str()).or_default() += 1;
    }
    let counts: Vec<String> = counts.iter().map(|(c, n)| format!("{c}={n}")).collect();
    report.push(format!(
        "bo2 rules entities: {blocks} map blocks kept in order; {renamed_spawns} mp_dd_ spawns \
         named mp_dem_, {renamed_zones} dd_bombzone named bombzone_dem, {aliased_gameobjects} \
         script_gameobjectname given BO2 names"
    ));
    report.push(format!("bo2 rules spawns: {}", counts.join(" ")));
    if !added.is_empty() {
        report.push(format!(
            "bo2 rules spawns added (the map has none of these): {}",
            added.join(", ")
        ));
    }
    ForeignEntities { text: out, report }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(text: &str) -> Vec<Vec<(String, String)>> {
        let mut out = Vec::new();
        let mut cur: Option<Vec<(String, String)>> = None;
        for line in text.lines() {
            match line.trim() {
                "{" => cur = Some(Vec::new()),
                "}" => out.extend(cur.take()),
                l => {
                    if let (Some(c), Some((k, v))) = (cur.as_mut(), quoted_pair(l)) {
                        c.push((k.to_owned(), v.to_owned()));
                    }
                }
            }
        }
        out
    }

    fn class_count(text: &str, class: &str) -> usize {
        blocks(text)
            .iter()
            .filter(|b| b.iter().any(|(k, v)| k == "classname" && v == class))
            .count()
    }

    const IW_MAP: &str = "{\n\"classname\" \"worldspawn\"\n}\n\
        {\n\"origin\" \"1 2 3\"\n\"angles\" \"0 90 0\"\n\"classname\" \"mp_dm_spawn\"\n}\n\
        {\n\"origin\" \"4 5 6\"\n\"classname\" \"mp_tdm_spawn_allies_start\"\n}\n\
        {\n\"origin\" \"7 8 9\"\n\"classname\" \"mp_tdm_spawn_axis_start\"\n}\n\
        {\n\"origin\" \"0 0 0\"\n\"classname\" \"mp_dd_spawn_attacker_start\"\n}\n\
        {\n\"classname\" \"trigger_use_touch\"\n\"targetname\" \"dd_bombzone\"\n\"script_gameobjectname\" \"dd_bombzone\"\n}\n\
        {\n\"classname\" \"script_model\"\n\"model\" \"com_bomb_objective\"\n\"script_gameobjectname\" \"war dom\"\n}\n";

    #[test]
    fn keeps_every_block_in_order_and_appends_spawns() {
        let out = foreign_entities_for_t6_rules(IW_MAP, ForeignEntityGame::InfinityWard);
        let before = blocks(IW_MAP);
        let after = blocks(&out.text);
        assert!(after.len() > before.len());
        for (i, b) in before.iter().enumerate() {
            let class = |b: &Vec<(String, String)>| {
                b.iter()
                    .find(|(k, _)| k == "classname")
                    .map(|(_, v)| v.clone())
            };
            let (old, new) = (class(b), class(&after[i]));
            assert!(
                old == new || old.as_deref() == Some("mp_dd_spawn_attacker_start"),
                "block {i}: {old:?} -> {new:?}"
            );
        }
    }

    #[test]
    fn iw_names_become_bo2_names() {
        let out = foreign_entities_for_t6_rules(IW_MAP, ForeignEntityGame::InfinityWard);
        assert_eq!(class_count(&out.text, "mp_dd_spawn_attacker_start"), 0);
        assert_eq!(class_count(&out.text, "mp_dem_spawn_attacker_start"), 1);
        assert!(out.text.contains("\"targetname\" \"bombzone_dem\""));
        assert!(
            out.text
                .contains("\"script_gameobjectname\" \"dd_bombzone dem bombzone_dem\"")
        );
        assert!(
            out.text
                .contains("\"script_gameobjectname\" \"war dom tdm\"")
        );
    }

    #[test]
    fn missing_spawn_classes_are_copied() {
        let out = foreign_entities_for_t6_rules(IW_MAP, ForeignEntityGame::InfinityWard);
        assert_eq!(class_count(&out.text, "mp_tdm_spawn"), 1);
        assert_eq!(class_count(&out.text, "mp_dom_spawn"), 1);
        assert_eq!(class_count(&out.text, "mp_sd_spawn_attacker"), 1);
        assert_eq!(class_count(&out.text, "mp_sd_spawn_defender"), 1);
        assert_eq!(class_count(&out.text, "mp_ctf_spawn_allies_start"), 1);
        // The map's own Demolition start is kept, not copied over.
        assert_eq!(class_count(&out.text, "mp_dem_spawn_attacker_start"), 1);
        assert_eq!(class_count(&out.text, "mp_dem_spawn_defender_start"), 1);
        let tdm = blocks(&out.text)
            .into_iter()
            .find(|b| {
                b.iter()
                    .any(|(k, v)| k == "classname" && v == "mp_tdm_spawn")
            })
            .unwrap();
        assert!(tdm.iter().any(|(k, v)| k == "origin" && v == "1 2 3"));
        assert!(tdm.iter().any(|(k, v)| k == "angles" && v == "0 90 0"));
    }

    #[test]
    fn treyarch_names_are_kept() {
        let text = "{\n\"classname\" \"mp_dem_spawn_attacker\"\n\"origin\" \"1 1 1\"\n}\n\
            {\n\"classname\" \"script_model\"\n\"script_gameobjectname\" \"dem\"\n}\n";
        let out = foreign_entities_for_t6_rules(text, ForeignEntityGame::Treyarch);
        assert!(out.text.contains("\"script_gameobjectname\" \"dem\"\n"));
        assert_eq!(class_count(&out.text, "mp_dem_spawn_attacker"), 1);
        // Nothing else to copy from: every class comes from the one point.
        assert_eq!(class_count(&out.text, "mp_dm_spawn"), 0);
    }
}
