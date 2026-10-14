//! bo2mp: a player's class as Black Ops II's class script reads it
//! (`getloadoutweapon`, `getloadoutitem`, `getloadoutitemref`,
//! `getloadoutperks`, `isbonuscardactive`, `getweaponattachments`).
//!
//! Items come from his `mp/statstable.csv` (index, group, name, reference,
//! attachments, default classes, slot). A bot's class is what its own
//! scripts built (`botclassadditem`, `botclassaddattachment`); a player's
//! is BO2's default class for that slot (mp/reset_classes.cfg: the five
//! default custom classes, then the same five again) until his saved
//! classes are read: with his stats file (`stats`), the local player's
//! class is his own, as the front end's Create-a-Class saved it.

use std::sync::{Arc, Mutex, PoisonError};

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};

use super::{Zm, arg, entnum, list, text};

const TABLE: &str = "mp/statstable.csv";

/// BO2's default custom classes in class order.
const DEFAULT_CLASSES: [&str; 5] = [
    "class_custom_assault",
    "class_custom_smg",
    "class_custom_lmg",
    "class_custom_cqb",
    "class_custom_sniper",
];

#[derive(Clone, Debug, Default)]
struct Item {
    index: i32,
    name: String,
    reference: String,
    /// Its statstable group (`weapon_sniper`).
    group: String,
    attachments: Vec<String>,
    /// Default classes it is in, each with its extra words (a count).
    classes: Vec<(String, Vec<String>)>,
    slot: String,
    /// The HUD picture (a scorestreak's) and its momentum cost.
    icon: String,
    cost: i32,
}

/// The table's items, read once (its size the key: the table is read
/// again with the level): the HUD asks each tick, the class scripts at
/// each spawn.
#[derive(Default)]
pub(crate) struct ItemCache(Mutex<Option<((usize, usize), Arc<Vec<Item>>)>>);

fn items(world: &World) -> Arc<Vec<Item>> {
    let zm = world.resource::<Zm>();
    let Some(t) = zm.tables.get(TABLE) else {
        return Arc::default();
    };
    let key = (t.rows, t.cells.len());
    let mut cache = zm.items.0.lock().unwrap_or_else(PoisonError::into_inner);
    match &*cache {
        Some((k, all)) if *k == key => all.clone(),
        _ => {
            let all = Arc::new(read_items(t));
            *cache = Some((key, all.clone()));
            all
        }
    }
}

fn read_items(t: &super::T6Table) -> Vec<Item> {
    let cell = |r: usize, c: usize| t.cells.get(r * t.columns + c).map_or("", String::as_str);
    let mut out = Vec::new();
    for r in 0..t.rows {
        let Ok(index) = cell(r, 0).parse::<i32>() else {
            continue;
        };
        if cell(r, 4).is_empty() {
            continue;
        }
        let mut classes: Vec<(String, Vec<String>)> = Vec::new();
        for w in cell(r, 11).split_whitespace() {
            if w.starts_with("class_") || w == "all" {
                classes.push((w.to_owned(), Vec::new()));
            } else if let Some(last) = classes.last_mut() {
                last.1.push(w.to_owned());
            }
        }
        out.push(Item {
            index,
            name: cell(r, 3).to_owned(),
            reference: cell(r, 4).to_owned(),
            group: cell(r, 2).to_owned(),
            attachments: cell(r, 8).split_whitespace().map(str::to_owned).collect(),
            classes,
            slot: cell(r, 13).to_owned(),
            icon: cell(r, 6).to_owned(),
            cost: cell(r, 16).parse().unwrap_or(0),
        });
    }
    out
}

/// A player's scorestreaks as BO2's HUD column lists them: `icon:cost`
/// by `;`, the three he picked (the default ones, in table order, until
/// his own are read), the dearest first (the column lists it top down).
pub(crate) fn streak_picks(world: &World) -> String {
    let mut picks: Vec<(String, i32)> = items(world)
        .iter()
        .filter(|i| i.slot.starts_with("killstreak") && i.classes.iter().any(|(c, _)| c == "all") && i.cost > 0)
        .map(|i| (i.icon.clone(), i.cost))
        .collect();
    picks.truncate(3);
    picks.sort_by_key(|p| std::cmp::Reverse(p.1));
    picks.iter().map(|(i, c)| format!("{i}:{c}")).collect::<Vec<_>>().join(";")
}

/// A player's own scorestreaks as BO2's HUD column lists them:
/// `icon:cost:held` by `;`, dearest first, `held` 1 while he has one of it
/// to use (earned and not used yet). His picks are the menu references
/// BO2's class script gives him (`player.killstreak`, pick order); not
/// given yet (no array): `None`, the default ones (`streak_picks`) stand
/// in. An empty array (he picked none, or the game type's
/// `loadoutKillstreaksEnabled` is off: `level.loadoutkillstreaksenabled`)
/// is an empty column.
///
/// Held is what BO2's scripts count: `pers["killstreak_quantity"][weapon]`
/// (`changekillstreakquantity`: +1 earned, -1 used; it outlives a death),
/// `pers["held_killstreak_ammo_count"][weapon]` for the guns (Death
/// Machine, War Machine), else the streak weapon in his hands with one in
/// its clip.
pub(crate) fn his_streaks(vm: &mut Vm<World>, world: &mut World, client: u32) -> Option<String> {
    let obj = world.resource::<Zm>().players.get(&client)?.obj;
    let f = vm.intern("killstreak");
    let refs: Vec<String> = match vm.raw_field(obj, f) {
        Value::Array(a) => a.read().values_in_order().map(|v| vm.to_text(v)).collect(),
        _ => return None,
    };
    let enabled = vm.intern("loadoutkillstreaksenabled");
    let enabled = vm.raw_field(vm.level, enabled);
    if refs.is_empty() || (!enabled.is_undefined() && enabled.as_int() == Some(0)) {
        return Some(String::new());
    }
    let all = items(world);
    let (types, killstreaks, weapon, pers) = (
        vm.intern("menureferenceforkillstreak"),
        vm.intern("killstreaks"),
        vm.intern("weapon"),
        vm.intern("pers"),
    );
    let lookup = |vm: &mut Vm<World>, table: &Value, key: &str| match table {
        Value::Array(a) => {
            let k = gsc_t6::Key::Str(vm.intern(key));
            a.read().get(&k).cloned().unwrap_or_default()
        }
        _ => Value::Undefined,
    };
    let (by_menu, by_type) = (vm.raw_field(vm.level, types), vm.raw_field(vm.level, killstreaks));
    let his_pers = vm.raw_field(obj, pers);
    let mut out: Vec<(String, i32, bool)> = Vec::new();
    for r in refs {
        let Some(it) = all.iter().find(|i| i.reference == r) else {
            continue;
        };
        let kind = lookup(vm, &by_menu, &r);
        let kind = vm.to_text(&kind);
        let held = match lookup(vm, &by_type, &kind) {
            Value::Object(o) => {
                let name = vm.to_text(&vm.raw_field(o, weapon));
                let count = |vm: &mut Vm<World>, key: &str| {
                    let per_weapon = lookup(vm, &his_pers, key);
                    lookup(vm, &per_weapon, &name).as_int().unwrap_or(0)
                };
                count(vm, "killstreak_quantity") > 0
                    || count(vm, "held_killstreak_ammo_count") > 0
                    || super::weapon(world, &name).ok().filter(|w| *w != 0).is_some_and(|w| {
                        let f = super::frame(world);
                        let id = crate::world::ClientId(client);
                        f.player(id).is_some_and(|ps| ps.weapons.contains(&(w as i32)))
                            && crate::script_player::ammo_clip(&f, id, w) > 0
                    })
            }
            _ => false,
        };
        out.push((it.icon.clone(), it.cost, held));
    }
    Some(streak_list(out))
}

/// The column's dvar from his picks (icon, cost, held): dearest first (the
/// column lists it top down), equal costs in pick order.
fn streak_list(mut picks: Vec<(String, i32, bool)>) -> String {
    picks.sort_by_key(|p| std::cmp::Reverse(p.1));
    picks.iter().map(|(i, c, h)| format!("{i}:{c}:{}", u8::from(*h))).collect::<Vec<_>>().join(";")
}

/// A player's perks as BO2's perk list draws them (the spawn list, the
/// killcam's killer card): `icon|name` by `;`, perk 1 to 3.
pub(crate) fn perk_card(vm: &mut Vm<World>, world: &mut World, who: &Value, class: i32) -> String {
    let all = items(world);
    let mut picks: Vec<i32> = Vec::new();
    if let Some(p) = his(vm, world, who, "specialty1") {
        picks = (1..=3).map(|n| p.loadout_item(class, &format!("specialty{n}"))).collect();
    } else {
        for n in 1..=3 {
            let slot = format!("specialty{n}");
            picks.push(slot_item(vm, world, who, class, &slot).map_or(0, |(i, _)| i.index));
        }
    }
    let rows: Vec<String> = picks
        .into_iter()
        .filter_map(|index| all.iter().find(|i| i.index == index && !i.reference.is_empty() && i.reference != "specialty_null"))
        .map(|it| format!("{}|{}", it.icon, it.name))
        .collect();
    rows.join(";")
}

/// A class's items by slot: (slot, item, extra words or attachments).
fn class_items(vm: &mut Vm<World>, world: &World, who: &Value, class: i32) -> Vec<(String, Item, Vec<String>)> {
    let all = items(world);
    let n = entnum(vm, who).unwrap_or(u32::MAX);
    let zm = world.resource::<Zm>();
    if let Some(bot) = zm.bots.get(&n)
        && let Some(c) = bot.classes.get(&class)
        && !c.items.is_empty()
    {
        let mut out = Vec::new();
        for name in &c.items {
            let Some(it) = all.iter().find(|i| i.name.eq_ignore_ascii_case(name) || i.reference == *name) else {
                continue;
            };
            let atts: Vec<String> = c
                .attachments
                .iter()
                .filter(|(w, _, _)| w.eq_ignore_ascii_case(&it.name) || *w == it.reference)
                .map(|(_, a, _)| a.clone())
                .collect();
            out.push((it.slot.clone(), it.clone(), atts));
        }
        return out;
    }
    let default = DEFAULT_CLASSES[(class.max(0) as usize) % DEFAULT_CLASSES.len()];
    all.iter()
        .filter_map(|it| {
            // "all": in every default class (the default scorestreaks).
            let (_, words) = it.classes.iter().find(|(c, _)| c == default || c == "all")?;
            Some((it.slot.clone(), it.clone(), words.clone()))
        })
        .collect()
}

fn slot_item(
    vm: &mut Vm<World>,
    world: &World,
    who: &Value,
    class: i32,
    slot: &str,
) -> Option<(Item, Vec<String>)> {
    // Scorestreaks are the player's, not a class's: `killstreakN` is the
    // Nth he picked - a bot's own picks (its scripts add them to class 0),
    // else the default ones, in table order.
    if let Some(n) = slot.strip_prefix("killstreak").and_then(|n| n.parse::<usize>().ok()) {
        return class_items(vm, world, who, 0)
            .into_iter()
            .filter(|(s, _, _)| s.starts_with("killstreak"))
            .nth(n.saturating_sub(1))
            .map(|(_, i, w)| (i, w));
    }
    class_items(vm, world, who, class)
        .into_iter()
        .find(|(s, _, _)| s == slot)
        .map(|(_, i, w)| (i, w))
}

/// The local player's own classes (his stats file), for a class slot his
/// classes hold (scorestreaks are not a class's: those stay as below).
fn his<'w>(vm: &Vm<World>, world: &'w mut World, who: &Value, slot: &str) -> Option<&'w bo2_profile::Profile> {
    let n = entnum(vm, who)?;
    let p = super::stats::local_profile(world, n)?;
    // Scorestreaks are class 0's, shared by every class; his picks count
    // only once he has made some (else the defaults, as below).
    (!slot.starts_with("killstreak") || p.has_streak_picks()).then_some(p)
}

/// bo2mp: what a class's gun becomes when BO2's class script gives it
/// (`giveloadout` asks once a spawn: its `giveweapon` resolves the same
/// name the same way), and a warning when it is not his class's gun.
fn log_loadout_weapon(world: &mut World, client: u32, class: i32, slot: &str, name: &str, why: Option<String>) {
    if why.is_some() {
        diag::warn!(Sim, "bo2mp loadout: client {client} class {class} {slot} falls back: {}", why.as_deref().unwrap_or(""));
    }
    if name == "weapon_null_mp" {
        diag::info!(Sim, "bo2mp loadout: client {client} class {class} {slot} `{name}` (none)");
        return;
    }
    match super::resolve_weapon(world, name) {
        Ok((w, stand_in)) => {
            let script = super::weapon_text(world, w);
            diag::info!(Sim, "bo2mp loadout: client {client} class {class} {slot} `{name}` -> weapon {w} `{script}`");
            // (A BO2 gun's non-sight attachments are not modelled yet: its
            // base gun or sight variant is expected, not a fallback.)
            if let Some(s) = stand_in.filter(|_| bo2_profile::guest::is_guest_name(name)) {
                diag::warn!(Sim, "bo2mp loadout: client {client} class {class} {slot} falls back: {s}");
            }
        }
        Err(e) => diag::warn!(Sim, "bo2mp loadout: client {client} class {class} {slot} `{name}` -> no weapon: {e}"),
    }
}

/// bo2mp: the item a bot's class takes instead of the one its script
/// added (`botclassadditem`), now and then one of the mod's guns of the
/// same group and slot (`bo2_profile::guest::bot_mod_gun`: Black Ops II's
/// bot scripts list only items numbered below 256); `seed` is the bot's,
/// class's and item's. None: the item as added.
pub(super) fn bot_item_swap(world: &mut World, item: &str, seed: u32) -> Option<String> {
    let all = items(world);
    let it = all.iter().find(|i| i.name.eq_ignore_ascii_case(item) || i.reference == item)?;
    if !matches!(it.slot.as_str(), "primary" | "secondary") {
        return None;
    }
    let public = super::stats::public_match();
    let f = super::frame(world);
    let gun = bo2_profile::guest::bot_mod_gun(&it.group, &it.slot, seed, public, |g| {
        f.weapon_index_by_script_name(&bo2_profile::guest::script_name(g)).is_some_and(|w| w != 0)
    })?;
    all.iter().find(|i| i.reference == gun.gun).map(|i| i.name.clone())
}

/// bo2mp: the guns a public match gives for a class slot holding one of the
/// mod's Zombies weapons (the first default class's, CLASS_SMG).
const PUBLIC_STAND_IN_PRIMARY: &str = "mp7_mp";
const PUBLIC_STAND_IN_SECONDARY: &str = "fiveseven_mp";

pub(super) fn bind(vm: &mut Vm<World>) {
    macro_rules! f {
        ($name:literal, $body:expr) => {
            vm.bind($name, false, $body);
        };
    }
    macro_rules! m {
        ($name:literal, $body:expr) => {
            vm.bind($name, true, $body);
        };
    }
    // The gun in a slot: `<reference>_mp`, a bot's attachments after it
    // (`+reflex+fmj`; the engine gives the base gun until attachments draw).
    m!("getloadoutweapon", |vm, world, s, a| {
        let class = arg(a, 0).as_int().unwrap_or(0);
        let slot = text(vm, a, 1);
        let client = entnum(vm, s).unwrap_or(u32::MAX);
        let (name, why) = if let Some(p) = his(vm, world, s, &slot) {
            let set = p.match_set.clone();
            let (w, why) = p.loadout_weapon_checked(class, &slot);
            (w, why.map(|why| format!("his class (set {set}): {why}")))
        } else {
            let mut name = String::from("weapon_null_mp");
            if let Some((it, extra)) = slot_item(vm, world, s, class, &slot) {
                name = format!("{}_mp", it.reference);
                if world.resource::<Zm>().bots.contains_key(&client) {
                    for att in extra.iter().filter(|a| it.attachments.contains(a)) {
                        name.push('+');
                        name.push_str(att);
                    }
                }
            }
            (name, None)
        };
        // bo2mp: in a public match a class holding one of the mod's Zombies
        // weapons (Custom Games only) gets the default class's gun instead.
        let (name, why) = if world.resource::<Zm>().mp && super::stats::public_match() && bo2_profile::guest::is_custom_only(&name) {
            let stand_in = if slot == "secondary" { PUBLIC_STAND_IN_SECONDARY } else { PUBLIC_STAND_IN_PRIMARY };
            (stand_in.to_owned(), Some(format!("`{name}` is for Custom Games only; `{stand_in}` in a public match")))
        } else {
            (name, why)
        };
        log_loadout_weapon(world, client, class, &slot, &name, why);
        Ok(vm.string(&name))
    });
    // bo2mp: the options `giveweapon` takes with a class's gun: its camo
    // (`primarycamo` / `secondarycamo`, slot 0 / 1), as the option's number.
    m!("calcweaponoptions", |vm, world, s, a| {
        let class = arg(a, 0).as_int().unwrap_or(0);
        let key = if arg(a, 1).as_int() == Some(1) { "secondarycamo" } else { "primarycamo" };
        let bot = entnum(vm, s).is_some_and(|n| world.resource::<Zm>().bots.contains_key(&n));
        let chosen = his(vm, world, s, key).map_or(0, |p| p.loadout_item(class, key));
        // bo2mp: a bot wears a camo on half his guns, one of the first eight
        // (a camo in Black Ops II, Modern Warfare 2 and 3 alike), the same
        // for his class and slot all match.
        let camo = if bot && chosen == 0 {
            let n = entnum(vm, s).unwrap_or(0) as i32;
            let roll = (n * 7 + class * 3 + i32::from(key.starts_with('s')) * 5).rem_euclid(16);
            if roll < 8 { roll + 1 } else { 0 }
        } else {
            chosen
        };
        Ok(Value::Int(camo.clamp(0, 255)))
    });
    m!("getloadoutitem", |vm, world, s, a| {
        let class = arg(a, 0).as_int().unwrap_or(0);
        let slot = text(vm, a, 1);
        let class = if slot.starts_with("killstreak") { 0 } else { class };
        if let Some(p) = his(vm, world, s, &slot) {
            return Ok(Value::Int(p.loadout_item(class, &slot)));
        }
        let item = slot_item(vm, world, s, class, &slot);
        // A grenade slot's count rides on its own name ("<slot>count").
        if let Some(base) = slot.strip_suffix("count") {
            let n = slot_item(vm, world, s, class, base).map_or(0, |(_, w)| {
                w.first().and_then(|w| w.parse::<i32>().ok()).unwrap_or(1)
            });
            return Ok(Value::Int(n));
        }
        Ok(Value::Int(item.map_or(0, |(i, _)| i.index)))
    });
    m!("getloadoutitemref", |vm, world, s, a| {
        let class = arg(a, 0).as_int().unwrap_or(0);
        let slot = text(vm, a, 1);
        let class = if slot.starts_with("killstreak") { 0 } else { class };
        if let Some(p) = his(vm, world, s, &slot) {
            let r = p.loadout_item_ref(class, &slot);
            return Ok(vm.string(&r));
        }
        Ok(match slot_item(vm, world, s, class, &slot) {
            Some((it, _)) => vm.string(&it.reference),
            None => vm.string(""),
        })
    });
    // The class's perks as the engine gives them (setperk names): every
    // specialty a perk item names (`specialty_movefaster|specialty_fallheight`).
    m!("getloadoutperks", |vm, world, s, a| {
        let class = arg(a, 0).as_int().unwrap_or(0);
        if let Some(p) = his(vm, world, s, "specialty1") {
            let perks = p.loadout_perks(class);
            return Ok(list(perks.iter().map(|n| vm.string(n)).collect()));
        }
        let mut out = Vec::new();
        for (slot, it, _) in class_items(vm, world, s, class) {
            if slot.starts_with("specialty") {
                for p in it.reference.split('|').filter(|p| !p.is_empty()) {
                    out.push(vm.string(p));
                }
            }
        }
        Ok(list(out))
    });
    m!("isbonuscardactive", |vm, world, s, a| {
        let card = arg(a, 0).as_int().unwrap_or(-1);
        let class = arg(a, 1).as_int().unwrap_or(0);
        if let Some(p) = his(vm, world, s, "bonuscard1") {
            return Ok(Value::bool(p.bonus_card_active(card, class)));
        }
        Ok(Value::bool(
            class_items(vm, world, s, class)
                .iter()
                .any(|(slot, it, _)| slot.starts_with("bonuscard") && it.index == card),
        ))
    });
    // A gun's item number from its weapon name (`mp7_mp+reflex` -> MP7's).
    f!("getbaseweaponitemindex", |vm, world, _, a| {
        let w = text(vm, a, 0);
        let base = w.split('+').next().unwrap_or(&w);
        let reference = base.strip_suffix("_mp").unwrap_or(base);
        Ok(Value::Int(
            items(world)
                .iter()
                .find(|i| i.reference == reference)
                .map_or(0, |i| i.index),
        ))
    });
    f!("getreffromitemindex", |vm, world, _, a| {
        let Some(index) = arg(a, 0).as_int() else {
            return Ok(vm.string(""));
        };
        let r = items(world)
            .iter()
            .find(|i| i.index == index)
            .map(|i| i.reference.clone())
            .unwrap_or_default();
        Ok(vm.string(&r))
    });
    // A gun's attachments by its item name (`WEAPON_MK48`) or reference.
    f!("getweaponattachments", |vm, world, _, a| {
        let want = text(vm, a, 0);
        let atts = items(world)
            .iter()
            .find(|i| i.name.eq_ignore_ascii_case(&want) || i.reference == want)
            .map(|i| i.attachments.clone())
            .unwrap_or_default();
        let v = atts.iter().map(|s| vm.string(s)).collect();
        Ok(list(v))
    });
}

#[cfg(test)]
mod tests {
    use super::streak_list;

    /// A guest gun's class name against a script-names table shaped like
    /// the match's (index = weapon; BO2's rows, then the guests' with
    /// their variants as the catalog names them).
    #[test]
    fn guest_guns_take_their_variant_else_the_nearest() {
        let names: Vec<String> = [
            "none",
            "tar21_mp",
            "tar21_mp+reflex",
            "bo1_ak74u_mp",
            "bo1_ak74u_mp+reflex",
            "bo1_ak74u_mp+grip+reflex",
            "bo1_ak74u_mp+rf",
            "mw2_m4_mp",
            "mw2_m4_mp+eotech",
            "gh_ak12_mp",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
        let near = |n: &str| crate::world::nearest_guest_variant(&names, n, |_| true);
        // Exactly (any order), else the most of his and none other.
        assert_eq!(near("bo1_ak74u_mp+reflex+grip"), Some(5));
        assert_eq!(near("bo1_ak74u_mp+reflex+silencer"), Some(4));
        assert_eq!(near("bo1_ak74u_mp+extclip"), Some(3));
        assert_eq!(near("mw2_m4_mp+eotech+fmj"), Some(8));
        assert_eq!(near("gh_ak12_mp+reflex"), Some(9));
        // A variant that can't be given is passed over.
        assert_eq!(crate::world::nearest_guest_variant(&names, "bo1_ak74u_mp+grip+reflex", |w| w != 5), Some(4));
        assert_eq!(near("bo1_famas_mp+reflex"), None);
        for n in ["bo1_ak74u_mp+rf", "mw2_m4_mp", "gh_ak12_mp+acog", "cod4_m16_mp", "waw_mp40_mp+silencer"] {
            assert!(bo2_profile::guest::is_guest_name(n), "{n}");
        }
        assert!(!bo2_profile::guest::is_guest_name("tar21_mp+reflex"));
    }

    #[test]
    fn streak_column_lists_dearest_first_with_held() {
        // Pick order (as `player.killstreak` holds them) is not cost order.
        let picks = vec![
            ("hud_ks_qr_drone".to_owned(), 425, false),
            ("hud_ks_spy_sat".to_owned(), 350, true),
            ("hud_ks_straferun".to_owned(), 1000, false),
        ];
        assert_eq!(streak_list(picks), "hud_ks_straferun:1000:0;hud_ks_qr_drone:425:0;hud_ks_spy_sat:350:1");
        // Equal costs keep his pick order.
        let picks = vec![("a".to_owned(), 500, false), ("b".to_owned(), 500, true)];
        assert_eq!(streak_list(picks), "a:500:0;b:500:1");
        assert_eq!(streak_list(Vec::new()), "");
    }
}
