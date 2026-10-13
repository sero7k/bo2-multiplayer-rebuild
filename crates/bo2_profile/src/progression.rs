//! bo2mp: the combined level and prestige progression. BO2's 55 levels go
//! on to 155 (mp/rankTable.csv's rows carried on along BO2's own XP curve),
//! match XP comes three times as fast, and the prestiges run through the
//! other games' prestige icons in order: Black Ops' 15, then Modern
//! Warfare 2's 10, then Modern Warfare 3's 20, and last BO2's own Prestige
//! Master (its master prestige icon from mp/rankIconTable.csv). Shared by
//! the menus (`hks_t6::mp`), the picture loader (`ui::lui_hud`) and the
//! server's XP (`sim::t6::stats`). Std only.
//!
//! The rank tables stay BO2's: `extend_table` rewrites mp/rankTable.csv
//! and mp/rankIconTable.csv as they load, so BO2's own scripts (the LUI
//! menus' `Engine.TableLookup`, the GSC `_rank.gsc` reading `maxrank` and
//! `maxprestige`) see the longer ladder without a line of theirs changed.
//!
//! A picture from another game goes to the menus as `<game>:<material>`
//! (`t5:rank_prestige01`): no BO2 material has a colon in its name, and BO2
//! has `rank_prestige*` pictures of its own a bare name would find first.
//! `foreign_picture` turns that back into the game and the names to try.

use std::sync::RwLock;

/// BO2's own last level (mp/rankTable.csv's `maxrank` 54, shown as 55).
pub const BO2_MAX_LEVEL: i32 = 55;

/// The last level, shown as such (rank 154, 0 = level 1).
pub const MAX_LEVEL: i32 = 155;

/// The last rank as BO2's tables and stats count it (0 = level 1): the
/// rank table's `maxrank`.
pub const MAX_RANK: i32 = MAX_LEVEL - 1;

/// Every match XP award (an event's mp/scoreInfo.csv XP, the scripts'
/// `addrankxpvalue`) is this many times BO2's.
pub const XP_MULTIPLIER: f32 = 3.0;

/// Black Ops' prestiges (its `rank_prestige01` .. `rank_prestige15`).
pub const BO1_PRESTIGES: i32 = 15;
/// Modern Warfare 2's prestiges (its `rank_prestige1` .. `rank_prestige10`).
pub const MW2_PRESTIGES: i32 = 10;
/// Modern Warfare 3's prestiges (1 .. 20).
pub const MW3_PRESTIGES: i32 = 20;

/// The last prestige: every other game's prestiges, then BO2's Prestige
/// Master (the rank icon table's `maxprestige`).
pub const MAX_PRESTIGE: i32 = BO1_PRESTIGES + MW2_PRESTIGES + MW3_PRESTIGES + 1;

/// The game prefix of BO2's own pictures (`ui::foreign_maps` names the
/// others: `t5` Black Ops, `iw4` Modern Warfare 2, `iw5` Modern Warfare 3).
pub const BO2: &str = "t6";

/// Where a prestige's icon comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrestigeIcon {
    /// No prestige: BO2's icon for the level (mp/rankIconTable.csv
    /// column 1; past BO2's 55 its last level's).
    Bo2Level,
    /// Black Ops' prestige `n` (1 first).
    Bo1(i32),
    /// Modern Warfare 2's prestige `n` (1 first).
    Mw2(i32),
    /// Modern Warfare 3's prestige `n` (1 first).
    Mw3(i32),
    /// The last prestige: BO2's Prestige Master icon.
    Bo2Master,
}

/// A prestige's place in the combined order (clamped to 0 ..=
/// `MAX_PRESTIGE`).
#[must_use]
pub fn prestige_icon(prestige: i32) -> PrestigeIcon {
    let p = prestige.clamp(0, MAX_PRESTIGE);
    if p == 0 {
        PrestigeIcon::Bo2Level
    } else if p <= BO1_PRESTIGES {
        PrestigeIcon::Bo1(p)
    } else if p <= BO1_PRESTIGES + MW2_PRESTIGES {
        PrestigeIcon::Mw2(p - BO1_PRESTIGES)
    } else if p < MAX_PRESTIGE {
        PrestigeIcon::Mw3(p - BO1_PRESTIGES - MW2_PRESTIGES)
    } else {
        PrestigeIcon::Bo2Master
    }
}

/// The material names a game's prestige `n` icon may have, the likeliest
/// first. None of these were read off the games' own rank icon tables
/// (the zones are not here), so the picture loader tries each in turn and
/// logs the one it found:
/// - Black Ops: `rank_prestige01` .. `rank_prestige15` (two digits, as its
///   `rank_*` set), else the one-digit form World at War used.
/// - Modern Warfare 2: `rank_prestige1` .. `rank_prestige10` (its
///   rankIconTable's prestige columns), else its calling card icons.
/// - Modern Warfare 3: `rank_prestige1` .. `rank_prestige20`, else its
///   `cardicon_prestige*` calling card icons.
#[must_use]
pub fn icon_candidates(game: &str, n: i32) -> Vec<String> {
    match game {
        "t5" => vec![
            format!("rank_prestige{n:02}"),
            format!("rank_prestige{n}"),
            format!("rank_prestige_{n}"),
            format!("cardicon_prestige{n}"),
        ],
        "iw4" => vec![
            format!("rank_prestige{n}"),
            format!("rank_prestige{n:02}"),
            format!("cardicon_prestige_{n}"),
            format!("cardicon_prestige{n}"),
        ],
        "iw5" => vec![
            format!("rank_prestige{n}"),
            format!("cardicon_prestige{n}"),
            format!("cardicon_prestige_{n}"),
            format!("cardicon_prestige{n}_1"),
            format!("rank_prestige{n:02}"),
        ],
        _ => Vec::new(),
    }
}

/// BO2's own rank pictures, read off its mp/rankIconTable.csv: each
/// level's (column 1, 0 = level 1) and the Prestige Master's (the
/// `maxprestige` column).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bo2RankIcons {
    /// Level icons by rank (0 = level 1), BO2's 55 or the extended 155.
    pub levels: Vec<String>,
    /// The Prestige Master's icon.
    pub master: String,
}

impl Bo2RankIcons {
    /// From mp/rankIconTable.csv's rows (BO2's own or `extend_table`'s:
    /// either way the `maxprestige` column of row 0 is the master's).
    #[must_use]
    pub fn from_rank_icon_table(rows: &[Vec<String>]) -> Self {
        let cell = |r: &[String], i: usize| r.get(i).map_or("", |c| c.trim()).to_owned();
        let mut levels: Vec<(i32, String)> = rows
            .iter()
            .filter_map(|r| Some((r.first()?.trim().parse::<i32>().ok()?, cell(r, 1))))
            .collect();
        levels.sort_by_key(|(id, _)| *id);
        let max = rows
            .iter()
            .find(|r| r.first().is_some_and(|c| c.trim() == "maxprestige"))
            .and_then(|r| r.get(1)?.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let master = rows
            .iter()
            .find(|r| max > 0 && r.first().is_some_and(|c| c.trim() == "0"))
            .map_or_else(String::new, |r| cell(r, max + 1));
        Self { levels: levels.into_iter().map(|(_, n)| n).collect(), master }
    }

    /// A level's own icon (1 first): BO2's, or past its last its last.
    #[must_use]
    pub fn level(&self, level: i32) -> &str {
        let i = usize::try_from(level - 1).unwrap_or(0);
        self.levels.get(i).or(self.levels.last()).map_or("", String::as_str)
    }
}

static BO2_ICONS: RwLock<Option<Bo2RankIcons>> = RwLock::new(None);

/// Keep BO2's rank pictures for `rank_icon` (once its table has loaded;
/// `extend_table` does this as it extends mp/rankIconTable.csv).
pub fn register_bo2_rank_icons(icons: Bo2RankIcons) {
    if icons.levels.is_empty() && icons.master.is_empty() {
        return;
    }
    if let Ok(mut g) = BO2_ICONS.write() {
        *g = Some(icons);
    }
}

/// BO2's rank pictures as last registered (none before its table loads).
#[must_use]
pub fn bo2_rank_icons() -> Option<Bo2RankIcons> {
    BO2_ICONS.read().ok().and_then(|g| g.clone())
}

/// A level's icon at a prestige in the combined table: the game's prefix
/// (`t6` BO2, `t5` Black Ops, `iw4` Modern Warfare 2, `iw5` Modern Warfare
/// 3) and its material, the first of `icon_candidates` for another game's
/// (the picture loader tries the rest). `level` counts from 1 (rank + 1),
/// clamped to 1 ..= `MAX_LEVEL`; `prestige` to 0 ..= `MAX_PRESTIGE`. BO2's
/// own names come from its registered rank icon table; before it loads a
/// level's icon is empty and the master's `rank_prestige_master`.
#[must_use]
pub fn rank_icon(level: i32, prestige: i32) -> (&'static str, String) {
    rank_icon_with(bo2_rank_icons().as_ref(), level, prestige)
}

/// `rank_icon` with BO2's own pictures given (`None`: not yet known).
#[must_use]
pub fn rank_icon_with(bo2: Option<&Bo2RankIcons>, level: i32, prestige: i32) -> (&'static str, String) {
    let level = level.clamp(1, MAX_LEVEL);
    let first = |game: &'static str, n: i32| (game, icon_candidates(game, n).swap_remove(0));
    match prestige_icon(prestige) {
        // bo2mp: levels wear Modern Warfare 3's rank icons (its own
        // pictures, read from its files), not Black Ops II's.
        PrestigeIcon::Bo2Level => {
            let _ = bo2;
            ("iw5", mw3_level_icon(level).to_owned())
        }
        PrestigeIcon::Bo1(n) => first("t5", n),
        PrestigeIcon::Mw2(n) => first("iw4", n),
        PrestigeIcon::Mw3(n) => first("iw5", n),
        PrestigeIcon::Bo2Master => (
            BO2,
            bo2.map(|b| b.master.clone()).filter(|m| !m.is_empty()).unwrap_or_else(|| "rank_prestige_master".to_owned()),
        ),
    }
}

/// Modern Warfare 3's rank icon for a level: its ranks in order (Private
/// to Commander, the order of its `rank_*1` materials), three levels a
/// rank through the enlisted ranks, four through the officers, General
/// from 70 and Commander at 80 and above (the extended levels).
#[must_use]
pub fn mw3_level_icon(level: i32) -> &'static str {
    const ENLISTED: [&str; 11] = [
        "rank_pvt1", "rank_pfc1", "rank_spc1", "rank_cpl1", "rank_sgt1", "rank_ssgt1", "rank_sfc1", "rank_msgt1",
        "rank_1stsgt1", "rank_sgtmaj1", "rank_cmdsgtmaj1",
    ];
    const OFFICERS: [&str; 9] = [
        "rank_2ndlt1", "rank_1stlt1", "rank_capt1", "rank_maj1", "rank_ltcol1", "rank_col1", "rank_bgen1",
        "rank_majgen1", "rank_ltgen1",
    ];
    let level = level.max(1);
    if level >= 80 {
        "rank_comm1"
    } else if level >= 70 {
        "rank_gen1"
    } else if level <= 33 {
        ENLISTED[((level - 1) / 3) as usize]
    } else {
        OFFICERS[(((level - 34) / 4) as usize).min(OFFICERS.len() - 1)]
    }
}

/// The picture name the menus get for a game's material: BO2's as it is,
/// another game's as `<game>:<material>`.
#[must_use]
pub fn picture_name(game: &str, material: &str) -> String {
    if game == BO2 || material.is_empty() { material.to_owned() } else { format!("{game}:{material}") }
}

/// `rank_icon` as the menus' picture name (`picture_name`).
#[must_use]
pub fn rank_picture(level: i32, prestige: i32) -> String {
    let (game, material) = rank_icon(level, prestige);
    picture_name(game, &material)
}

/// A menu picture name from another game (`t5:rank_prestige01`): the game's
/// prefix and the names to try, the given one first, then the rest of its
/// prestige's `icon_candidates`.
#[must_use]
pub fn foreign_picture(name: &str) -> Option<(&'static str, Vec<String>)> {
    let (game, material) = name.split_once(':')?;
    let game = ["t5", "iw4", "iw5", "iw6", "t4", "iw3", "s1"].into_iter().find(|g| *g == game)?;
    // (`t5:menu_mp_weapons_ak74u|hud_icon_ak74u`: the names to try, in order.)
    let mut parts = material.split('|');
    let material = parts.next().unwrap_or(material);
    let mut names = vec![material.to_owned()];
    names.extend(parts.filter(|p| !p.is_empty()).map(str::to_owned));
    let ranked = material.contains("rank") || material.contains("prestige");
    let n = material.trim_start_matches(|c: char| !c.is_ascii_digit());
    let n = n.split(|c: char| !c.is_ascii_digit()).next().and_then(|d| d.parse::<i32>().ok());
    if let Some(n) = n.filter(|_| ranked) {
        for c in icon_candidates(game, n) {
            if !names.contains(&c) {
                names.push(c);
            }
        }
    }
    Some((game, names))
}

/// Whether `extend_table` changes this table (name any case).
#[must_use]
pub fn is_rank_table(name: &str) -> bool {
    name.eq_ignore_ascii_case("mp/rankTable.csv")
        || name.eq_ignore_ascii_case("mp/rankIconTable.csv")
        || crate::guest::is_guest_table(name)
}

/// Extend a BO2 string table as it loads (name any case; row-major cells):
/// mp/rankTable.csv to `MAX_LEVEL` levels, mp/rankIconTable.csv to the
/// combined prestiges. True when it changed. Running it again changes
/// nothing.
pub fn extend_table(name: &str, columns: &mut usize, rows: &mut usize, cells: &mut Vec<String>) -> bool {
    let mut grid: Vec<Vec<String>> = cells.chunks((*columns).max(1)).map(<[String]>::to_vec).collect();
    let changed = if name.eq_ignore_ascii_case("mp/rankTable.csv") {
        extend_rank_rows(&mut grid)
    } else if name.eq_ignore_ascii_case("mp/rankIconTable.csv") {
        extend_rank_icon_rows(&mut grid)
    } else if name.eq_ignore_ascii_case("mp/statsTable.csv") {
        crate::guest::extend_stats_rows(&mut grid)
    } else if name.eq_ignore_ascii_case("mp/attachmentTable.csv") {
        crate::guest::extend_attachment_rows(&mut grid)
    } else {
        return false;
    };
    if changed {
        let width = grid.iter().map(Vec::len).max().unwrap_or(0).max(*columns);
        *columns = width;
        *rows = grid.len();
        *cells = grid
            .into_iter()
            .flat_map(|mut r| {
                r.resize(width, String::new());
                r
            })
            .collect();
    }
    changed
}

fn id_of(row: &[String]) -> Option<i32> {
    row.first()?.trim().parse().ok()
}

fn num_at(row: &[String], i: usize) -> Option<i64> {
    row.get(i)?.trim().parse().ok()
}

/// mp/rankTable.csv to `MAX_LEVEL` rows (rows of the table, any order):
/// each new rank a copy of BO2's last (its name, icon and unlock tokens),
/// its id (column 0), shown level, start (column 2), XP to the next
/// (column 3) and end (column 7) carried on. The XP a level takes keeps
/// growing as BO2's last ten levels' did. `maxrank` becomes `MAX_RANK`.
/// Rows past 154 or a table already this long are left alone.
pub fn extend_rank_rows(rows: &mut Vec<Vec<String>>) -> bool {
    let mut changed = false;
    // BO2's own level rows, by id.
    let mut ranks: Vec<(i32, i64, i64)> =
        rows.iter().filter_map(|r| Some((id_of(r)?, num_at(r, 2)?, num_at(r, 7)?))).collect();
    ranks.sort_by_key(|r| r.0);
    if let Some(&(last_id, last_min, last_max)) = ranks.last()
        && last_id < MAX_RANK
        && let Some(template) = rows.iter().find(|r| id_of(r) == Some(last_id)).cloned()
    {
        // The XP each level takes, and how much more each next one takes.
        let steps: Vec<i64> = ranks.iter().map(|(_, lo, hi)| hi - lo).collect();
        let k = steps.len().saturating_sub(1).min(10);
        let growth = if k > 0 {
            ((steps[steps.len() - 1] - steps[steps.len() - 1 - k]) as f64 / k as f64).max(0.0)
        } else {
            0.0
        };
        let (t_id, t_level) = (last_id.to_string(), (last_id + 1).to_string());
        let t_step = (last_max - last_min).to_string();
        let (t_min, t_max) = (last_min.to_string(), last_max.to_string());
        let (mut min, mut step) = (last_max, (last_max - last_min) as f64);
        for id in last_id + 1..=MAX_RANK {
            step += growth;
            // (Round to BO2's own grain: its steps are whole hundreds.)
            let xp = ((step / 100.0).round() as i64 * 100).max(100);
            let max = min + xp;
            let row: Vec<String> = template
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let t = c.trim();
                    if i == 0 {
                        id.to_string()
                    } else if i == 2 {
                        min.to_string()
                    } else if i == 7 {
                        max.to_string()
                    } else if t == t_step {
                        xp.to_string()
                    } else if t == t_level {
                        (id + 1).to_string()
                    // (Another column counting the rank or its XP.)
                    } else if t == t_id {
                        id.to_string()
                    } else if t == t_min {
                        min.to_string()
                    } else if t == t_max {
                        max.to_string()
                    } else {
                        c.clone()
                    }
                })
                .collect();
            rows.push(row);
            min = max;
        }
        changed = true;
    }
    changed |= set_key_row(rows, "maxrank", MAX_RANK);
    changed
}

/// mp/rankIconTable.csv to `MAX_LEVEL` rows and `MAX_PRESTIGE` prestige
/// columns: each rank's row its BO2 level icon (column 1; past BO2's last
/// level, its last's), then each prestige's icon (`rank_icon`, as a menu
/// picture name), the last BO2's Prestige Master. `maxprestige` becomes
/// `MAX_PRESTIGE`. BO2's own pictures are registered for `rank_icon`.
pub fn extend_rank_icon_rows(rows: &mut Vec<Vec<String>>) -> bool {
    let bo2 = Bo2RankIcons::from_rank_icon_table(rows);
    if bo2.levels.is_empty() {
        return false;
    }
    register_bo2_rank_icons(bo2.clone());
    let mut changed = false;
    for rank in 0..=MAX_RANK {
        let want: Vec<String> = std::iter::once(rank.to_string())
            .chain((0..=MAX_PRESTIGE).map(|p| {
                let (game, material) = rank_icon_with(Some(&bo2), rank + 1, p);
                picture_name(game, &material)
            }))
            .collect();
        match rows.iter_mut().find(|r| id_of(r) == Some(rank)) {
            Some(r) if *r == want => {}
            Some(r) => {
                *r = want;
                changed = true;
            }
            None => {
                rows.push(want);
                changed = true;
            }
        }
    }
    changed |= set_key_row(rows, "maxprestige", MAX_PRESTIGE);
    changed
}

/// A `<key>,<value>` row (`maxrank`, `maxprestige`) set to `value` (added
/// when missing). True when it changed.
fn set_key_row(rows: &mut Vec<Vec<String>>, key: &str, value: i32) -> bool {
    let v = value.to_string();
    match rows.iter_mut().find(|r| r.first().is_some_and(|c| c.trim() == key)) {
        Some(r) if r.get(1).is_some_and(|c| c.trim() == v) => false,
        Some(r) => {
            if r.len() < 2 {
                r.resize(2, String::new());
            }
            r[1] = v;
            true
        }
        None => {
            rows.push(vec![key.to_owned(), v]);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|c| (*c).to_owned()).collect()
    }

    /// A small rank table in BO2's columns (0 id, 1 name, 2 start, 3 to
    /// next, 7 end, 14 level shown, 17 tokens).
    fn rank_table() -> Vec<Vec<String>> {
        let mut rows = vec![row(&["maxrank", "54"])];
        let mut min = 0;
        for id in 0..=54 {
            let step = 800 + id * 500;
            let level = (id + 1).to_string();
            rows.push(row(&[
                &id.to_string(),
                "rank",
                &min.to_string(),
                &step.to_string(),
                "",
                "MPUI_RANK",
                "",
                &(min + step).to_string(),
                "",
                "",
                "",
                "",
                "",
                "",
                &level,
                "",
                "",
                "token",
            ]));
            min += step;
        }
        rows
    }

    #[test]
    fn rank_table_goes_on_to_155_along_bo2s_curve() {
        let mut rows = rank_table();
        assert!(extend_rank_rows(&mut rows));
        let r = |id: i32| rows.iter().find(|r| id_of(r) == Some(id)).unwrap().clone();
        assert_eq!(rows.iter().filter(|r| id_of(r).is_some()).count(), MAX_LEVEL as usize);
        let (last, next) = (r(54), r(55));
        // Starts where BO2's last ends, shows level 56, keeps its name.
        assert_eq!(next[2], last[7]);
        assert_eq!((next[1].as_str(), next[14].as_str(), next[17].as_str()), ("rank", "56", "token"));
        // Each level takes more than the one before (BO2's grows by 500).
        let step = |r: &[String]| num_at(r, 7).unwrap() - num_at(r, 2).unwrap();
        assert_eq!(step(&next), step(&last) + 500);
        assert_eq!(next[3], step(&next).to_string());
        assert!(step(&r(154)) > step(&r(100)));
        assert_eq!(r(154)[14], "155");
        assert!(rows.iter().any(|r| r[0] == "maxrank" && r[1] == "154"));
        // Once is enough.
        assert!(!extend_rank_rows(&mut rows));
    }

    #[test]
    fn prestiges_run_bo1_then_mw2_then_mw3_then_bo2s_master() {
        assert_eq!(MAX_PRESTIGE, 46);
        let bo2 = Bo2RankIcons { levels: vec!["rank_pvt".into(), "rank_com".into()], master: "rank_master".into() };
        let at = |l, p| rank_icon_with(Some(&bo2), l, p);
        // Levels wear Modern Warfare 3's rank icons.
        assert_eq!(at(1, 0), ("iw5", "rank_pvt1".to_owned()));
        assert_eq!(at(15, 0), ("iw5", "rank_sgt1".to_owned()));
        assert_eq!(at(34, 0), ("iw5", "rank_2ndlt1".to_owned()));
        assert_eq!(at(70, 0), ("iw5", "rank_gen1".to_owned()));
        assert_eq!(at(155, 0), ("iw5", "rank_comm1".to_owned()));
        assert_eq!(at(1, 1), ("t5", "rank_prestige01".to_owned()));
        assert_eq!(at(1, 15), ("t5", "rank_prestige15".to_owned()));
        assert_eq!(at(1, 16), ("iw4", "rank_prestige1".to_owned()));
        assert_eq!(at(1, 25), ("iw4", "rank_prestige10".to_owned()));
        assert_eq!(at(1, 26), ("iw5", "rank_prestige1".to_owned()));
        assert_eq!(at(1, 45), ("iw5", "rank_prestige20".to_owned()));
        assert_eq!(at(1, 46), ("t6", "rank_master".to_owned()));
        assert_eq!(at(1, 99), at(1, 46));
        assert_eq!(picture_name("t5", "rank_prestige01"), "t5:rank_prestige01");
        assert_eq!(picture_name("t6", "rank_pvt"), "rank_pvt");
        let (game, names) = foreign_picture("iw5:rank_prestige7").unwrap();
        assert_eq!((game, names[0].as_str()), ("iw5", "rank_prestige7"));
        assert!(names.contains(&"cardicon_prestige7".to_owned()));
        assert!(foreign_picture("rank_pvt").is_none() && foreign_picture("x:y").is_none());
    }

    #[test]
    fn rank_icon_table_gets_every_level_and_prestige() {
        // BO2's shape: a level icon, prestige 1-10, the master (11).
        let mut rows = vec![row(&["maxprestige", "11"])];
        for id in 0..=54 {
            let mut r = row(&[&id.to_string(), &format!("rank_{id}")]);
            r.extend((1..=11).map(|p| format!("rank_prestige{p:02}")));
            rows.push(r);
        }
        let mut cols = 13;
        let mut n = rows.len();
        let mut cells: Vec<String> = rows.into_iter().flat_map(|mut r| {
            r.resize(13, String::new());
            r
        }).collect();
        assert!(extend_table("mp/RANKICONTABLE.csv", &mut cols, &mut n, &mut cells));
        assert_eq!(cols, MAX_PRESTIGE as usize + 2);
        let grid: Vec<Vec<String>> = cells.chunks(cols).map(<[String]>::to_vec).collect();
        let r = |id: &str| grid.iter().find(|r| r[0] == id).unwrap();
        assert_eq!(r("maxprestige")[1], "46");
        assert_eq!(r("154")[1], "iw5:rank_comm1");
        assert_eq!(r("14")[1], "iw5:rank_sgt1");
        assert_eq!(r("3")[2], "t5:rank_prestige01");
        assert_eq!(r("3")[17], "iw4:rank_prestige1");
        assert_eq!(r("3")[47], "rank_prestige11");
        let again = cells.clone();
        assert!(!extend_table("mp/rankIconTable.csv", &mut cols, &mut n, &mut cells));
        assert_eq!(cells, again);
        assert!(!extend_table("mp/gametypesTable.csv", &mut cols, &mut n, &mut cells));
    }
}
