//! bo2mp shell: his classes for the shell's own Create-a-Class and Choose
//! Class screens.
//!
//! Read from his stats (the custom match class set, the one his matches
//! give: `bo2_profile::LOCAL_MATCH_SET`) through BO2's scripts' host each
//! frame they are wanted, and changed there: the front end saves them as
//! BO2's own Create-a-Class did.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bo2_profile::{LOCAL_MATCH_SET, Profile, SLOTS, StatValue};
use hks_t6::host::Host;
use hks_t6::mp::Mp;

/// The points a class may spend (Pick 10).
pub(crate) const POINTS: i32 = 10;

/// An item he can put in a class.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClassItem {
    pub index: i32,
    pub reference: String,
    pub name: String,
    pub image: String,
    pub desc: String,
    pub cost: i32,
    /// statstable's group (`weapon_smg`, `specialty`, ...).
    pub group: String,
    /// The class slot it goes in (`primary`, `specialty1`, `bonuscard`, ...).
    pub slot: String,
    /// The level it unlocks at (1 = from the start).
    pub level: i32,
    pub locked: bool,
    /// A weapon's attachments: (number, name, picture), 1 first.
    pub attachments: Vec<(i32, String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClassView {
    pub n: i32,
    pub name: String,
    /// Slot -> item index (or attachment number, or count).
    pub slots: HashMap<String, i32>,
    pub spent: i32,
}

impl ClassView {
    pub fn get(&self, slot: &str) -> i32 {
        self.slots.get(slot).copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ClassBook {
    pub classes: Vec<ClassView>,
    pub items: Vec<ClassItem>,
    /// bo2mp: Black Ops II's camos (mp/attachmenttable.csv's "camo" weapon
    /// options), in its order.
    pub camos: Vec<Camo>,
}

/// bo2mp: a camo he can put on a Black Ops II gun.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Camo {
    pub index: i32,
    pub name: String,
    pub image: String,
    /// One of the base game's camos (drawn in every match); the DLC camos'
    /// pictures come with their packs.
    pub drawn: bool,
    /// A material camo (Gold, Diamond, Carbon Fiber, the DLC camos), which
    /// other games' guns wear too; else a pattern laid on Black Ops II's
    /// own guns' camo-ready parts.
    pub material: bool,
}

impl ClassBook {
    pub fn camo(&self, index: i32) -> Option<&Camo> {
        (index > 0).then(|| self.camos.iter().find(|c| c.index == index)).flatten()
    }

    pub fn item(&self, index: i32) -> Option<&ClassItem> {
        (index > 0).then(|| self.items.iter().find(|i| i.index == index)).flatten()
    }

    /// The items that go in a class slot, best first (by group, then the
    /// table's order).
    pub fn for_slot(&self, slot: &str) -> Vec<&ClassItem> {
        let want = slot_kind(slot);
        let mut v: Vec<&ClassItem> = self.items.iter().filter(|i| slot_kind(&i.slot) == want).collect();
        v.sort_by_key(|i| (group_order(&i.group), i.level, i.index));
        v
    }
}

/// The kind of slot an item goes in: `specialty1`, `primary`, `bonuscard`
/// (wildcards, any of the three), `killstreak` (any of the three).
pub(crate) fn slot_kind(slot: &str) -> String {
    let s = slot.to_ascii_lowercase();
    if s.starts_with("bonuscard") {
        "bonuscard".to_owned()
    } else if s.starts_with("killstreak") {
        "killstreak".to_owned()
    } else if let Some(n) = s.strip_prefix("specialty") {
        // (Perk Greed's extra perks are 4-6: the same lists as 1-3.)
        match n.parse::<i32>() {
            Ok(k) if k > 3 => format!("specialty{}", k - 3),
            _ => s,
        }
    } else {
        s
    }
}

/// A weapon group's name, and its order.
pub(crate) fn group_label(group: &str) -> &'static str {
    match group {
        "weapon_smg" => "SUBMACHINE GUNS",
        "weapon_assault" => "ASSAULT RIFLES",
        "weapon_cqb" => "SHOTGUNS",
        "weapon_lmg" => "LIGHT MACHINE GUNS",
        "weapon_sniper" => "SNIPER RIFLES",
        "weapon_pistol" => "PISTOLS",
        "weapon_launcher" => "LAUNCHERS",
        "weapon_special" => "SPECIALS",
        "weapon_knife" => "MELEE",
        _ => "",
    }
}

fn group_order(group: &str) -> i32 {
    ["weapon_smg", "weapon_assault", "weapon_cqb", "weapon_lmg", "weapon_sniper", "weapon_pistol", "weapon_launcher", "weapon_special", "weapon_knife"]
        .iter()
        .position(|g| *g == group)
        .map_or(99, |p| p as i32)
}

/// A change he made: slots of a class and their new values.
#[derive(Clone, Debug)]
pub(crate) struct Edit {
    pub class: i32,
    pub set: Vec<(String, i32)>,
}

#[derive(Default)]
struct Shared {
    book: Option<Arc<ClassBook>>,
    edits: Vec<Edit>,
    renames: Vec<(i32, String)>,
    built_at: f64,
}

static SHARED: Mutex<Option<Shared>> = Mutex::new(None);
static WANTED: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut Shared) -> R) -> R {
    let mut g = SHARED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(g.get_or_insert_with(Shared::default))
}

/// A screen that shows his classes is up (they are read each half second
/// while it is).
pub(crate) fn want(on: bool) {
    WANTED.store(on, Ordering::Relaxed);
}

pub(crate) fn book() -> Option<Arc<ClassBook>> {
    with(|s| s.book.clone())
}

pub(crate) fn edit(e: Edit) {
    with(|s| s.edits.push(e));
}

/// The points a class spends (as BO2's Create-a-Class counts them: each
/// item its cost, a grenade each one carried, each attachment one).
pub(crate) fn spent(book: &ClassBook, slots: &HashMap<String, i32>) -> i32 {
    let mut total = 0;
    for slot in SLOTS {
        let n = slots.get(slot).copied().unwrap_or(0);
        if n <= 0 || slot.ends_with("count") || slot.ends_with("camo") || slot.ends_with("reticle") {
            continue;
        }
        if slot.contains("attachment") {
            total += 1;
        } else {
            let each = if slot.ends_with("grenade") { slots.get(&format!("{slot}count")).copied().unwrap_or(1).max(1) } else { 1 };
            total += book.item(n).map_or(0, |i| i.cost) * each;
        }
    }
    total
}

fn localized(v: &hks_t6::host::EngineValues, key: &str, fallback: &str) -> String {
    if !key.is_empty()
        && let Some(t) = v.localize.get(&key.to_ascii_uppercase()).filter(|t| !t.is_empty())
    {
        return t.clone();
    }
    fallback.replace('_', " ").to_uppercase()
}

/// Black Ops II's camos: mp/attachmenttable.csv's weapon options of the
/// "camo" group (column 0 its number, 3 its name, 6 its swatch), and
/// whether mp/weaponoptions.csv makes it a material camo (column 3 = 1).
fn camos(v: &hks_t6::host::EngineValues) -> Vec<Camo> {
    let material: std::collections::HashSet<i32> = hks_t6::mp::table_rows(v, "mp/weaponoptions.csv")
        .iter()
        .filter(|r| r.get(1).is_some_and(|c| c.trim() == "camo") && r.get(3).is_some_and(|c| c.trim() == "1"))
        .filter_map(|r| r.first()?.trim().parse().ok())
        .collect();
    hks_t6::mp::table_rows(v, "mp/attachmenttable.csv")
        .iter()
        .filter(|r| r.get(1).is_some_and(|c| c.trim() == "camo") && r.get(2).is_some_and(|c| c.trim() == "weaponoption"))
        .filter_map(|r| {
            let index: i32 = r.first()?.trim().parse().ok()?;
            let reference = r.get(4).map_or("", |c| c.trim());
            (index > 0).then(|| Camo {
                index,
                name: localized(v, r.get(3).map_or("", |c| c.trim()), reference.trim_start_matches("camo_")),
                image: r.get(6).map_or_else(String::new, |c| c.trim().to_owned()),
                drawn: index <= 18 || material.contains(&index),
                material: material.contains(&index),
            })
        })
        .collect()
}

fn build(host: &Host, mp: &Mp) -> ClassBook {
    let v = host.values.borrow();
    let m = mp.borrow();
    let p: &Profile = &m.profile;
    // (Locked by level unless he turned Unlock All on.)
    let free = p.stat_num(super::bridge::UNLOCK_ALL_STAT) != 0.0;
    let rank = p.rank();
    let mut items = Vec::new();
    for it in &p.items {
        if it.reference.is_empty() || it.slot.is_empty() {
            continue;
        }
        let kind = slot_kind(&it.slot);
        let wanted = matches!(kind.as_str(), "primary" | "secondary" | "primarygrenade" | "specialgrenade" | "bonuscard" | "killstreak")
            || kind.starts_with("specialty");
        if !wanted {
            continue;
        }
        let attachments = it
            .attachments
            .iter()
            .enumerate()
            .filter_map(|(k, _)| {
                let a = p.attachment(it.index, k as i32 + 1)?;
                Some((k as i32 + 1, localized(&v, &a.name, &a.reference), a.image.clone()))
            })
            .collect();
        items.push(ClassItem {
            index: it.index,
            reference: it.reference.clone(),
            name: localized(&v, &it.name, &it.reference),
            image: it.image.clone(),
            desc: localized(&v, &it.desc, ""),
            cost: it.cost,
            group: it.group.clone(),
            slot: it.slot.clone(),
            level: it.unlock_rank + 1,
            locked: !free && it.unlock_rank > rank,
            attachments,
        });
    }
    let mut classes = Vec::new();
    for n in 0..5 {
        let mut slots = HashMap::new();
        for slot in SLOTS.iter().copied().chain(["killstreak1", "killstreak2", "killstreak3"]) {
            let key = Profile::class_key(LOCAL_MATCH_SET, n, slot);
            if let Some(x) = p.stat(&key).and_then(StatValue::as_num) {
                slots.insert(slot.to_owned(), x as i32);
            }
        }
        let name = match p.stat(&format!("{LOCAL_MATCH_SET}.customclassname.{n}")) {
            Some(StatValue::Str(s)) if !s.trim().is_empty() => s.clone(),
            _ => format!("Custom {}", n + 1),
        };
        classes.push(ClassView { n, name, slots, spent: 0 });
    }
    let book_camos = camos(&v);
    let mut book = ClassBook { classes, items, camos: book_camos };
    let spends: Vec<i32> = book.classes.iter().map(|c| spent(&book, &c.slots)).collect();
    for (c, s) in book.classes.iter_mut().zip(spends) {
        c.spent = s;
    }
    book
}

/// Once a frame with BO2's scripts' host (front end or match): his edits
/// in, his classes out while a screen wants them.
pub(crate) fn class_side(host: &Host, mp: Option<&Mp>) {
    let Some(mp) = mp else { return };
    let edits = with(|s| std::mem::take(&mut s.edits));
    let renames = with(|s| std::mem::take(&mut s.renames));
    let changed = !edits.is_empty() || !renames.is_empty();
    if changed || WANTED.load(Ordering::Relaxed) {
        mp.borrow_mut().load(&host.values.borrow());
    }
    for e in edits {
        let mut m = mp.borrow_mut();
        for (slot, value) in e.set {
            let key = Profile::class_key(LOCAL_MATCH_SET, e.class, &slot);
            m.profile.set_stat(key, StatValue::Num(value as f32));
        }
        diag::info!(Ui, "bo2mp shell: class {} changed", e.class + 1);
    }
    for (n, name) in renames {
        mp.borrow_mut().profile.set_stat(format!("{LOCAL_MATCH_SET}.customclassname.{n}"), StatValue::Str(name));
    }
    let now = host.now_ms();
    let due = with(|s| changed || s.book.is_none() || now - s.built_at > 500.0 || now < s.built_at);
    if !(WANTED.load(Ordering::Relaxed) || changed) || !due {
        return;
    }
    let book = build(host, mp);
    with(|s| {
        if s.book.as_deref() != Some(&book) {
            s.book = Some(Arc::new(book));
        }
        s.built_at = now;
    });
}

/// The preview's stand-in classes.
pub(crate) fn set_preview_book(b: ClassBook) {
    with(|s| s.book = Some(Arc::new(b)));
}
