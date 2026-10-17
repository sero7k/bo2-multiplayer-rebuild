//! bo2mp shell: Create-a-Class - his five classes, a class's slots, and the
//! items (or attachments) a slot takes, in the shell's own look. Pick 10:
//! a class spends at most ten points; the wildcards open the extra slots
//! (Primary / Secondary Gunfighter, the Perk Greeds, Danger Close,
//! Overkill), as in Black Ops II.

use bevy::prelude::*;

use super::classes::{self, ClassBook, ClassItem, ClassView, Edit, POINTS, group_label, slot_kind};
use super::theme::*;
use super::view::{Ctx, Kids, abs, header, picture_contain};
use super::{Act, Hit, Row, Screen, Setting, Shell};

/// A class's slots as the edit screen lists them: (slot, label).
pub(crate) const EDIT_SLOTS: [(&str, &str); 23] = [
    ("primary", "PRIMARY"),
    ("primaryattachment1", "ATTACHMENT 1"),
    ("primaryattachment2", "ATTACHMENT 2"),
    ("primaryattachment3", "ATTACHMENT 3"),
    ("primarycamo", "CAMO"),
    ("secondary", "SECONDARY"),
    ("secondaryattachment1", "ATTACHMENT"),
    ("secondaryattachment2", "ATTACHMENT 2"),
    ("secondarycamo", "CAMO"),
    ("primarygrenade", "LETHAL"),
    ("specialgrenade", "TACTICAL"),
    ("specialty1", "PERK 1"),
    ("specialty4", "PERK 1 · GREED"),
    ("specialty2", "PERK 2"),
    ("specialty5", "PERK 2 · GREED"),
    ("specialty3", "PERK 3"),
    ("specialty6", "PERK 3 · GREED"),
    ("bonuscard1", "WILDCARD 1"),
    ("bonuscard2", "WILDCARD 2"),
    ("bonuscard3", "WILDCARD 3"),
    ("killstreak1", "SCORESTREAK 1"),
    ("killstreak2", "SCORESTREAK 2"),
    ("killstreak3", "SCORESTREAK 3"),
];

/// A wildcard of this class whose reference has `part`.
fn has_card(book: &ClassBook, c: &ClassView, part: &str) -> bool {
    (1..=3).any(|k| book.item(c.get(&format!("bonuscard{k}"))).is_some_and(|i| i.reference.contains(part)))
}

/// bo2mp: the camos a gun takes: Black Ops II's for its own guns, Modern
/// Warfare 2's or 3's for theirs (their slot numbers, their menu swatches);
/// (number, name, picture). None: no camos for this gun.
fn camo_choices(book: &ClassBook, gun: i32) -> Option<Vec<(i32, String, String)>> {
    use asset_game::weapon_camo_models::{MW2_CAMOS, MW3_CAMOS};
    if gun <= 0 {
        return None;
    }
    // (The mod's guns are Black Ops II weapons with camo sets of their own.)
    if gun < bo2_profile::guest::FIRST_INDEX || bo2_profile::guest::game_for_index(gun) == Some(bo2_profile::guest::MOD_GAME) {
        return (!book.camos.is_empty()).then(|| book.camos.iter().map(|k| (k.index, k.name.to_uppercase(), k.image.clone())).collect());
    }
    let list: &[(u8, &str, &str)] = match bo2_profile::guest::game_for_index(gun)? {
        "iw4" => &MW2_CAMOS,
        "iw5" => &MW3_CAMOS,
        // (The other games' guns have no camo models: Black Ops II's
        // material camos draw over them whole.)
        _ => {
            let list: Vec<_> = book
                .camos
                .iter()
                .filter(|k| k.material)
                .map(|k| (k.index, k.name.to_uppercase(), k.image.clone()))
                .collect();
            return (!list.is_empty()).then_some(list);
        }
    };
    let game = bo2_profile::guest::game_for_index(gun)?;
    Some(list.iter().map(|(n, name, pic)| (i32::from(*n), name.to_uppercase(), format!("{game}:{pic}"))).collect())
}

/// Whether the edit screen lists this slot for the class.
fn shown(book: &ClassBook, c: &ClassView, slot: &str) -> bool {
    match slot {
        "primaryattachment3" => has_card(book, c, "primary_gunfighter"),
        "secondaryattachment2" => has_card(book, c, "secondary_gunfighter"),
        "specialty4" => has_card(book, c, "perk_1_greed"),
        "specialty5" => has_card(book, c, "perk_2_greed"),
        "specialty6" => has_card(book, c, "perk_3_greed"),
        // bo2mp: Black Ops II's camos, on its own guns.
        "primarycamo" | "secondarycamo" => camo_choices(book, c.get(weapon_of(slot))).is_some(),
        s if s.starts_with("primaryattachment") => book.item(c.get("primary")).is_some_and(|i| !i.attachments.is_empty()),
        s if s.starts_with("secondaryattachment") => book.item(c.get("secondary")).is_some_and(|i| !i.attachments.is_empty()),
        _ => true,
    }
}

/// The slots the edit screen lists for the class, by index into
/// `EDIT_SLOTS`.
pub(crate) fn edit_slots(book: &ClassBook, c: &ClassView) -> Vec<usize> {
    (0..EDIT_SLOTS.len()).filter(|&i| shown(book, c, EDIT_SLOTS[i].0)).collect()
}

fn weapon_of(slot: &str) -> &str {
    if slot.starts_with("secondary") { "secondary" } else { "primary" }
}

/// What a slot holds, as text.
pub(crate) fn slot_value(book: &ClassBook, c: &ClassView, slot: &str) -> String {
    let v = c.get(slot);
    if slot.ends_with("camo") {
        return camo_choices(book, c.get(weapon_of(slot)))
            .and_then(|l| l.into_iter().find(|(n, ..)| *n == v))
            .map_or_else(|| "NONE".to_owned(), |(_, name, _)| name);
    }
    if slot.contains("attachment") {
        let weapon = book.item(c.get(weapon_of(slot)));
        return weapon
            .and_then(|w| w.attachments.iter().find(|(n, ..)| *n == v))
            .map_or_else(|| "NONE".to_owned(), |(_, name, _)| name.to_uppercase());
    }
    match book.item(v) {
        Some(i) if slot.ends_with("grenade") => {
            let count = c.get(&format!("{slot}count")).max(1);
            if count > 1 { format!("{} x{count}", i.name.to_uppercase()) } else { i.name.to_uppercase() }
        }
        Some(i) => i.name.to_uppercase(),
        None => "EMPTY".to_owned(),
    }
}

pub(crate) fn rows(s: &Shell, book: &ClassBook) -> Vec<Row> {
    match s.screen {
        Screen::Classes => book
            .classes
            .iter()
            .map(|c| Row {
                label: c.name.to_uppercase(),
                sub: format!("{} / {POINTS} points", c.spent),
                value: None,
                act: Act::EditClass(c.n),
            })
            .collect(),
        Screen::ClassEdit => {
            let Some(c) = book.classes.get(s.class as usize) else { return Vec::new() };
            let mut out = Vec::new();
            for i in edit_slots(book, c) {
                let (slot, label) = EDIT_SLOTS[i];
                out.push(Row { label: label.to_owned(), sub: String::new(), value: Some(slot_value(book, c, slot)), act: Act::EditSlot(i) });
                // A grenade's count right after it.
                if slot.ends_with("grenade") && c.get(slot) > 0 {
                    let lethal = slot == "primarygrenade";
                    out.push(Row {
                        label: if lethal { "LETHAL COUNT" } else { "TACTICAL COUNT" }.to_owned(),
                        sub: String::new(),
                        value: Some(c.get(&format!("{slot}count")).max(1).to_string()),
                        act: Act::Cycle(if lethal { Setting::LethalCount } else { Setting::TacticalCount }),
                    });
                }
            }
            out
        }
        Screen::ItemPick => picks(s, book)
            .1
            .iter()
            .enumerate()
            .map(|(k, p)| Row { label: p.name.clone(), sub: p.desc.clone(), value: None, act: Act::Choose(k) })
            .collect(),
        _ => Vec::new(),
    }
}

/// One choice in the item picker.
#[derive(Clone, Debug)]
pub(crate) struct Pick {
    pub value: i32,
    pub name: String,
    pub desc: String,
    pub image: String,
    pub cost: i32,
    pub locked: bool,
    pub level: i32,
    /// A weapon's kind (ASSAULT RIFLES...), shown on its tile.
    pub group: String,
}

/// The game a weapon comes from, as its tab in the picker: (order, name).
pub(crate) fn game_tab(index: i32) -> (usize, &'static str) {
    if index < bo2_profile::guest::FIRST_INDEX {
        return (0, "BLACK OPS II");
    }
    match bo2_profile::guest::game_for_index(index) {
        Some("t5") => (1, "BLACK OPS"),
        Some("t7") => (2, "BLACK OPS III"),
        Some("iw3") => (3, "COD 4"),
        Some("t4") => (4, "WORLD AT WAR"),
        Some("iw4") => (5, "MW2"),
        Some("iw5") => (6, "MW3"),
        Some("iw6") => (7, "GHOSTS"),
        Some("s1") => (8, "ADVANCED WARFARE"),
        Some("jup") => (9, "MWIII"),
        Some(bo2_profile::guest::MOD_GAME) => (10, "MOD"),
        _ => (11, "OTHER"),
    }
}

/// The picker's tabs (weapon groups) and the choices on the tab he is on.
pub(crate) fn picks(s: &Shell, book: &ClassBook) -> (Vec<String>, Vec<Pick>) {
    let Some(c) = book.classes.get(s.class as usize) else { return (Vec::new(), Vec::new()) };
    let slot = EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)].0;
    // bo2mp: Black Ops II's camos for the gun (Personalize Weapon's list).
    if slot.ends_with("camo") {
        let mut v = vec![Pick { value: 0, name: "NONE".to_owned(), desc: "The gun as it comes.".to_owned(), image: String::new(), cost: 0, locked: false, level: 1, group: String::new() }];
        for (n, name, image) in camo_choices(book, c.get(weapon_of(slot))).unwrap_or_default() {
            v.push(Pick {
                value: n,
                name,
                desc: "Worn on the gun in matches, first and third person.".to_owned(),
                image,
                cost: 0,
                locked: false,
                level: 1,
                group: String::new(),
            });
        }
        return (vec!["CAMOS".to_owned()], v);
    }
    if slot.contains("attachment") {
        let mut v = vec![Pick { value: 0, name: "NONE".to_owned(), desc: "No attachment.".to_owned(), image: String::new(), cost: 0, locked: false, level: 1, group: String::new() }];
        if let Some(w) = book.item(c.get(weapon_of(slot))) {
            for (n, name, image) in &w.attachments {
                v.push(Pick { value: *n, name: name.to_uppercase(), desc: String::new(), image: image.clone(), cost: 1, locked: false, level: 1, group: String::new() });
            }
        }
        return (vec!["ATTACHMENTS".to_owned()], v);
    }
    let mut items: Vec<&ClassItem> = book.for_slot(slot);
    // bo2mp: the mod's Zombies weapons are Custom Games only: not listed
    // while he sets up a public match (FIND A MATCH).
    if s.public {
        items.retain(|i| !bo2_profile::guest::is_custom_only_index(i.index));
    }
    // Overkill: a second primary as the secondary.
    if slot == "secondary" && has_card(book, c, "overkill") {
        items.extend(book.for_slot("primary"));
    }
    let weapons = matches!(slot_kind(slot).as_str(), "primary" | "secondary");
    // bo2mp: weapons by the game they come from (a tab each, Black Ops II's
    // own first), as Modern Warfare III's Create-a-Class lists its games'.
    let mut tabs: Vec<String> = Vec::new();
    if weapons {
        let mut games: Vec<(usize, &str)> = items.iter().map(|i| game_tab(i.index)).collect();
        games.sort_unstable();
        games.dedup();
        tabs = games.into_iter().map(|(_, t)| t.to_owned()).collect();
    }
    if tabs.is_empty() {
        tabs.push(EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)].1.split(" ·").next().unwrap_or("").to_owned() + "S");
    }
    let tab = s.page.min(tabs.len() - 1);
    let mut out = Vec::new();
    // (A slot that may be empty: NONE first.)
    if !matches!(slot, "primary") {
        out.push(Pick { value: 0, name: "NONE".to_owned(), desc: "Leave this slot empty.".to_owned(), image: String::new(), cost: 0, locked: false, level: 1, group: String::new() });
    }
    for i in items {
        if weapons && game_tab(i.index).1 != tabs[tab] {
            continue;
        }
        out.push(Pick {
            value: i.index,
            name: i.name.to_uppercase(),
            desc: i.desc.clone(),
            image: i.image.clone(),
            cost: i.cost,
            locked: i.locked,
            level: i.level,
            group: group_label(&i.group).to_owned(),
        });
    }
    if weapons && tab != 0 {
        out.retain(|p| p.value != 0);
    }
    (tabs, out)
}

/// The weapon picker's categories: the runs of one kind in its list
/// (start, end) (NONE, alone, first for a slot that may be empty).
pub(crate) fn categories(picks: &[Pick]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (i, p) in picks.iter().enumerate() {
        match out.last_mut() {
            Some((start, end)) if picks[*start].group == p.group => *end = i + 1,
            _ => out.push((i, i + 1)),
        }
    }
    out
}

/// Whether the slot open in the picker takes a weapon (the picker is then
/// a category at a time, as Black Ops II's).
pub(crate) fn weapon_picker(s: &Shell) -> bool {
    matches!(EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)].0, "primary" | "secondary")
}

/// A move in the weapon picker: left / right through the category's guns,
/// up / down to the first of the category before or after (round).
pub(crate) fn weapon_step(s: &Shell, book: &ClassBook, dir: (i32, i32)) -> Option<usize> {
    let picks = picks(s, book).1;
    let cats = categories(&picks);
    let at = cats.iter().position(|(a, b)| (*a..*b).contains(&s.focus))?;
    let (a, b) = cats[at];
    if dir.1 != 0 {
        let n = cats.len() as i32;
        return Some(cats[(at as i32 + dir.1).rem_euclid(n) as usize].0);
    }
    Some((s.focus as i32 + dir.0).clamp(a as i32, b as i32 - 1) as usize)
}

/// The picker opens on the tab and tile of what the slot holds.
pub(crate) fn open_pick(s: &mut Shell, book: &ClassBook, slot_index: usize) {
    s.slot = slot_index;
    s.screen = Screen::ItemPick;
    s.page = 0;
    s.focus = 0;
    let Some(c) = book.classes.get(s.class as usize) else { return };
    let have = c.get(EDIT_SLOTS[slot_index].0);
    let tabs = picks(s, book).0.len();
    for t in 0..tabs {
        s.page = t;
        if let Some(k) = picks(s, book).1.iter().position(|p| p.value == have) {
            s.focus = k;
            return;
        }
    }
    s.page = 0;
}

/// Put a choice in the slot (within the points); true when it went in.
pub(crate) fn choose(s: &mut Shell, book: &ClassBook, k: usize, now: f64) -> bool {
    let Some(c) = book.classes.get(s.class as usize) else { return false };
    let Some(p) = picks(s, book).1.get(k).cloned() else { return false };
    if p.locked {
        s.note = Some((format!("{} unlocks at level {}.", p.name, p.level), now + 3.0));
        return false;
    }
    let slot = EDIT_SLOTS[s.slot].0;
    let mut set: Vec<(String, i32)> = vec![(slot.to_owned(), p.value)];
    if slot == "primary" || slot == "secondary" {
        // A new weapon starts bare.
        if c.get(slot) != p.value {
            for k in 1..=3 {
                set.push((format!("{slot}attachment{k}"), 0));
            }
            set.push((format!("{slot}camo"), 0));
        }
    }
    if slot.ends_with("grenade") {
        set.push((format!("{slot}count"), if p.value > 0 { 1 } else { 0 }));
    }
    if slot.contains("attachment") && p.value > 0 {
        // The same attachment twice is once.
        let w = weapon_of(slot);
        for k in 1..=3 {
            let other = format!("{w}attachment{k}");
            if other != slot && c.get(&other) == p.value {
                set.push((other, 0));
            }
        }
    }
    if slot.starts_with("killstreak") && p.value > 0 {
        for k in 1..=3 {
            let other = format!("killstreak{k}");
            if other != slot && c.get(&other) == p.value {
                set.push((other, c.get(slot)));
            }
        }
    }
    apply(s, book, c, set, now)
}

/// A grenade count up or down (two of a lethal only with Danger Close).
pub(crate) fn count(s: &mut Shell, book: &ClassBook, lethal: bool, by: i32, now: f64) {
    let Some(c) = book.classes.get(s.class as usize) else { return };
    let slot = if lethal { "primarygrenade" } else { "specialgrenade" };
    let max = if !lethal || has_card(book, c, "danger_close") { 2 } else { 1 };
    let n = (c.get(&format!("{slot}count")).max(1) - 1 + by).rem_euclid(max) + 1;
    apply(s, book, c, vec![(format!("{slot}count"), n)], now);
}

fn apply(s: &mut Shell, book: &ClassBook, c: &ClassView, set: Vec<(String, i32)>, now: f64) -> bool {
    let mut slots = c.slots.clone();
    for (k, v) in &set {
        slots.insert(k.clone(), *v);
    }
    let spent = classes::spent(book, &slots);
    if spent > POINTS && spent > c.spent {
        s.note = Some((format!("Not enough points: that needs {spent} of {POINTS}."), now + 3.0));
        return false;
    }
    classes::edit(Edit { class: c.n, set });
    true
}

/// Points: ten bars, the spent ones lit, and the count.
fn points_bar(c: &mut Kids, cx: &Ctx, spent: i32, width: f32) {
    c.spawn((
        Node { width: cx.px(width), height: cx.px(34.0), column_gap: cx.px(5.0), align_items: AlignItems::Center, padding: UiRect::horizontal(cx.px(10.0)), ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
    ))
    .with_children(|b| {
        b.spawn((Text::new("POINTS:"), cx.regular(19.0), TextColor(TEXT), Node { margin: UiRect::right(cx.px(14.0)), ..default() }));
        for k in 0..POINTS {
            b.spawn((Node { flex_grow: 1.0, height: cx.px(5.0), ..default() }, BackgroundColor(if k < spent { WHITE } else { WHITE.with_alpha(0.18) })));
        }
        b.spawn((
            Node { margin: UiRect::left(cx.px(10.0)), ..default() },
            Text::new(format!("{spent}/{POINTS}")),
            cx.regular(19.0),
            TextColor(if spent > POINTS { ENEMY } else { WHITE }),
        ));
    });
}

/// Where the loadout's boxes answer to: the edit screen's row for a slot
/// (a click on the box opens it), and the slot in focus (lit).
struct Focus<'a> {
    rows: &'a [Row],
    focus_slot: Option<&'a str>,
}

impl Focus<'_> {
    fn row(&self, slot: &str) -> Option<usize> {
        let at = EDIT_SLOTS.iter().position(|(s, _)| *s == slot)?;
        self.rows.iter().position(|r| r.act == Act::EditSlot(at))
    }

    fn lit(&self, slot: &str) -> bool {
        self.focus_slot == Some(slot)
    }
}

/// A box of the loadout: dark, lit in the accent when its slot is in
/// focus, a click opening the slot (on the edit screen).
fn slot_box<'a>(c: &'a mut Kids, cx: &Ctx, f: &Focus, slot: &str, node: Node) -> EntityCommands<'a> {
    let lit = f.lit(slot);
    let mut e = c.spawn((
        Node { border: UiRect::all(cx.px(2.0)), ..node },
        BorderColor::all(if lit { cx.pal.accent } else { WHITE.with_alpha(0.07) }),
        tile_fill(cx, lit),
    ));
    if let Some(r) = f.row(slot) {
        e.insert((Hit::Row(r), Button));
    }
    e
}

/// A box's fill, as Black Ops II's: a dark grey that sinks to black toward
/// its foot (a little of the accent in it when lit).
fn tile_fill(cx: &Ctx, lit: bool) -> BackgroundGradient {
    let top = if lit { cx.pal.accent_a(0.16) } else { Color::srgba(0.16, 0.17, 0.19, 0.82) };
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::new(top, Val::Percent(0.0)),
        ColorStop::new(Color::srgba(0.03, 0.035, 0.045, 0.88), Val::Percent(100.0)),
    ]))
}

/// The small dash under a box (Infinite Warfare's: lit when filled).
fn dash(c: &mut Kids, cx: &Ctx, filled: bool) {
    c.spawn((
        Node { position_type: PositionType::Absolute, right: cx.px(10.0), bottom: cx.px(6.0), width: cx.px(28.0), height: cx.px(3.0), ..default() },
        BackgroundColor(WHITE.with_alpha(if filled { 0.85 } else { 0.25 })),
    ));
}

/// A weapon's panel: its slot's name, the gun's name and picture, then its
/// attachment and camo boxes.
#[allow(clippy::too_many_arguments)]
fn weapon_panel(c: &mut Kids, cx: &mut Ctx, book: &ClassBook, class: &ClassView, f: &Focus, weapon: &str, title: &str, width: f32) {
    let gun = book.item(class.get(weapon));
    let attach_slots: Vec<String> = (1..=3)
        .map(|k| format!("{weapon}attachment{k}"))
        .filter(|s| shown(book, class, s) || class.get(s) > 0)
        .chain(std::iter::once(format!("{weapon}camo")).filter(|s| shown(book, class, s)))
        .collect();
    c.spawn((Node { width: cx.px(width), flex_direction: FlexDirection::Column, ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.50)))).with_children(|p| {
        p.spawn((Node { padding: UiRect::axes(cx.px(10.0), cx.px(4.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))))
            .with_child((Text::new(title), cx.bold(18.0), TextColor(TEXT)));
        p.spawn(Node { column_gap: cx.px(4.0), padding: UiRect::all(cx.px(4.0)), ..default() }).with_children(|row| {
            let main_w = width * 0.52;
            slot_box(row, cx, f, weapon, Node { width: cx.px(main_w), height: cx.px(166.0), flex_direction: FlexDirection::Column, padding: UiRect::all(cx.px(8.0)), ..default() })
                .with_children(|b| {
                    b.spawn((Text::new(gun.map_or_else(|| "EMPTY".to_owned(), |g| g.name.to_uppercase())), cx.regular(21.0), TextColor(WHITE)));
                    let names = gun.map(|g| vec![g.image.clone()]).unwrap_or_default();
                    b.spawn(Node { flex_grow: 1.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() })
                        .with_children(|pic| picture_contain(pic, cx, &names, main_w - 30.0, 110.0));
                    dash(b, cx, gun.is_some());
                });
            let cell = 78.0;
            row.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: cx.px(4.0), row_gap: cx.px(4.0), width: cx.px(width - main_w - 12.0), align_content: AlignContent::FlexStart, ..default() })
                .with_children(|grid| {
                    for slot in &attach_slots {
                        let value = class.get(slot);
                        let pic = if slot.ends_with("camo") {
                            camo_choices(book, class.get(weapon)).and_then(|l| l.into_iter().find(|(n, ..)| *n == value)).map(|(_, _, p)| p)
                        } else {
                            gun.and_then(|g| g.attachments.iter().find(|(n, ..)| *n == value)).map(|(_, _, p)| p.clone())
                        };
                        slot_box(grid, cx, f, slot, Node { width: cx.px(cell), height: cx.px(cell), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() })
                            .with_children(|b| {
                                match pic.filter(|p| !p.is_empty() && value > 0) {
                                    Some(p) => picture_contain(b, cx, &[p], cell - 18.0, cell - 18.0),
                                    None => {
                                        // (An empty slot: a ring with a plus.)
                                        b.spawn((
                                            Node { width: cx.px(28.0), height: cx.px(28.0), border: UiRect::all(cx.px(2.0)), border_radius: BorderRadius::all(cx.px(14.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                            BorderColor::all(WHITE.with_alpha(0.45)),
                                        ))
                                        .with_child((Text::new("+"), cx.regular(20.0), TextColor(WHITE.with_alpha(0.6))));
                                    }
                                }
                                dash(b, cx, value > 0);
                            });
                    }
                });
        });
    });
}

/// A small box: its label at the top, the item's picture, a line under it.
#[allow(clippy::too_many_arguments)]
fn small_box(c: &mut Kids, cx: &mut Ctx, book: &ClassBook, class: &ClassView, f: &Focus, slot: &str, label: &str, extra: &str, w: f32) {
    let item = book.item(class.get(slot));
    slot_box(c, cx, f, slot, Node { width: cx.px(w), height: cx.px(120.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(cx.px(6.0)), ..default() })
        .with_children(|b| {
            b.spawn((Text::new(label), cx.regular(17.0), TextColor(TEXT)));
            let names = item.map(|i| vec![i.image.clone()]).unwrap_or_default();
            b.spawn(Node { flex_grow: 1.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() })
                .with_children(|pic| picture_contain(pic, cx, &names, w - 30.0, 58.0));
            b.spawn((Text::new(item.map_or_else(String::new, |i| i.name.to_uppercase()) + extra), cx.regular(14.0), TextColor(DIM)));
            dash(b, cx, item.is_some());
        });
}

/// The loadout, as Infinite Warfare's: the points, the primary's and the
/// secondary's panels, then lethal, tactical and the three perks, then the
/// wildcards (each box lit when its slot is in focus).
fn loadout(c: &mut Kids, cx: &mut Ctx, book: &ClassBook, class: &ClassView, f: &Focus, left: f32, top: f32, width: f32) {
    c.spawn(Node { width: cx.px(width), flex_direction: FlexDirection::Column, row_gap: cx.px(4.0), ..abs(Some(left), Some(top), None, None, cx) }).with_children(|col| {
        points_bar(col, cx, class.spent, width);
        weapon_panel(col, cx, book, class, f, "primary", "PRIMARY WEAPON", width);
        weapon_panel(col, cx, book, class, f, "secondary", "SECONDARY WEAPON", width);
        let count = |slot: &str| {
            let n = class.get(&format!("{slot}count"));
            if n > 1 { format!(" x{n}") } else { String::new() }
        };
        let fifth = (width - 16.0) / 5.0;
        col.spawn(Node { column_gap: cx.px(4.0), ..default() }).with_children(|row| {
            small_box(row, cx, book, class, f, "primarygrenade", "LETHAL", &count("primarygrenade"), fifth);
            small_box(row, cx, book, class, f, "specialgrenade", "TACTICAL", &count("specialgrenade"), fifth);
            for k in 1..=3 {
                // (Perk Greed's extra perk under its perk.)
                let greed = format!("specialty{}", k + 3);
                let slot = if f.lit(&greed) { greed } else { format!("specialty{k}") };
                let extra = book.item(class.get(&format!("specialty{}", k + 3))).filter(|_| !slot.ends_with(&(k + 3).to_string())).map(|i| format!(" + {}", i.name.to_uppercase())).unwrap_or_default();
                small_box(row, cx, book, class, f, &slot, &format!("PERK {k}"), &extra, fifth);
            }
        });
        let third = (width - 8.0) / 3.0;
        col.spawn(Node { column_gap: cx.px(4.0), ..default() }).with_children(|row| {
            for k in 1..=3 {
                small_box(row, cx, book, class, f, &format!("bonuscard{k}"), &format!("WILDCARD {k}"), "", third);
            }
        });
    });
}

/// A class at a glance (the in-game Choose Class shows it too).
pub(crate) fn class_card(c: &mut Kids, cx: &mut Ctx, book: &ClassBook, class: &ClassView, left: f32, top: f32, width: f32) {
    loadout(c, cx, book, class, &Focus { rows: &[], focus_slot: None }, left, top, width.min(640.0));
}

/// A dark strip with a name (the left column's headings).
fn strip(c: &mut Kids, cx: &Ctx, text: &str, width: f32) {
    c.spawn((Node { width: cx.px(width), height: cx.px(38.0), align_items: AlignItems::Center, padding: UiRect::horizontal(cx.px(14.0)), border: UiRect::left(cx.px(6.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)), BorderColor::all(WHITE)))
        .with_child((Text::new(text.to_owned()), cx.regular(23.0), TextColor(WHITE)));
}

/// The class's scorestreaks: their pictures side by side, their names under.
fn streaks(c: &mut Kids, cx: &mut Ctx, book: &ClassBook, class: &ClassView, width: f32) {
    strip(c, cx, "SCORESTREAKS", width);
    let third = (width - 8.0) / 3.0;
    c.spawn(Node { column_gap: cx.px(4.0), ..default() }).with_children(|row| {
        for k in 1..=3 {
            let item = book.item(class.get(&format!("killstreak{k}")));
            row.spawn((Node { width: cx.px(third), height: cx.px(92.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: cx.px(2.0), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))))
                .with_children(|b| {
                    let names = item.map(|i| vec![i.image.clone()]).unwrap_or_default();
                    picture_contain(b, cx, &names, third - 40.0, 50.0);
                    b.spawn((Text::new(item.map_or_else(|| "—".to_owned(), |i| i.name.to_uppercase())), cx.regular(15.0), TextColor(TEXT)));
                });
        }
    });
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, s: &Shell, book: &ClassBook, rows: &[Row]) {
    let (left, col_w, mid) = (126.0, 400.0, 560.0);
    match s.screen {
        Screen::Classes => {
            header(c, cx, &[super::view::section(s), "LOADOUT SELECT"]);
            let class = book.classes.get(s.focus);
            c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(4.0), ..abs(Some(left), Some(196.0), None, None, cx) }).with_children(|col| {
                strip(col, cx, "SELECT CLASS", col_w);
                let shown: Vec<Row> = rows.iter().map(|r| Row { sub: String::new(), ..r.clone() }).collect();
                super::view::menu_rows(col, cx, &shown, s.focus, col_w - 21.0, false);
                if let Some(class) = class {
                    col.spawn(Node { height: cx.px(10.0), ..default() });
                    streaks(col, cx, book, class, col_w);
                    col.spawn((Text::new(format!("{} / {POINTS} points.  Create a loadout of weapons, equipment and perks to bring into battle.", class.spent)), cx.regular(21.0), TextColor(WHITE), Node { max_width: cx.px(col_w), margin: UiRect::top(cx.px(8.0)), ..default() }));
                }
            });
            if let Some(class) = class {
                loadout(c, cx, book, class, &Focus { rows: &[], focus_slot: None }, mid, 196.0, 640.0);
            }
        }
        Screen::ClassEdit => {
            let Some(class) = book.classes.get(s.class as usize) else { return };
            edit_view(c, cx, s, book, class, rows);
        }
        Screen::ItemPick => {
            let Some(class) = book.classes.get(s.class as usize) else { return };
            pick_view(c, cx, s, book, class);
        }
        _ => {}
    }
}

/// The edit screen, laid out as Black Ops II's Pick 10 class: the class's
/// name and its ten points across the top; the primary and the secondary
/// as large panels with their attachment boxes under each at the left; the
/// perks, lethal and tactical down the right (name, then the item's box);
/// the wildcards and scorestreaks along the foot. The box of the slot in
/// focus is lit; a click on any box opens its slot.
fn edit_view(c: &mut Kids, cx: &mut Ctx, s: &Shell, book: &ClassBook, class: &ClassView, rows: &[Row]) {
    let focus_slot = rows.get(s.focus).and_then(|r| match r.act {
        Act::EditSlot(i) => Some(EDIT_SLOTS[i].0),
        Act::Cycle(Setting::LethalCount) => Some("primarygrenadecount"),
        Act::Cycle(Setting::TacticalCount) => Some("specialgrenadecount"),
        _ => None,
    });
    let f = Focus { rows, focus_slot };
    let count_row = |what: Setting| rows.iter().position(|r| r.act == Act::Cycle(what));
    let (l, lw, r, rw) = (126.0, 900.0, 1050.0, 744.0);
    // The name, the points.
    pick10_top(c, cx, &class.name, class.spent);
    // The weapons.
    c.spawn(Node { width: cx.px(lw), flex_direction: FlexDirection::Column, row_gap: cx.px(10.0), ..abs(Some(l), Some(176.0), None, None, cx) }).with_children(|col| {
        let second = if has_card(book, class, "overkill") { "SECOND PRIMARY" } else { "SECONDARY" };
        for (weapon, label, h) in [("primary", "PRIMARY", 240.0), ("secondary", second, 130.0)] {
            let gun = book.item(class.get(weapon));
            let lit = f.lit(weapon);
            slot_box(col, cx, &f, weapon, Node { width: cx.px(lw), height: cx.px(h), ..default() }).with_children(|b| {
                b.spawn((Text::new(label), cx.regular(26.0), TextColor(DIM), abs(Some(12.0), Some(6.0), None, None, cx)));
                let names = gun.map(|g| vec![g.image.clone()]).unwrap_or_default();
                b.spawn(Node { width: cx.px(lw * 0.55), height: cx.px(h - 30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..abs(Some(lw * 0.30), Some(14.0), None, None, cx) })
                    .with_children(|pic| picture_contain(pic, cx, &names, lw * 0.55, h - 40.0));
                b.spawn((
                    Text::new(gun.map_or_else(|| "EMPTY".to_owned(), |g| g.name.clone())),
                    cx.bold(if h > 200.0 { 46.0 } else { 38.0 }),
                    TextColor(if lit { cx.pal.accent } else if gun.is_some() { WHITE } else { DIM }),
                    abs(Some(14.0), None, None, Some(8.0), cx),
                ));
                // Its camo's swatch at the bottom right.
                let camo = format!("{weapon}camo");
                if shown(book, class, &camo) && class.get(&camo) > 0 {
                    let pic = camo_choices(book, class.get(weapon)).and_then(|v| v.into_iter().find(|(n, ..)| *n == class.get(&camo))).map(|(_, _, p)| p);
                    if let Some(p) = pic {
                        b.spawn(abs(None, None, Some(14.0), Some(14.0), cx)).with_children(|k| picture_contain(k, cx, &[p], 64.0, 64.0));
                    }
                }
            });
            // The attachments (and camo) under it.
            col.spawn(Node { width: cx.px(lw), height: cx.px(116.0), column_gap: cx.px(12.0), align_items: AlignItems::Center, ..default() }).with_children(|row| {
                row.spawn((Node { width: cx.px(300.0), ..default() },)).with_child((Text::new("ATTACHMENTS"), cx.regular(26.0), TextColor(DIM)));
                let slots: Vec<String> = (1..=3)
                    .map(|k| format!("{weapon}attachment{k}"))
                    .filter(|x| shown(book, class, x))
                    .chain(std::iter::once(format!("{weapon}camo")).filter(|x| shown(book, class, x)))
                    .collect();
                for slot in &slots {
                    let value = class.get(slot);
                    let pic = if slot.ends_with("camo") {
                        camo_choices(book, class.get(weapon)).and_then(|v| v.into_iter().find(|(n, ..)| *n == value)).map(|(_, _, p)| p)
                    } else {
                        gun.and_then(|g| g.attachments.iter().find(|(n, ..)| *n == value)).map(|(_, _, p)| p.clone())
                    };
                    let name = slot_value(book, class, slot);
                    slot_box(row, cx, &f, slot, Node { width: cx.px(140.0), height: cx.px(108.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: cx.px(2.0), ..default() })
                        .with_children(|b| {
                            match pic.filter(|p| !p.is_empty() && value > 0) {
                                Some(p) => picture_contain(b, cx, &[p], 96.0, 64.0),
                                None => {
                                    b.spawn((Text::new(if slot.ends_with("camo") { "CAMO" } else { "+" }), cx.regular(if slot.ends_with("camo") { 18.0 } else { 34.0 }), TextColor(WHITE.with_alpha(0.35))));
                                }
                            }
                            if value > 0 {
                                b.spawn((Text::new(name), cx.regular(14.0), TextColor(TEXT)));
                            }
                        });
                }
            });
        }
    });
    // Perks, lethal, tactical.
    c.spawn(Node { width: cx.px(rw), flex_direction: FlexDirection::Column, row_gap: cx.px(10.0), ..abs(Some(r), Some(176.0), None, None, cx) }).with_children(|col| {
        let mut lines: Vec<(String, String, Option<String>, Option<usize>)> = Vec::new();
        for k in 1..=3 {
            let greed = format!("specialty{}", k + 3);
            lines.push((format!("specialty{k}"), format!("PERK {k}"), shown(book, class, &greed).then_some(greed), None));
        }
        lines.push(("primarygrenade".to_owned(), "LETHAL".to_owned(), None, count_row(Setting::LethalCount)));
        lines.push(("specialgrenade".to_owned(), "TACTICAL".to_owned(), None, count_row(Setting::TacticalCount)));
        for (slot, label, greed, count) in lines {
            let item = book.item(class.get(&slot));
            col.spawn(Node { height: cx.px(112.0), column_gap: cx.px(12.0), align_items: AlignItems::Center, ..default() }).with_children(|line| {
                line.spawn(Node { width: cx.px(300.0), flex_direction: FlexDirection::Column, ..default() }).with_children(|t| {
                    t.spawn((Text::new(label.clone()), cx.regular(26.0), TextColor(DIM)));
                    let mut name = item.map_or_else(|| "Empty".to_owned(), |i| i.name.clone());
                    if let Some(g) = greed.as_deref().and_then(|g| book.item(class.get(g))) {
                        name = format!("{name} + {}", g.name);
                    }
                    t.spawn((Text::new(name), cx.regular(30.0), TextColor(if f.lit(&slot) { cx.pal.accent } else { WHITE })));
                });
                for box_slot in std::iter::once(slot.clone()).chain(greed.clone()) {
                    let it = book.item(class.get(&box_slot));
                    slot_box(line, cx, &f, &box_slot, Node { width: cx.px(150.0), height: cx.px(108.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() })
                        .with_children(|b| match it {
                            Some(i) => picture_contain(b, cx, &[i.image.clone()], 100.0, 90.0),
                            None => {
                                b.spawn((Text::new("+"), cx.regular(34.0), TextColor(WHITE.with_alpha(0.35))));
                            }
                        });
                }
                // A grenade's count: a box of its own (A / D, or a click,
                // changes it).
                if let Some(row) = count {
                    let n = class.get(&format!("{slot}count")).max(1);
                    let lit = f.lit(&format!("{slot}count"));
                    line.spawn((
                        Hit::Row(row),
                        Button,
                        Node { width: cx.px(84.0), height: cx.px(84.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(cx.px(if lit { 2.0 } else { 1.0 })), ..default() },
                        BorderColor::all(if lit { cx.pal.accent } else { WHITE.with_alpha(0.25) }),
                        BackgroundColor(if lit { cx.pal.accent_a(0.30) } else { Color::srgba(0.12, 0.13, 0.15, 0.72) }),
                    ))
                    .with_child((Text::new(if n > 1 { format!("x{n}") } else { "+".to_owned() }), cx.bold(34.0), TextColor(WHITE)));
                }
            });
        }
    });
    // Wildcards and scorestreaks along the foot.
    let foot = 836.0;
    c.spawn(Node { width: cx.px(lw), flex_direction: FlexDirection::Column, row_gap: cx.px(6.0), ..abs(Some(l), Some(foot), None, None, cx) }).with_children(|col| {
        col.spawn((Text::new("WILDCARDS"), cx.regular(26.0), TextColor(DIM)));
        col.spawn(Node { column_gap: cx.px(12.0), ..default() }).with_children(|row| {
            for k in 1..=3 {
                let slot = format!("bonuscard{k}");
                footer_box(row, cx, &f, book, class, &slot, (lw - 24.0) / 3.0);
            }
        });
    });
    c.spawn(Node { width: cx.px(rw), flex_direction: FlexDirection::Column, row_gap: cx.px(6.0), ..abs(Some(r), Some(foot), None, None, cx) }).with_children(|col| {
        col.spawn((Text::new("SCORESTREAKS"), cx.regular(26.0), TextColor(DIM)));
        col.spawn(Node { column_gap: cx.px(12.0), ..default() }).with_children(|row| {
            for k in 1..=3 {
                let slot = format!("killstreak{k}");
                footer_box(row, cx, &f, book, class, &slot, (rw - 24.0) / 3.0);
            }
        });
    });
}

/// A wildcard's or scorestreak's box: its name over its picture.
fn footer_box(c: &mut Kids, cx: &mut Ctx, f: &Focus, book: &ClassBook, class: &ClassView, slot: &str, w: f32) {
    let item = book.item(class.get(slot));
    let lit = f.lit(slot);
    slot_box(c, cx, f, slot, Node { width: cx.px(w), height: cx.px(110.0), padding: UiRect::all(cx.px(10.0)), column_gap: cx.px(8.0), justify_content: JustifyContent::SpaceBetween, ..default() })
        .with_children(|b| {
            b.spawn((Text::new(item.map_or_else(String::new, |i| i.name.clone())), cx.regular(24.0), TextColor(if lit { cx.pal.accent } else { WHITE }), Node { max_width: cx.px(w * 0.55), ..default() }));
            let names = item.map(|i| vec![i.image.clone()]).unwrap_or_default();
            if names.is_empty() {
                b.spawn((Text::new("+"), cx.regular(34.0), TextColor(WHITE.with_alpha(0.35))));
            } else {
                picture_contain(b, cx, &names, w * 0.38, 88.0);
            }
        });
}

/// The class's name or the picker's title at the top left and its points
/// across the top, as Black Ops II's Pick 10 screens.
fn pick10_top(c: &mut Kids, cx: &mut Ctx, title: &str, spent: i32) {
    let (l, r) = (126.0, 1794.0);
    c.spawn((Text::new(title.to_owned()), cx.regular(68.0), TextColor(WHITE), abs(Some(l - 4.0), Some(40.0), None, None, cx)));
    c.spawn((Text::new(format!("{spent}/{POINTS}")), cx.bold(54.0), TextColor(if spent > POINTS { ENEMY } else { cx.pal.accent }), abs(None, Some(64.0), Some(cx.w - r), None, cx)));
    c.spawn(Node { width: cx.px(r - l), height: cx.px(12.0), column_gap: cx.px(8.0), ..abs(Some(l), Some(140.0), None, None, cx) }).with_children(|b| {
        for k in 0..POINTS {
            b.spawn((Node { flex_grow: 1.0, ..default() }, BackgroundColor(if k < spent { cx.pal.accent } else { cx.pal.accent_a(0.22) })));
        }
    });
}

/// A slot's picker, as Black Ops II's: its title and the points, a grid of
/// tiles (each the item's picture, its name on a dark strip at the foot, a
/// lock or LV for one not unlocked, EQUIPPED for the one in the class), then
/// the one in focus large with what it does; for attachments and camos the
/// gun at the bottom right. Weapons are in a tab per game (top right).
fn pick_view(c: &mut Kids, cx: &mut Ctx, s: &Shell, book: &ClassBook, class: &ClassView) {
    if weapon_picker(s) {
        return weapon_view(c, cx, s, book, class);
    }
    let pal = cx.pal;
    let (slot, label) = EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)];
    let gun = book.item(class.get(weapon_of(slot)));
    let gun_name = gun.map_or_else(String::new, |g| g.name.to_uppercase());
    let small = slot.contains("attachment") || slot.ends_with("camo");
    let title = match slot {
        "primary" => "PRIMARY".to_owned(),
        "secondary" if has_card(book, class, "overkill") => "SECOND PRIMARY".to_owned(),
        "secondary" => "SECONDARY".to_owned(),
        "primarygrenade" => "LETHALS".to_owned(),
        "specialgrenade" => "TACTICALS".to_owned(),
        x if x.contains("attachment") => format!("{gun_name} ATTACHMENTS"),
        x if x.ends_with("camo") => format!("PERSONALIZE {gun_name}"),
        _ => label.split(" ·").next().unwrap_or(label).to_owned(),
    };
    pick10_top(c, cx, &title, class.spent);
    let (tabs, picks) = picks(s, book);
    let tab = s.page.min(tabs.len().saturating_sub(1));
    if tabs.len() > 1 {
        game_tabs(c, cx, &tabs, tab);
    }
    let have = class.get(slot);
    let (l, width) = (126.0, 1668.0);
    let (cols, gap, th) = if small { (6usize, 12.0, 156.0) } else { (3usize, 14.0, 196.0) };
    let tw = (width - gap * (cols as f32 - 1.0)) / cols as f32;
    // (Rows that fit above the words at the foot.)
    let fit_rows = ((640.0 + gap) / (th + gap)).floor().max(1.0) as usize;
    let focus_row = s.focus / cols;
    let first_row = focus_row.saturating_sub(fit_rows - 1);
    super::GRID_COLS.store(cols, std::sync::atomic::Ordering::Relaxed);
    let (first, last) = (first_row * cols, ((first_row + fit_rows) * cols).min(picks.len()));
    c.spawn(Node { width: cx.px(width), flex_wrap: FlexWrap::Wrap, column_gap: cx.px(gap), row_gap: cx.px(gap), ..abs(Some(l), Some(178.0), None, None, cx) }).with_children(|grid| {
        for (k, p) in picks.iter().enumerate().take(last).skip(first) {
            let on = k == s.focus;
            grid.spawn((
                Hit::Row(k),
                Button,
                Node { width: cx.px(tw), height: cx.px(th), flex_direction: FlexDirection::Column, border: UiRect::all(cx.px(2.0)), ..default() },
                BorderColor::all(if on { pal.accent } else { WHITE.with_alpha(0.07) }),
                tile_fill(cx, on),
            ))
            .with_children(|tile| {
                tile.spawn(Node { flex_grow: 1.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }).with_children(|pic| {
                    if p.locked {
                        lock(pic, cx);
                    } else if p.image.is_empty() {
                        // (NONE: a ring with a stroke through it.)
                        pic.spawn((
                            Node { width: cx.px(64.0), height: cx.px(64.0), border: UiRect::all(cx.px(4.0)), border_radius: BorderRadius::all(cx.px(32.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BorderColor::all(WHITE.with_alpha(0.35)),
                        ))
                        .with_child((Node { width: cx.px(4.0), height: cx.px(60.0), ..default() }, BackgroundColor(WHITE.with_alpha(0.35)), UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY }));
                    } else {
                        picture_contain(pic, cx, std::slice::from_ref(&p.image), tw * 0.55, th - 64.0);
                    }
                });
                tile.spawn((
                    Node { height: cx.px(if small { 40.0 } else { 46.0 }), padding: UiRect::horizontal(cx.px(10.0)), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.70)),
                ))
                .with_children(|bar| {
                    bar.spawn((Text::new(title_case(&p.name)), cx.regular(if small { 22.0 } else { 28.0 }), TextColor(if on { pal.accent } else if p.locked { DIM } else { WHITE })));
                    let tag = if p.locked {
                        format!("LV {}", p.level)
                    } else if p.value == have && p.value != 0 {
                        "EQUIPPED".to_owned()
                    } else {
                        String::new()
                    };
                    if !tag.is_empty() {
                        bar.spawn((Text::new(tag), cx.bold(16.0), TextColor(if p.locked { GOLD } else { pal.accent })));
                    }
                });
                if !p.group.is_empty() {
                    tile.spawn((Text::new(p.group.clone()), cx.regular(15.0), TextColor(DIM), abs(Some(10.0), Some(6.0), None, None, cx)));
                }
            });
        }
    });
    // The one in focus, large, and what it does.
    let foot = 178.0 + fit_rows as f32 * (th + gap) + 30.0;
    super::view::grid_more(c, cx, first, last, picks.len(), l + width - 200.0, foot - 26.0);
    if let Some(p) = picks.get(s.focus) {
        c.spawn(Node { width: cx.px(940.0), flex_direction: FlexDirection::Column, row_gap: cx.px(4.0), ..abs(Some(l), Some(foot), None, None, cx) }).with_children(|t| {
            t.spawn((Text::new(title_case(&p.name)), cx.regular(64.0), TextColor(WHITE)));
            let line = if p.locked { format!("Unlocks at level {}.", p.level) } else { p.desc.clone() };
            if !line.is_empty() {
                t.spawn((Text::new(line), cx.regular(28.0), TextColor(DIM)));
            }
            if p.cost > 0 && !p.locked {
                t.spawn((Text::new(format!("{} POINT{}", p.cost, if p.cost == 1 { "" } else { "S" })), cx.regular(20.0), TextColor(pal.accent)));
            }
        });
    }
    if small && let Some(g) = gun {
        c.spawn(abs(None, None, Some(126.0), Some(110.0), cx)).with_children(|b| picture_contain(b, cx, &[g.image.clone()], 520.0, 200.0));
    }
}

/// The weapon picker, as Black Ops II's: the slot's title with the games as
/// a toggle under it (Q / E) and the points; one category at a time, the
/// gun in focus a large tile and the rest of its category smaller beside
/// it; under them a dot per gun, what the gun in focus is, and the
/// categories (this one large in the accent, the rest after it).
fn weapon_view(c: &mut Kids, cx: &mut Ctx, s: &Shell, book: &ClassBook, class: &ClassView) {
    let pal = cx.pal;
    let slot = EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)].0;
    let title = match slot {
        "secondary" if has_card(book, class, "overkill") => "SECOND PRIMARY",
        "secondary" => "SECONDARY",
        _ => "PRIMARY",
    };
    let (l, r) = (126.0, 1794.0);
    // The title, the games' toggle, the points.
    c.spawn((Text::new(class.name.to_uppercase()), cx.regular(20.0), TextColor(TEXT), abs(Some(l), Some(22.0), None, None, cx)));
    c.spawn((Text::new(title), cx.regular(60.0), TextColor(WHITE), abs(Some(l - 3.0), Some(40.0), None, None, cx)));
    let (tabs, picks) = picks(s, book);
    let tab = s.page.min(tabs.len().saturating_sub(1));
    c.spawn(Node { column_gap: cx.px(30.0), align_items: AlignItems::FlexEnd, ..abs(Some(l), Some(100.0), None, None, cx) }).with_children(|t| {
        if tabs.len() > 1 {
            super::view::hint(t, cx, "Q", "xenonbutton_lb", "");
        }
        for (i, name) in tabs.iter().enumerate() {
            let sel = i == tab;
            t.spawn((Hit::Page(i), Button, Node { flex_direction: FlexDirection::Column, row_gap: cx.px(3.0), ..default() })).with_children(|b| {
                b.spawn((Text::new(name.clone()), if sel { cx.black(27.0) } else { cx.bold(25.0) }, TextColor(if sel { WHITE } else { DIM })));
                b.spawn((Node { width: Val::Percent(100.0), height: cx.px(3.0), ..default() }, BackgroundColor(if sel { pal.accent } else { Color::NONE })));
            });
        }
        if tabs.len() > 1 {
            super::view::hint(t, cx, "E", "xenonbutton_rb", "");
        }
    });
    c.spawn((Text::new(format!("{}/{POINTS}", class.spent)), cx.bold(54.0), TextColor(if class.spent > POINTS { ENEMY } else { pal.accent }), abs(None, Some(64.0), Some(cx.w - r), None, cx)));
    c.spawn(Node { width: cx.px(r - l), height: cx.px(12.0), column_gap: cx.px(8.0), ..abs(Some(l), Some(146.0), None, None, cx) }).with_children(|b| {
        for k in 0..POINTS {
            b.spawn((Node { flex_grow: 1.0, ..default() }, BackgroundColor(if k < class.spent { pal.accent } else { pal.accent_a(0.22) })));
        }
    });
    let cats = categories(&picks);
    let Some(at) = cats.iter().position(|(a, b)| (*a..*b).contains(&s.focus)) else { return };
    let (a, b) = cats[at];
    let have = class.get(slot);
    // The row: the gun in focus large, then up to three after it.
    let (big_w, big_h, w, h, gap) = (600.0, 380.0, 330.0, 252.0, 20.0);
    let first = s.focus.saturating_sub(0).max(a);
    let shown_after: Vec<usize> = (first + 1..b).take(3).collect();
    let before: Vec<usize> = (a..first).rev().take(3usize.saturating_sub(shown_after.len())).collect::<Vec<_>>().into_iter().rev().collect();
    c.spawn(Node { column_gap: cx.px(gap), align_items: AlignItems::Center, ..abs(Some(l), Some(176.0), None, None, cx) }).with_children(|row| {
        for k in before.iter().copied().chain(std::iter::once(s.focus)).chain(shown_after.iter().copied()) {
            let p = &picks[k];
            let on = k == s.focus;
            let (tw, th) = if on { (big_w, big_h) } else { (w, h) };
            row.spawn((
                Hit::Row(k),
                Button,
                Node { width: cx.px(tw), height: cx.px(th), flex_direction: FlexDirection::Column, border: UiRect::all(cx.px(2.0)), ..default() },
                BorderColor::all(if on { pal.accent } else { WHITE.with_alpha(0.07) }),
                tile_fill(cx, false),
            ))
            .with_children(|tile| {
                tile.spawn(Node { flex_grow: 1.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }).with_children(|pic| {
                    if p.locked {
                        lock(pic, cx);
                    } else if !p.image.is_empty() {
                        picture_contain(pic, cx, std::slice::from_ref(&p.image), tw * 0.86, th - 70.0);
                    }
                });
                tile.spawn((
                    Node { height: cx.px(if on { 50.0 } else { 44.0 }), padding: UiRect::horizontal(cx.px(10.0)), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
                ))
                .with_children(|bar| {
                    bar.spawn((Text::new(p.name.clone()), cx.bold(if on { 34.0 } else { 30.0 }), TextColor(if on { pal.accent } else if p.locked { DIM } else { WHITE })));
                    let tag = if p.locked {
                        format!("LV {}", p.level)
                    } else if p.value == have && p.value != 0 {
                        "EQUIPPED".to_owned()
                    } else {
                        String::new()
                    };
                    if !tag.is_empty() {
                        bar.spawn((Text::new(tag), cx.bold(16.0), TextColor(if p.locked { GOLD } else { pal.accent })));
                    }
                });
            });
        }
    });
    // Under the row, at the left: a dot per gun, what it is, the categories.
    let p = &picks[s.focus];
    c.spawn(Node { width: cx.px(big_w), flex_direction: FlexDirection::Column, row_gap: cx.px(8.0), ..abs(Some(l), Some(176.0 + big_h + 16.0), None, None, cx) }).with_children(|col| {
        col.spawn(Node { column_gap: cx.px(10.0), ..default() }).with_children(|d| {
            for k in a..b {
                let on = k == s.focus;
                d.spawn((
                    Node { width: cx.px(if on { 10.0 } else { 7.0 }), height: cx.px(if on { 10.0 } else { 7.0 }), ..default() },
                    BackgroundColor(if on { pal.accent } else { WHITE.with_alpha(0.4) }),
                    UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
                ));
            }
        });
        col.spawn(Node { column_gap: cx.px(12.0), align_items: AlignItems::FlexStart, ..default() }).with_children(|t| {
            super::view::pointer(t, cx);
            let line = if p.locked { format!("Unlocks at level {}.", p.level) } else { p.desc.clone() };
            t.spawn((Text::new(line), cx.regular(28.0), TextColor(DIM), Node { max_width: cx.px(big_w - 30.0), ..default() }));
        });
        col.spawn(Node { height: cx.px(30.0), ..default() });
        // The categories: this one large, the rest after it in turn.
        let name = |k: usize| {
            let g = &picks[cats[k].0].group;
            if g.is_empty() { "NONE".to_owned() } else { g.clone() }
        };
        col.spawn((Text::new(name(at)), cx.bold(64.0), TextColor(pal.accent)));
        col.spawn((Node { width: Val::Percent(100.0), height: cx.px(1.0), margin: UiRect::bottom(cx.px(6.0)), ..default() }, BackgroundColor(WHITE.with_alpha(0.25))));
        for i in 1..cats.len() {
            let k = (at + i) % cats.len();
            col.spawn((Text::new(name(k)), cx.regular(30.0), TextColor(WHITE)));
        }
    });
    // At the right: what the gun is, in a few lines.
    c.spawn(Node { width: cx.px(640.0), flex_direction: FlexDirection::Column, row_gap: cx.px(10.0), ..abs(Some(l + big_w + 260.0), Some(176.0 + big_h + 60.0), None, None, cx) }).with_children(|col| {
        let game = if p.value > 0 { game_tab(p.value).1 } else { "" };
        let lines = [
            ("GAME", game.to_owned()),
            ("POINTS", if p.cost > 0 { p.cost.to_string() } else { String::new() }),
            ("UNLOCKS", if p.level > 1 { format!("LEVEL {}", p.level) } else { String::new() }),
            ("IN THIS CLASS", if p.value == have && p.value != 0 { "EQUIPPED".to_owned() } else { String::new() }),
        ];
        for (k, v) in lines.into_iter().filter(|(_, v)| !v.is_empty()) {
            col.spawn(Node { column_gap: cx.px(20.0), ..default() }).with_children(|r| {
                r.spawn((Text::new(k), cx.regular(28.0), TextColor(DIM), Node { width: cx.px(220.0), ..default() }));
                r.spawn((Text::new(v), cx.bold(28.0), TextColor(WHITE)));
            });
        }
    });
}

/// A padlock, drawn: its shackle a ring, its body a rounded block.
fn lock(c: &mut Kids, cx: &Ctx) {
    c.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }).with_children(|l| {
        l.spawn((
            Node { width: cx.px(30.0), height: cx.px(28.0), border: UiRect::new(cx.px(6.0), cx.px(6.0), cx.px(6.0), cx.px(0.0)), border_radius: BorderRadius::top(cx.px(15.0)), ..default() },
            BorderColor::all(WHITE.with_alpha(0.45)),
        ));
        l.spawn((Node { width: cx.px(46.0), height: cx.px(36.0), border_radius: BorderRadius::all(cx.px(6.0)), ..default() }, BackgroundColor(WHITE.with_alpha(0.45))));
    });
}

/// `SWAT-556` -> `Swat-556`, `FMJ` stays (Black Ops II's names in its
/// pickers; a word of four letters or fewer is left in capitals).
fn title_case(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            if w.len() <= 4 || w.chars().any(|c| c.is_ascii_digit()) {
                return w.to_owned();
            }
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c.flat_map(char::to_lowercase)).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The picker's tabs as Modern Warfare III's game tabs: the names side by
/// side under the title, the one he is on bright with a line under it.
fn game_tabs(c: &mut Kids, cx: &mut Ctx, labels: &[String], on: usize) {
    c.spawn(Node { column_gap: cx.px(22.0), align_items: AlignItems::FlexEnd, ..abs(None, Some(62.0), Some(cx.w - 1794.0 + 200.0), None, cx) }).with_children(|t| {
        super::view::hint(t, cx, "Q", "xenonbutton_lb", "");
        for (i, l) in labels.iter().enumerate() {
            let sel = i == on;
            t.spawn((Hit::Page(i), Button, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: cx.px(4.0), ..default() })).with_children(|b| {
                b.spawn((Text::new(l.clone()), if sel { cx.black(26.0) } else { cx.bold(24.0) }, TextColor(if sel { WHITE } else { DIM })));
                b.spawn((Node { width: Val::Percent(100.0), height: cx.px(3.0), ..default() }, BackgroundColor(if sel { cx.pal.accent } else { Color::NONE })));
            });
        }
        super::view::hint(t, cx, "E", "xenonbutton_rb", "");
    });
}

#[cfg(test)]
mod tests {
    use super::{Pick, categories};

    fn pick(group: &str) -> Pick {
        Pick { value: 1, name: String::new(), desc: String::new(), image: String::new(), cost: 1, locked: false, level: 1, group: group.to_owned() }
    }

    #[test]
    fn categories_are_runs_of_one_kind() {
        let picks = [pick(""), pick("SMG"), pick("SMG"), pick("AR"), pick("SNIPER"), pick("SNIPER")];
        assert_eq!(categories(&picks), vec![(0, 1), (1, 3), (3, 4), (4, 6)]);
        assert!(categories(&[]).is_empty());
    }
}
