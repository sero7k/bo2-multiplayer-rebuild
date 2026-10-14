//! bo2mp lane C: the players' stats on the server, as BO2's scripts read
//! and write them (its DDL stats: `getdstat`, `setdstat`, `adddstat`,
//! `addplayerstat`, `addweaponstat`, ...), and his own classes from them.
//!
//! The local player's stats are his stats file (`bo2mp_stats.txt`, the
//! front end's: `bo2_profile`'s store by stats path), read when the match
//! first asks and, in a ranked match, written back as they change and at
//! its end (merged into the file, so a path the match never touched stays
//! as the front end left it). A bot's stats start empty and are never
//! saved.
//!
//! The file: what the front end sets (`set_local_stats_path`), else
//! `BO2MP_STATS`, else `iw4l-artifacts/bo2mp_stats.txt` when there is one
//! (a match started straight from the command line).
//!
//! bo2zm: in a Zombies match (not a multiplayer map) `getdstat`,
//! `setdstat` and `adddstat` keep a player's stats as BO2's engine keeps
//! them for the scripts: TranZit's bank (`depositBox`) and weapon locker
//! (`weaponLocker`) live there, under `PlayerStatsByMap` (Die Rise and
//! Buried use TranZit's). Kept for the match; a stat never set reads 0.
//! The multiplayer stats natives keep Zombies' stand-ins there.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use bevy_ecs::prelude::{Resource, World};
use bo2_profile::{Profile, StatValue};
use gsc_t6::{Value, Vm};

use super::{Zm, arg, client, entnum, text};

static LOCAL_STATS: Mutex<Option<PathBuf>> = Mutex::new(None);
static RANKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the next match is ranked (Combat Training: XP counts, the
/// public class set), as the front end's playlist chose; `BO2MP_RANKED=1`
/// for a match started from the command line.
pub fn set_local_match_ranked(ranked: bool) {
    RANKED.store(ranked, std::sync::atomic::Ordering::Relaxed);
}

/// The class set the local player's classes come from, as the front end
/// that edited them chose (`None`: by the match, see `class_set`). The
/// shell's own Create-a-Class edits `bo2_profile::LOCAL_MATCH_SET` while
/// its matches are ranked (they earn XP), so the set can't follow from
/// `ranked` alone: BO2's own front end edits the public set in Combat
/// Training.
static LOCAL_CLASS_SET: Mutex<Option<String>> = Mutex::new(None);

/// The class set the next match gives the local player (the front end
/// calls this at START MATCH, with the set its Create-a-Class edited).
pub fn set_local_class_set(set: Option<&str>) {
    if let Ok(mut s) = LOCAL_CLASS_SET.lock() {
        *s = set.map(str::to_owned);
    }
}

pub fn ranked() -> bool {
    RANKED.load(std::sync::atomic::Ordering::Relaxed) || std::env::var("BO2MP_RANKED").is_ok_and(|v| v == "1")
}

/// His stats file for the matches this process plays (the front end calls
/// this at START MATCH).
pub fn set_local_stats_path(path: PathBuf) {
    if let Ok(mut p) = LOCAL_STATS.lock() {
        *p = Some(path);
    }
}

fn local_path() -> Option<PathBuf> {
    if let Some(p) = LOCAL_STATS.lock().ok().and_then(|p| p.clone()) {
        return Some(p);
    }
    if let Ok(p) = std::env::var("BO2MP_STATS") {
        return Some(PathBuf::from(p));
    }
    let p = PathBuf::from("iw4l-artifacts").join("bo2mp_stats.txt");
    p.exists().then_some(p)
}

/// How often changed stats are written (ms of match time).
const SAVE_EVERY_MS: i64 = 10000;

/// One player's stats.
pub(crate) struct PlayerStats {
    /// The items (his tables) and the stats; his classes read from them.
    pub profile: Profile,
    /// His stats file (the local player only).
    file: Option<PathBuf>,
    /// The paths changed since the last save.
    changed: BTreeSet<String>,
}

#[derive(Resource, Default)]
pub(crate) struct ServerStats {
    players: HashMap<u32, PlayerStats>,
    last_save: i64,
}

fn table_rows(world: &World, name: &str) -> Vec<Vec<String>> {
    let zm = world.resource::<Zm>();
    let Some(t) = zm.tables.get(name) else {
        return Vec::new();
    };
    t.cells.chunks(t.columns.max(1)).map(<[String]>::to_vec).collect()
}

/// Whether a client is the local player (the listen server's own, not a
/// bot).
fn is_local(world: &World, client: u32) -> bool {
    client == 0 && !world.resource::<Zm>().bots.contains_key(&client)
}

/// A player's stats, read the first time they are asked for.
pub(crate) fn player(world: &mut World, client: u32) -> &mut PlayerStats {
    if !world.contains_resource::<ServerStats>() {
        world.insert_resource(ServerStats::default());
    }
    if !world.resource::<ServerStats>().players.contains_key(&client) {
        let file = is_local(world, client).then(local_path).flatten();
        if file.is_some() {
            bo2_profile::wait_saved(std::time::Duration::from_secs(10));
        }
        let stats = file.as_deref().map(Profile::read_stats_file).unwrap_or_default();
        let (items, atts) = (table_rows(world, "mp/statstable.csv"), table_rows(world, "mp/attachmenttable.csv"));
        let mut profile = Profile::from_tables(&items, &atts, &|_| None, stats);
        profile.match_set = class_set(world);
        // bo2mp: a bot plays at the level the lobby showed for its name.
        if world.resource::<Zm>().bots.contains_key(&client) {
            let name = super::frame(world)
                .client_meta(crate::world::ClientId(client))
                .map(|m| String::from_utf8_lossy(&m.name).trim_end_matches('\0').to_owned())
                .unwrap_or_default();
            let rank = crate::lobby_bots::level_for(&name) - 1;
            let min_xp = table_rows(world, "mp/ranktable.csv")
                .iter()
                .find(|r| r.first().and_then(|c| c.trim().parse::<i32>().ok()) == Some(rank))
                .and_then(|r| r.get(2).and_then(|c| c.trim().parse::<f32>().ok()))
                .unwrap_or(0.0);
            profile.set_stat(bo2_profile::RANK.to_owned(), StatValue::Num(rank as f32));
            profile.set_stat("playerstatslist.rankxp.statvalue".to_owned(), StatValue::Num(min_xp));
        }
        if let Some(f) = &file {
            diag::info!(
                Sim,
                "bo2mp stats: player {client} from {} ({} stats, rank {}, classes from {})",
                f.display(),
                profile.stats.len(),
                profile.rank(),
                profile.match_set
            );
        }
        world
            .resource_mut::<ServerStats>()
            .players
            .insert(client, PlayerStats { profile, file, changed: BTreeSet::new() });
    }
    world.resource_mut::<ServerStats>().into_inner().players.get_mut(&client).expect("inserted above")
}

/// The class set a match gives: the one the front end said its
/// Create-a-Class edited, else BO2's `cacRoot` (a ranked match, Combat
/// Training, the public one, else the custom match set).
fn class_set(world: &World) -> String {
    if let Some(s) = LOCAL_CLASS_SET.lock().ok().and_then(|s| s.clone()) {
        return s;
    }
    class_set_for(world.resource::<Zm>().ranked_match).to_owned()
}

fn class_set_for(ranked: bool) -> &'static str {
    if ranked { "cacloadouts" } else { bo2_profile::LOCAL_MATCH_SET }
}

/// The local player's profile, when the match has his stats (his classes).
pub(crate) fn local_profile(world: &mut World, client: u32) -> Option<&Profile> {
    if !is_local(world, client) {
        return None;
    }
    let p = player(world, client);
    p.file.is_some().then_some(&p.profile)
}

/// His classes as his stats file holds them now (Create-a-Class during the
/// match writes them there): read again into the match's copy.
pub(crate) fn refresh_classes(world: &mut World, client: u32) {
    let Some(file) = player(world, client).file.clone() else { return };
    bo2_profile::wait_saved(std::time::Duration::from_secs(2));
    let saved = Profile::read_stats_file(&file);
    let p = player(world, client);
    let mut n = 0;
    for (k, v) in saved {
        if k.contains("customclass") && p.profile.stat(&k) != Some(&v) {
            p.profile.set_stat(k, v);
            n += 1;
        }
    }
    if n > 0 {
        diag::info!(Sim, "bo2mp stats: player {client}'s classes read again ({n} changed)");
    }
}

fn get(world: &mut World, client: u32, path: &str) -> Option<StatValue> {
    player(world, client).profile.stat(path).cloned()
}

fn set(world: &mut World, client: u32, path: String, v: StatValue) {
    let p = player(world, client);
    if p.profile.stat(&path) != Some(&v) {
        p.changed.insert(path.clone());
        p.profile.set_stat(path, v);
    }
}

fn add(world: &mut World, client: u32, path: String, delta: f32) {
    let now = get(world, client, &path).and_then(|v| v.as_num()).unwrap_or(0.0);
    set(world, client, path, StatValue::Num(now + delta));
}

/// A stats path from the scripts' words (`"PlayerStatsList", "RANKXP",
/// "StatValue"` -> `playerstatslist.rankxp.statvalue`).
fn path_of(vm: &mut Vm<World>, words: &[Value]) -> String {
    words
        .iter()
        .map(|w| match w {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => (*f as i64).to_string(),
            other => vm.to_text(other).to_ascii_lowercase(),
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn to_value(vm: &mut Vm<World>, v: Option<StatValue>) -> Value {
    match v {
        Some(StatValue::Num(n)) if n == n.trunc() && n.abs() < 2.0e9 => Value::Int(n as i32),
        Some(StatValue::Num(n)) => Value::Float(n),
        Some(StatValue::Str(s)) => vm.string(&s),
        None => Value::Int(0),
    }
}

fn of_value(vm: &mut Vm<World>, v: &Value) -> StatValue {
    match v {
        Value::Int(n) => StatValue::Num(*n as f32),
        Value::Float(f) => StatValue::Num(*f),
        other => {
            let t = vm.to_text(other);
            t.trim().parse::<f32>().map_or(StatValue::Str(t), StatValue::Num)
        }
    }
}

/// A weapon's stats path start (`itemstats.<item index>.stats`): its
/// item from the statstable by reference (`mp7_mp+reflex` -> MP7).
fn weapon_item(world: &mut World, client: u32, weapon: &str) -> Option<i32> {
    let base = weapon.split('+').next().unwrap_or(weapon);
    let reference = base.strip_suffix("_mp").unwrap_or(base);
    player(world, client).profile.item_by_ref(reference).map(|it| it.index)
}

/// Each tick: hand the local player's changed stats to the saver (merged
/// over what his file holds then). Only a ranked match's (Combat
/// Training): as in BO2, a custom game keeps its stats to itself (his
/// Combat Record and weapon stats count only ranked play).
pub(crate) fn save(world: &mut World, force: bool) {
    if !world.resource::<Zm>().ranked_match {
        return;
    }
    let now = world.resource::<Zm>().now_ms;
    let Some(mut s) = world.get_resource_mut::<ServerStats>() else { return };
    if !force && now - s.last_save < SAVE_EVERY_MS {
        return;
    }
    s.last_save = now;
    for (client, p) in &mut s.players {
        let Some(file) = p.file.clone() else { continue };
        if p.changed.is_empty() {
            continue;
        }
        // bo2mp: his classes are Create-a-Class's (the front end's): a
        // match never writes them back (a class the match read at its
        // start must not undo one he made since).
        let (classes, rest): (Vec<String>, Vec<String>) =
            std::mem::take(&mut p.changed).into_iter().partition(|path| path.contains(".customclass"));
        if !classes.is_empty() {
            diag::info!(Sim, "bo2mp stats: player {client}'s class stats left out of the save: {}", classes.join(", "));
        }
        let changed: Vec<(String, StatValue)> = rest
            .into_iter()
            .filter_map(|path| p.profile.stat(&path).cloned().map(|v| (path, v)))
            .collect();
        if changed.is_empty() {
            continue;
        }
        let client = *client;
        let done: bo2_profile::SaveDone = Box::new(move |file, result| match result {
            Ok(()) => diag::info!(Sim, "bo2mp stats: player {client}'s saved to {}", file.display()),
            Err(e) => diag::warn!(Sim, "bo2mp stats: player {client}'s not saved ({}): {e}", file.display()),
        });
        bo2_profile::save_in_background(&file, bo2_profile::SaveJob::Merge(changed), Some(done));
    }
}

fn client_of(vm: &mut Vm<World>, s: &Value) -> Option<u32> {
    entnum(vm, s)
}

/// bo2mp/bo2zm merge: a multiplayer map's match (the stats above), else
/// Zombies (`DStats` below).
fn mp(world: &World) -> bool {
    world.resource::<Zm>().mp
}

// bo2zm: Zombies' stats, kept for the match (see the top).

#[derive(Clone, Debug)]
enum ZmStat {
    Int(i32),
    Float(f32),
    Text(String),
}

#[derive(Resource, Default)]
pub(crate) struct DStats(HashMap<(u32, Vec<String>), ZmStat>);

/// The stat's path, e.g. `PlayerStatsByMap`, `zm_transit`, `depositBox`
/// (names are not case sensitive).
fn zm_path(vm: &Vm<World>, keys: &[Value]) -> Vec<String> {
    keys.iter()
        .map(|k| vm.to_text(k).to_ascii_lowercase())
        .collect()
}

fn zm_get(world: &World, key: &(u32, Vec<String>)) -> Option<ZmStat> {
    world
        .get_resource::<DStats>()
        .and_then(|d| d.0.get(key).cloned())
}

fn zm_put(world: &mut World, key: (u32, Vec<String>), v: ZmStat) {
    world
        .get_resource_or_insert_with(DStats::default)
        .0
        .insert(key, v);
}

fn zm_stat(vm: &Vm<World>, v: &Value) -> ZmStat {
    match v {
        Value::Int(i) => ZmStat::Int(*i),
        Value::Float(f) => ZmStat::Float(*f),
        Value::Undefined => ZmStat::Int(0),
        v => ZmStat::Text(vm.to_text(v)),
    }
}

// Ranks and XP: the engine's half of BO2's `_rank.gsc` (its `addrankxp`
// and `CodeCallback_RankUp`). The scripts' own half (giverankxp,
// incrankxp, updaterank, syncxpstat) runs as written.

/// A table by name, any case (`mp/rankTable.csv`).
fn table<'a>(zm: &'a Zm, name: &str) -> Option<&'a super::T6Table> {
    zm.tables.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, t)| t)
}

/// mp/rankTable.csv: each rank's id, the XP it starts at, the XP it ends
/// at, and the unlock tokens reaching it gives (column 17: `token` is one,
/// `token 4` four).
struct RankRow {
    id: i32,
    min: f32,
    max: f32,
    tokens: i32,
}

fn rank_rows(world: &World) -> Vec<RankRow> {
    let zm = world.resource::<Zm>();
    let Some(t) = table(zm, "mp/rankTable.csv") else { return Vec::new() };
    t.cells
        .chunks(t.columns.max(1))
        .filter_map(|r| {
            let cell = |i: usize| r.get(i).map_or("", |c| c.trim());
            Some(RankRow {
                id: cell(0).parse().ok()?,
                min: cell(2).parse().ok()?,
                max: cell(7).parse().ok()?,
                tokens: cell(17)
                    .strip_prefix("token")
                    .map_or(0, |n| n.trim().parse().unwrap_or(1)),
            })
        })
        .collect()
}

fn rank_for_xp(rows: &[RankRow], xp: f32) -> i32 {
    rows.iter().filter(|r| r.min <= xp).map(|r| r.id).max().unwrap_or(0)
}

/// An event's row in mp/scoreInfo.csv: its XP in this game type (its
/// `<gametype> XP` column, what BO2's _rank.gsc registers with registerxp),
/// whether it also counts toward the weapon (its GunXP), and its row number
/// when it is a medal (a Medal Reference in column 2).
struct ScoreEvent {
    xp: f32,
    gun: bool,
    medal: Option<i32>,
}

fn score_event(world: &World, gametype: &str, event: &str) -> Option<ScoreEvent> {
    let zm = world.resource::<Zm>();
    let t = table(zm, "mp/scoreInfo.csv")?;
    let mut rows = t.cells.chunks(t.columns.max(1));
    let header = rows.next()?;
    let col = header.iter().position(|h| h.trim().eq_ignore_ascii_case(&format!("{gametype} XP")))?;
    let gun = header.iter().position(|h| h.trim().eq_ignore_ascii_case("GunXP"));
    // Row numbers count the header as row 0 (the scripts' and the HUD's
    // tablelookup rows).
    let (index, row) = rows.enumerate().find(|(_, r)| r.first().is_some_and(|c| c.trim().eq_ignore_ascii_case(event)))?;
    let xp = row.get(col)?.trim().parse::<f32>().ok()?;
    let gunxp = gun.and_then(|g| row.get(g)).is_some_and(|c| c.trim().eq_ignore_ascii_case("TRUE"));
    let medal = row.get(2).is_some_and(|c| !c.trim().is_empty()).then_some(index as i32 + 1);
    Some(ScoreEvent { xp, gun: gunxp, medal })
}

/// What a stat reads as a number.
fn num(world: &mut World, client: u32, path: &str) -> f32 {
    get(world, client, path).and_then(|v| v.as_num()).unwrap_or(0.0)
}

/// The unlock tokens he has to spend in Create-a-Class (Pick 10): BO2's
/// `unlocks[]` stat, the pool its menus read (CoD.CAC.GetUnlockCountForGroup:
/// `GetDStat(controller, "unlocks", GetUnlockIndexFromGroupName(group))`,
/// one pool, index 0).
use bo2_profile::UNLOCK_TOKENS;
const RANK_XP: &str = "playerstatslist.rankxp.statvalue";

/// XP for a player in a ranked match: his rank XP (`XP_MULTIPLIER` times
/// BO2's award, capped at the last rank's end), the weapon's XP (BO2's
/// award as it is: gun levels keep their pace), and a new rank: its stats,
/// its unlock tokens, and BO2's `CodeCallback_RankUp(rank, prestige,
/// tokens added)` (the scripts' rank-up popup).
fn give_xp(vm: &mut Vm<World>, world: &mut World, who: &Value, client: u32, xp: f32, weapon: Option<(&str, bool)>) {
    if xp <= 0.0 {
        return;
    }
    // bo2mp: the one place match XP turns into rank XP.
    let rank_xp = (xp * bo2_profile::progression::XP_MULTIPLIER).round();
    let rows = rank_rows(world);
    let cap = rows.iter().map(|r| r.max).fold(0.0f32, f32::max);
    let old = num(world, client, "playerstatslist.rankxp.statvalue");
    let new = if cap > 0.0 { (old + rank_xp).min(cap) } else { old + rank_xp };
    set(world, client, "playerstatslist.rankxp.statvalue".to_owned(), StatValue::Num(new));
    if let Some((w, true)) = weapon
        && let Some(item) = weapon_item(world, client, w)
    {
        add(world, client, format!("itemstats.{item}.xp.statvalue"), xp);
    }
    // The scripts' own count (pers["rankxp"]) stays with the stat: their
    // syncxpstat writes it back.
    set_pers_rankxp(vm, who, new);
    rank_up(vm, world, who, client, old, new);
}

/// The scripts' own count of his rank XP (`pers["rankxp"]`).
fn set_pers_rankxp(vm: &mut Vm<World>, who: &Value, xp: f32) {
    if let Value::Object(o) = who {
        let pers = vm.intern("pers");
        if let Value::Array(a) = vm.raw_field(*o, pers) {
            let key = gsc_t6::Key::Str(vm.intern("rankxp"));
            a.write().set(key, Value::Int(xp as i32));
        }
    }
}

/// bo2mp: rank XP the scripts gave themselves (giverankxp's incrankxp, then
/// syncxpstat's setdstat) counts `XP_MULTIPLIER` times too: the gain over
/// his stat scaled, capped at the last rank's end, and their count put
/// back to match. (What `give_xp` already gave comes back unchanged.)
fn scale_script_xp(vm: &mut Vm<World>, world: &World, who: &Value, old: f32, written: f32) -> f32 {
    if written <= old {
        return written;
    }
    let cap = rank_rows(world).iter().map(|r| r.max).fold(0.0f32, f32::max);
    let scaled = old + ((written - old) * bo2_profile::progression::XP_MULTIPLIER).round();
    let scaled = if cap > 0.0 { scaled.min(cap) } else { scaled };
    set_pers_rankxp(vm, who, scaled);
    scaled
}

/// His rank XP went from `old` to `new` (the engine's addrankxp, or the
/// scripts' giverankxp writing it with setdstat): a new rank gets its
/// stats, its unlock tokens, and BO2's `CodeCallback_RankUp(rank,
/// prestige, tokens added)` (the scripts' rank-up popup).
fn rank_up(vm: &mut Vm<World>, world: &mut World, who: &Value, client: u32, old: f32, new: f32) {
    let rows = rank_rows(world);
    let (from, to) = (rank_for_xp(&rows, old), rank_for_xp(&rows, new));
    if to <= from {
        return;
    }
    let tokens: i32 = rows.iter().filter(|r| r.id > from && r.id <= to).map(|r| r.tokens).sum();
    add(world, client, UNLOCK_TOKENS.to_owned(), tokens as f32);
    set(world, client, "playerstatslist.rank.statvalue".to_owned(), StatValue::Num(to as f32));
    if let Some(r) = rows.iter().find(|r| r.id == to) {
        set(world, client, "playerstatslist.minxp.statvalue".to_owned(), StatValue::Num(r.min));
        set(world, client, "playerstatslist.maxxp.statvalue".to_owned(), StatValue::Num(r.max));
    }
    let prestige = num(world, client, "playerstatslist.plevel.statvalue") as i32;
    diag::info!(Sim, "bo2mp rank: player {client} rank {from} -> {to} ({new} XP, {tokens} unlock tokens)");
    vm.spawn_named(
        world,
        "maps/mp/gametypes/_rank",
        "codecallback_rankup",
        who.clone(),
        vec![Value::Int(to), Value::Int(prestige), Value::Int(tokens)],
    );
}

/// Binds both modes' stats natives (each checks the mode as it runs:
/// Zombies keeps its own stand-ins), so binding twice is harmless.
pub(super) fn bind(vm: &mut Vm<World>) {
    // The match's end: his stats written first, then the session takes it
    // from here (as natives_game's exitlevel).
    vm.bind("exitlevel", false, |_, world, _, _| {
        save(world, true);
        world.resource_mut::<crate::script::Runtime>().exit_level = true;
        Ok(Value::Undefined)
    });
    // A ranked match (Combat Training): XP counts.
    for name in ["gamemodeisusingxp", "gamemodeisusingstats"] {
        vm.bind(name, false, |_, world, _, _| Ok(Value::Int(i32::from(world.resource::<Zm>().ranked_match))));
    }
    // addrankxp(event, weapon, ...): a medal event with XP shows his medal in
    // any match (the engine calls BO2's _medals::codecallback_medal(row)
    // before its stats checks); the event's XP (mp/scoreInfo.csv) counts in
    // a ranked match, scaled by scr_xpscale.
    vm.bind("addrankxp", true, |vm, world, s, a| {
        if !mp(world) {
            return Ok(Value::Undefined);
        }
        let Some(c) = client_of(vm, s) else { return Ok(Value::Undefined) };
        let event = text(vm, a, 0);
        let gametype = vm.dvars.get("g_gametype").cloned().unwrap_or_default();
        let Some(e) = score_event(world, &gametype, &event) else { return Ok(Value::Undefined) };
        if e.xp > 0.0
            && let Some(row) = e.medal
        {
            vm.spawn_named(world, "maps/mp/_medals", "codecallback_medal", s.clone(), vec![Value::Int(row)]);
        }
        if !world.resource::<Zm>().ranked_match {
            return Ok(Value::Undefined);
        }
        let weapon = text(vm, a, 1);
        let scale = vm.dvars.get("scr_xpscale").and_then(|v| v.parse::<f32>().ok()).filter(|x| *x > 0.0).unwrap_or(1.0);
        give_xp(vm, world, s, c, (e.xp * scale).round(), Some((&weapon, e.gun)));
        Ok(Value::Undefined)
    });
    // addrankxpvalue(event, value): that much XP.
    vm.bind("addrankxpvalue", true, |vm, world, s, a| {
        if !mp(world) {
            return Ok(Value::Undefined);
        }
        if !world.resource::<Zm>().ranked_match {
            return Ok(Value::Undefined);
        }
        let Some(c) = client_of(vm, s) else { return Ok(Value::Undefined) };
        let xp = arg(a, 1).as_float().unwrap_or(0.0);
        give_xp(vm, world, s, c, xp, None);
        Ok(Value::Undefined)
    });
    // getdstat(path words...): the stat (0 never written).
    vm.bind("getdstat", true, |vm, world, s, a| {
        if !mp(world) {
            let id = client(vm, world, s)?;
            return Ok(match zm_get(world, &(id.0, zm_path(vm, a))) {
                Some(ZmStat::Int(i)) => Value::Int(i),
                Some(ZmStat::Float(f)) => Value::Float(f),
                Some(ZmStat::Text(t)) => vm.string(&t),
                None => Value::Int(0),
            });
        }
        let Some(c) = client_of(vm, s) else { return Ok(Value::Int(0)) };
        let path = path_of(vm, a);
        let v = get(world, c, &path);
        Ok(to_value(vm, v))
    });
    // setdstat(path words..., value).
    vm.bind("setdstat", true, |vm, world, s, a| {
        if !mp(world) {
            let id = client(vm, world, s)?;
            if let Some((v, keys)) = a.split_last() {
                let v = zm_stat(vm, v);
                zm_put(world, (id.0, zm_path(vm, keys)), v);
            }
            return Ok(Value::Undefined);
        }
        if let (Some(c), Some((value, words))) = (client_of(vm, s), a.split_last()) {
            let path = path_of(vm, words);
            let v = of_value(vm, value);
            // The scripts' own XP (giverankxp's syncxpstat): a new rank
            // as the engine sees one.
            // (His own only: BO2's bot scripts write the bots' made-up
            // ranks the same way.)
            let xp = (path == RANK_XP && world.resource::<Zm>().ranked_match && is_local(world, c))
                .then(|| (num(world, c, RANK_XP), v.as_num().unwrap_or(0.0)));
            let xp = xp.map(|(old, new)| (old, scale_script_xp(vm, world, s, old, new)));
            let v = xp.map_or(v, |(_, new)| StatValue::Num(new));
            set(world, c, path, v);
            if let Some((old, new)) = xp {
                rank_up(vm, world, s, c, old, new);
            }
        }
        Ok(Value::Undefined)
    });
    // adddstat(path words..., amount).
    vm.bind("adddstat", true, |vm, world, s, a| {
        if !mp(world) {
            // Added to what is there.
            let id = client(vm, world, s)?;
            if let Some((v, keys)) = a.split_last() {
                let key = (id.0, zm_path(vm, keys));
                let sum = match (zm_get(world, &key), zm_stat(vm, v)) {
                    (Some(ZmStat::Float(x)), ZmStat::Int(d)) => ZmStat::Float(x + d as f32),
                    (Some(ZmStat::Float(x)), ZmStat::Float(d)) => ZmStat::Float(x + d),
                    (Some(ZmStat::Int(x)), ZmStat::Float(d)) => ZmStat::Float(x as f32 + d),
                    (Some(ZmStat::Int(x)), ZmStat::Int(d)) => ZmStat::Int(x.saturating_add(d)),
                    (_, d) => d,
                };
                zm_put(world, key, sum);
            }
            return Ok(Value::Undefined);
        }
        if let (Some(c), Some((value, words))) = (client_of(vm, s), a.split_last()) {
            let path = path_of(vm, words);
            let n = value.as_float().unwrap_or(0.0);
            add(world, c, path, n);
        }
        Ok(Value::Undefined)
    });
    // addplayerstat(name, amount): PlayerStatsList.<name>.StatValue.
    vm.bind("addplayerstat", true, |vm, world, s, a| {
        if !mp(world) {
            return Ok(Value::Undefined);
        }
        if let Some(c) = client_of(vm, s) {
            let name = text(vm, a, 0).to_ascii_lowercase();
            let n = arg(a, 1).as_float().unwrap_or(0.0);
            add(world, c, format!("playerstatslist.{name}.statvalue"), n);
        }
        Ok(Value::Undefined)
    });
    // addplayerstatwithgametype(name, amount): its game type's row too
    // (PlayerStatsByGameType.<gametype>.<name>.StatValue).
    vm.bind("addplayerstatwithgametype", true, |vm, world, s, a| {
        if !mp(world) {
            return Ok(Value::Undefined);
        }
        if let Some(c) = client_of(vm, s) {
            let name = text(vm, a, 0).to_ascii_lowercase();
            let n = arg(a, 1).as_float().unwrap_or(0.0);
            let gametype = vm.dvars.get("g_gametype").cloned().unwrap_or_default().to_ascii_lowercase();
            add(world, c, format!("playerstatsbygametype.{gametype}.{name}.statvalue"), n);
        }
        Ok(Value::Undefined)
    });
    // addweaponstat(weapon, stat, amount): ItemStats.<item>.stats.<stat>.
    vm.bind("addweaponstat", true, |vm, world, s, a| {
        if !mp(world) {
            return Ok(Value::Undefined);
        }
        if let Some(c) = client_of(vm, s) {
            let weapon = text(vm, a, 0);
            let stat = text(vm, a, 1).to_ascii_lowercase();
            let n = arg(a, 2).as_float().unwrap_or(0.0);
            if let Some(item) = weapon_item(world, c, &weapon) {
                add(world, c, format!("itemstats.{item}.stats.{stat}.statvalue"), n);
            }
        }
        Ok(Value::Undefined)
    });
    // getrankxpstat() / getcodpointsstat(): PlayerStatsList.rankxp / codpoints.
    vm.bind("getrankxpstat", true, |vm, world, s, _| {
        if !mp(world) {
            return Ok(Value::Int(0));
        }
        let Some(c) = client_of(vm, s) else { return Ok(Value::Int(0)) };
        let v = get(world, c, "playerstatslist.rankxp.statvalue");
        Ok(to_value(vm, v))
    });
    vm.bind("getcodpointsstat", true, |vm, world, s, _| {
        if !mp(world) {
            return Ok(Value::Int(0));
        }
        let Some(c) = client_of(vm, s) else { return Ok(Value::Int(0)) };
        let v = get(world, c, "playerstatslist.codpoints.statvalue");
        Ok(to_value(vm, v))
    });
}

#[cfg(test)]
mod tests {
    /// Without the front end's word, a ranked match gives the public set
    /// and a custom one the custom match set (BO2's `cacRoot`).
    #[test]
    fn class_set_by_match() {
        assert_eq!(super::class_set_for(true), "cacloadouts");
        assert_eq!(super::class_set_for(false), bo2_profile::LOCAL_MATCH_SET);
    }
}
