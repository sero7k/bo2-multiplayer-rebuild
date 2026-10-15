//! bo2mp shell: Create-a-Class - his five classes, a class's slots, and the
//! items (or attachments) a slot takes, in the shell's own look. Pick 10:
//! a class spends at most ten points; the wildcards open the extra slots
//! (Primary / Secondary Gunfighter, the Perk Greeds, Danger Close,
//! Overkill), as in Black Ops II.

use bevy::prelude::*;

use super::classes::{self, ClassBook, ClassItem, ClassView, Edit, POINTS, group_label, slot_kind};
use super::theme::*;
use super::view::{Ctx, Kids, header, picture_contain};
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
}

/// The picker's tabs (weapon groups) and the choices on the tab he is on.
pub(crate) fn picks(s: &Shell, book: &ClassBook) -> (Vec<String>, Vec<Pick>) {
    let Some(c) = book.classes.get(s.class as usize) else { return (Vec::new(), Vec::new()) };
    let slot = EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)].0;
    // bo2mp: Black Ops II's camos for the gun (Personalize Weapon's list).
    if slot.ends_with("camo") {
        let mut v = vec![Pick { value: 0, name: "NONE".to_owned(), desc: "The gun as it comes.".to_owned(), image: String::new(), cost: 0, locked: false, level: 1 }];
        for (n, name, image) in camo_choices(book, c.get(weapon_of(slot))).unwrap_or_default() {
            v.push(Pick {
                value: n,
                name,
                desc: "Worn on the gun in matches, first and third person.".to_owned(),
                image,
                cost: 0,
                locked: false,
                level: 1,
            });
        }
        return (vec!["CAMOS".to_owned()], v);
    }
    if slot.contains("attachment") {
        let mut v = vec![Pick { value: 0, name: "NONE".to_owned(), desc: "No attachment.".to_owned(), image: String::new(), cost: 0, locked: false, level: 1 }];
        if let Some(w) = book.item(c.get(weapon_of(slot))) {
            for (n, name, image) in &w.attachments {
                v.push(Pick { value: *n, name: name.to_uppercase(), desc: String::new(), image: image.clone(), cost: 1, locked: false, level: 1 });
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
    let mut tabs: Vec<String> = Vec::new();
    if weapons {
        for i in &items {
            let g = group_label(&i.group).to_owned();
            if !g.is_empty() && !tabs.contains(&g) {
                tabs.push(g);
            }
        }
    }
    if tabs.is_empty() {
        tabs.push(EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)].1.split(" ·").next().unwrap_or("").to_owned() + "S");
    }
    let tab = s.page.min(tabs.len() - 1);
    let mut out = Vec::new();
    // (A slot that may be empty: NONE first.)
    if !matches!(slot, "primary") {
        out.push(Pick { value: 0, name: "NONE".to_owned(), desc: "Leave this slot empty.".to_owned(), image: String::new(), cost: 0, locked: false, level: 1 });
    }
    for i in items {
        if weapons && group_label(&i.group) != tabs[tab] {
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
        });
    }
    if weapons && tab != 0 {
        out.retain(|p| p.value != 0);
    }
    (tabs, out)
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

/// The ten points as ten bars, the spent ones lit.
fn points_bar(c: &mut Kids, cx: &Ctx, spent: i32) {
    c.spawn(Node { column_gap: cx.px(6.0), align_items: AlignItems::Center, ..default() }).with_children(|b| {
        for k in 0..POINTS {
            b.spawn((
                Node { width: cx.px(46.0), height: cx.px(6.0), ..default() },
                BackgroundColor(if k < spent { cx.pal.accent } else { KEYCAP }),
            ));
        }
        b.spawn((
            Node { margin: UiRect::left(cx.px(12.0)), ..default() },
            Text::new(format!("{spent}/{POINTS}")),
            cx.bold(30.0),
            TextColor(if spent > POINTS { ENEMY } else { WHITE }),
        ));
    });
}

/// An item with its picture, its slot's label and its name.
fn item_cell(c: &mut Kids, cx: &mut Ctx, label: &str, item: Option<&ClassItem>, extra: &str, w: f32, pic_w: f32, pic_h: f32) {
    c.spawn(Node {
        width: cx.px(w),
        column_gap: cx.px(14.0),
        align_items: AlignItems::Center,
        padding: UiRect::axes(cx.px(12.0), cx.px(8.0)),
        ..default()
    })
    .insert(BackgroundColor(cx.pal.panel_soft))
    .with_children(|r| {
        let names = item.map(|i| vec![i.image.clone()]).unwrap_or_default();
        picture_contain(r, cx, &names, pic_w, pic_h);
        r.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(2.0), flex_shrink: 1.0, ..default() }).with_children(|t| {
            t.spawn((Text::new(label), cx.bold(17.0), TextColor(FAINT)));
            t.spawn((
                Text::new(item.map_or_else(|| "EMPTY".to_owned(), |i| i.name.to_uppercase())),
                cx.bold(24.0),
                TextColor(if item.is_some() { WHITE } else { DIM }),
            ));
            if !extra.is_empty() {
                t.spawn((Text::new(extra), cx.regular(17.0), TextColor(DIM)));
            }
        });
    });
}

/// A class at a glance: its weapons with their attachments, perks,
/// grenades and wildcards.
pub(crate) fn class_card(c: &mut Kids, cx: &mut Ctx, book: &ClassBook, class: &ClassView, left: f32, top: f32, width: f32) {
    let attach = |weapon: &str| -> String {
        let names: Vec<String> = (1..=3)
            .filter_map(|k| {
                let n = class.get(&format!("{weapon}attachment{k}"));
                book.item(class.get(weapon))?.attachments.iter().find(|(x, ..)| *x == n).map(|(_, name, _)| name.clone())
            })
            .collect();
        names.join("  ·  ")
    };
    // (And the gun's camo after its attachments.)
    let with_camo = |weapon: &str, line: String| -> String {
        let n = class.get(&format!("{weapon}camo"));
        let name = camo_choices(book, class.get(weapon)).and_then(|l| l.into_iter().find(|(k, ..)| *k == n)).map(|(_, name, _)| name);
        match name {
            Some(k) if line.is_empty() => k,
            Some(k) => format!("{line}  ·  {k}"),
            None => line,
        }
    };
    let pa = with_camo("primary", attach("primary"));
    let sa = with_camo("secondary", attach("secondary"));
    c.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: cx.px(left),
            top: cx.px(top),
            width: cx.px(width),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(8.0),
            padding: UiRect::all(cx.px(18.0)),
            border: UiRect::top(cx.px(3.0)),
            ..default()
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|h| {
            h.spawn((Text::new(class.name.to_uppercase()), cx.bold(36.0), TextColor(WHITE)));
            points_bar(h, cx, class.spent);
        });
        let half = (width - 36.0 - 8.0) / 2.0;
        p.spawn(Node { column_gap: cx.px(8.0), ..default() }).with_children(|row| {
            item_cell(row, cx, "PRIMARY", book.item(class.get("primary")), &pa, half, 200.0, 90.0);
            item_cell(row, cx, "SECONDARY", book.item(class.get("secondary")), &sa, half, 200.0, 90.0);
        });
        let third = (width - 36.0 - 16.0) / 3.0;
        p.spawn(Node { column_gap: cx.px(8.0), ..default() }).with_children(|row| {
            for k in 1..=3 {
                let greed = book.item(class.get(&format!("specialty{}", k + 3))).map(|i| format!("+ {}", i.name.to_uppercase())).unwrap_or_default();
                item_cell(row, cx, &format!("PERK {k}"), book.item(class.get(&format!("specialty{k}"))), &greed, third, 64.0, 64.0);
            }
        });
        p.spawn(Node { column_gap: cx.px(8.0), ..default() }).with_children(|row| {
            let count = |slot: &str| {
                let n = class.get(&format!("{slot}count"));
                if n > 1 { format!("x{n}") } else { String::new() }
            };
            item_cell(row, cx, "LETHAL", book.item(class.get("primarygrenade")), &count("primarygrenade"), half, 64.0, 64.0);
            item_cell(row, cx, "TACTICAL", book.item(class.get("specialgrenade")), &count("specialgrenade"), half, 64.0, 64.0);
        });
        p.spawn(Node { column_gap: cx.px(8.0), ..default() }).with_children(|row| {
            for k in 1..=3 {
                item_cell(row, cx, &format!("WILDCARD {k}"), book.item(class.get(&format!("bonuscard{k}"))), "", third, 56.0, 56.0);
            }
        });
    });
}

/// Compact rows: a label on the left, its value on the right.
fn slot_rows(c: &mut Kids, cx: &Ctx, rows: &[Row], focus: usize, width: f32) {
    // (The rows share the height from the header to the footer: a class
    // with every wildcard slot open still fits.)
    let height = (870.0 / rows.len().max(1) as f32 - 3.0).clamp(26.0, 38.0);
    let text = if height < 34.0 { 18.0 } else { 21.0 };
    for (i, r) in rows.iter().enumerate() {
        let on = i == focus;
        let mut e = c.spawn((
            Hit::Row(i),
            Button,
            Node {
                width: cx.px(width),
                height: cx.px(height),
                padding: UiRect::horizontal(cx.px(16.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: UiRect::left(cx.px(4.0)),
                ..default()
            },
            BorderColor::all(if on { cx.pal.accent } else { Color::NONE }),
        ));
        if on {
            e.insert(fade_right(cx.pal.accent_a(0.30), cx.pal.accent_a(0.06)));
        } else {
            e.insert(BackgroundColor(cx.pal.panel_soft));
        }
        e.with_children(|line| {
            line.spawn((Text::new(r.label.clone()), cx.bold(text), TextColor(if on { WHITE } else { DIM })));
            if let Some(v) = &r.value {
                let selector = matches!(r.act, Act::Cycle(_));
                let shown = if selector { format!("‹  {v}  ›") } else { format!("{v}  ›") };
                line.spawn((Text::new(shown), cx.bold(text), TextColor(if on { WHITE } else { TEXT })));
            }
        });
    }
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, s: &Shell, book: &ClassBook, rows: &[Row]) {
    let pal = cx.pal;
    match s.screen {
        Screen::Classes => {
            header(c, cx, &[super::view::section(s), "CREATE A CLASS"]);
            c.spawn(Node {
                position_type: PositionType::Absolute,
                left: cx.px(72.0),
                top: cx.px(176.0),
                flex_direction: FlexDirection::Column,
                row_gap: cx.px(8.0),
                ..default()
            })
            .with_children(|col| super::view::menu_rows(col, cx, rows, s.focus, 440.0, false));
            if let Some(class) = book.classes.get(s.focus) {
                class_card(c, cx, book, class, 560.0, 176.0, 1288.0);
            }
        }
        Screen::ClassEdit => {
            let Some(class) = book.classes.get(s.class as usize) else { return };
            header(c, cx, &["CREATE A CLASS", &class.name.to_uppercase()]);
            c.spawn(Node {
                position_type: PositionType::Absolute,
                left: cx.px(72.0),
                top: cx.px(140.0),
                flex_direction: FlexDirection::Column,
                row_gap: cx.px(3.0),
                ..default()
            })
            .with_children(|col| slot_rows(col, cx, rows, s.focus, 560.0));
            class_card(c, cx, book, class, 680.0, 176.0, 1168.0);
        }
        Screen::ItemPick => {
            let Some(class) = book.classes.get(s.class as usize) else { return };
            let (slot, label) = EDIT_SLOTS[s.slot.min(EDIT_SLOTS.len() - 1)];
            header(c, cx, &[&class.name.to_uppercase(), label]);
            let (tabs, picks) = picks(s, book);
            let tab = s.page.min(tabs.len().saturating_sub(1));
            super::view::tabs(c, cx, &tabs, tab, Hit::Page);
            let have = class.get(slot);
            let (tw, th, gap) = (280.0, 168.0, 14.0);
            let (cols, first, last) = super::view::grid_window(cx, picks.len(), s.focus, tw, th, gap, 196.0, 230.0);
            super::view::grid_more(c, cx, first, last, picks.len(), 72.0 + cols as f32 * (tw + gap) - 220.0, 160.0);
            c.spawn(Node {
                position_type: PositionType::Absolute,
                left: cx.px(72.0),
                top: cx.px(196.0),
                width: cx.px(cols as f32 * (tw + gap)),
                flex_wrap: FlexWrap::Wrap,
                column_gap: cx.px(gap),
                row_gap: cx.px(gap),
                ..default()
            })
            .with_children(|grid| {
                for (k, p) in picks.iter().enumerate().take(last).skip(first) {
                    let on = k == s.focus;
                    grid.spawn((
                        Hit::Row(k),
                        Button,
                        Node {
                            width: cx.px(tw),
                            height: cx.px(th),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::all(cx.px(10.0)),
                            border: UiRect::all(cx.px(if on { 3.0 } else { 1.0 })),
                            ..default()
                        },
                        BorderColor::all(if on { pal.accent } else { pal.line }),
                        BackgroundColor(if on { pal.accent_a(0.16) } else { pal.panel }),
                    ))
                    .with_children(|tile| {
                        tile.spawn(Node { justify_content: JustifyContent::Center, flex_grow: 1.0, align_items: AlignItems::Center, ..default() })
                            .with_children(|pic| {
                                if p.image.is_empty() {
                                    pic.spawn((Text::new("—"), cx.bold(40.0), TextColor(FAINT)));
                                } else {
                                    picture_contain(pic, cx, std::slice::from_ref(&p.image), tw - 30.0, th - 60.0);
                                }
                            });
                        tile.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() })
                            .with_children(|bar| {
                                bar.spawn((Text::new(p.name.clone()), cx.bold(20.0), TextColor(if p.locked { FAINT } else if on { WHITE } else { TEXT })));
                                let tag = if p.locked {
                                    format!("LV {}", p.level)
                                } else if p.value == have && p.value != 0 {
                                    "●".to_owned()
                                } else if p.cost > 0 {
                                    format!("{}", p.cost)
                                } else {
                                    String::new()
                                };
                                bar.spawn((Text::new(tag), cx.bold(18.0), TextColor(if p.locked { GOLD } else { pal.accent })));
                            });
                    });
                }
            });
            // The one in focus, in words.
            if let Some(p) = picks.get(s.focus) {
                c.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: cx.px(72.0),
                        bottom: cx.px(86.0),
                        width: cx.px(620.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: cx.px(4.0),
                        ..default()
                    },
                ))
                .with_children(|t| {
                    t.spawn((Text::new(p.name.clone()), cx.bold(54.0), TextColor(WHITE)));
                    if !p.desc.is_empty() {
                        t.spawn((Text::new(p.desc.clone()), cx.regular(21.0), TextColor(DIM), TextLayout::justify(Justify::Right)));
                    }
                    t.spawn((Text::new(format!("{} / {POINTS} POINTS IN THIS CLASS", class.spent)), cx.bold(18.0), TextColor(pal.accent)));
                });
            }
        }
        _ => {}
    }
}
