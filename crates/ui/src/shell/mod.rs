//! bo2mp shell: the client's own front end.
//!
//! Every front-end screen is drawn here - the main menu, Multiplayer with
//! its online panel, the Custom Game lobby (LOBBY / LOADOUTS / SETTINGS),
//! the map and mode pickers - over a full-screen map picture that fades
//! into the dark on the side the menus sit. BO2's Global (the globe and its
//! activity screens) is gone: the online panel says how many players are on
//! the master server instead. The loading screen (`loading`) and the in-game
//! HUD (`hud_modern`) share the look (`theme`).
//!
//! BO2's scripts still run underneath (`bridge`): the shell starts the match
//! through them, and opens their Create-a-Class, Barracks and Options
//! screens over itself until it has its own.
//!
//! `BO2_SHELL=off` brings BO2's own menus back.

mod background;
pub(crate) mod bridge;
mod cac;
pub(crate) mod classes;
pub(crate) mod ingame;
mod loading;
mod online;
pub(crate) mod options;
pub(crate) mod pics; // (hud_match reuses the picture resolver)
#[doc(hidden)]
pub mod preview;
pub(crate) mod theme; // (hud_match reuses the colours and fonts)
mod view;

use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use frame::ClientSet;

pub(crate) use bridge::{covers_loading, covers_lui, enabled, lui_side};

use bridge::{Catalog, Request, StartSpec};

/// The screen he is on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Screen {
    #[default]
    Main,
    Multiplayer,
    Lobby,
    MapPick,
    ModePick,
    /// Create-a-Class: his classes, one class's slots, a slot's items.
    Classes,
    ClassEdit,
    ItemPick,
    /// Options (its tab is `page`), Barracks.
    Options,
    Barracks,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Tab {
    #[default]
    Lobby,
    Loadouts,
    Settings,
}

impl Tab {
    const ALL: [Tab; 3] = [Tab::Lobby, Tab::Loadouts, Tab::Settings];

    fn label(self) -> &'static str {
        match self {
            Tab::Lobby => "LOBBY",
            Tab::Loadouts => "LOADOUTS",
            Tab::Settings => "SETTINGS",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Setting {
    Mode,
    Friends,
    Enemies,
    Difficulty,
    LethalCount,
    TacticalCount,
    /// An Options row (of the tab he is on).
    Option(u8),
}

/// What a row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Act {
    Multiplayer,
    Zombies,
    Options,
    Barracks,
    Prestige,
    Quit,
    CustomGame,
    Start,
    PickMap,
    PickMode,
    Cycle(Setting),
    /// A map tile / mode row (its index in the list shown).
    Choose(usize),
    /// Create-a-Class: his classes, a class, a slot of it (`cac::EDIT_SLOTS`).
    Classes,
    EditClass(i32),
    EditSlot(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Row {
    pub label: String,
    pub sub: String,
    /// A selector's value (drawn `< value >`).
    pub value: Option<String>,
    pub act: Act,
}

impl Row {
    fn new(label: &str, sub: &str, act: Act) -> Row {
        Row {
            label: label.to_owned(),
            sub: sub.to_owned(),
            value: None,
            act,
        }
    }
}

pub(crate) const DIFFICULTY: [&str; 4] = ["RECRUIT", "REGULAR", "HARDENED", "VETERAN"];
/// Maps a picker page holds (6 by 4).
pub(crate) const PAGE: usize = 24;
pub(crate) const COLUMNS: usize = 6;
/// The columns the map / item grid has on this screen, and its first row
/// in view (the view sets them; moving uses them).
pub(crate) static GRID_COLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(COLUMNS);
pub(crate) static GRID_FIRST_ROW: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Resource, Debug, Clone)]
pub(crate) struct Shell {
    pub screen: Screen,
    pub focus: usize,
    pub tab: Tab,
    pub map: String,
    pub mode: String,
    pub friends: u32,
    pub enemies: u32,
    pub difficulty: u32,
    /// The map picker's page (a group, split by `PAGE`).
    pub page: usize,
    pub note: Option<(String, f64)>,
    /// He started a match from the lobby (coming back lands there).
    pub played: bool,
    /// Create-a-Class: the class, the slot (`cac::EDIT_SLOTS`), and the
    /// screen it was opened from.
    pub class: i32,
    pub slot: usize,
    pub cac_from: Screen,
    /// The clock (seconds) as of his last input, for notes.
    pub now: f64,
    /// His new name while he types it (Options > ACCOUNT).
    pub typing: Option<String>,
}

impl Default for Shell {
    fn default() -> Self {
        Shell {
            screen: Screen::Main,
            focus: 0,
            tab: Tab::Lobby,
            map: "mp_la".to_owned(),
            mode: "tdm".to_owned(),
            friends: 5,
            enemies: 6,
            difficulty: 1,
            page: 0,
            note: None,
            played: false,
            class: 0,
            slot: 0,
            cac_from: Screen::Multiplayer,
            now: 0.0,
            typing: None,
        }
    }
}

/// The map picker's pages: (title, maps by index into the catalog).
pub(crate) fn pages(c: &Catalog) -> Vec<(String, Vec<usize>)> {
    let mut out = Vec::new();
    for g in c.groups() {
        let all: Vec<usize> = c.maps.iter().enumerate().filter(|(_, m)| m.group == g).map(|(i, _)| i).collect();
        let n = all.len().div_ceil(PAGE);
        for (k, chunk) in all.chunks(PAGE).enumerate() {
            let title = if n > 1 { format!("{g} {}", k + 1) } else { g.clone() };
            out.push((title, chunk.to_vec()));
        }
    }
    out
}

/// The rows the screen shows.
pub(crate) fn rows(s: &Shell, c: &Catalog) -> Vec<Row> {
    match s.screen {
        Screen::Main => vec![
            Row::new("MULTIPLAYER", "Custom games with bots on every map.", Act::Multiplayer),
            Row::new("ZOMBIES", "Survive the undead.", Act::Zombies),
            Row::new("OPTIONS", "Graphics, sound and controls.", Act::Options),
            Row::new("QUIT", "Back to the desktop.", Act::Quit),
        ],
        Screen::Multiplayer => vec![
            Row::new("CUSTOM GAME", "Pick the map, the mode and the bots, then play.", Act::CustomGame),
            Row::new("CREATE A CLASS", "Build your loadouts.", Act::Classes),
            Row::new("BARRACKS", "Your combat record and prestige.", Act::Barracks),
            Row::new("OPTIONS", "Graphics, sound and controls.", Act::Options),
        ],
        Screen::Lobby => match s.tab {
            Tab::Lobby => vec![
                Row::new("START MATCH", "Everyone in, the match begins.", Act::Start),
                Row::new("CHANGE MAP", "Every map from every game installed.", Act::PickMap),
                Row::new("CHANGE MODE", "Team Deathmatch, Domination, Search & Destroy...", Act::PickMode),
            ],
            Tab::Loadouts => vec![
                Row::new("CREATE A CLASS", "Build your loadouts.", Act::Classes),
                Row::new("BARRACKS", "Your combat record and prestige.", Act::Barracks),
            ],
            Tab::Settings => {
                let mode = c.mode(&s.mode).map_or_else(|| s.mode.to_uppercase(), |m| m.title.clone());
                let mut r = vec![
                    Row::new("GAME MODE", "", Act::Cycle(Setting::Mode)),
                    Row::new("FRIENDLY BOTS", "", Act::Cycle(Setting::Friends)),
                    Row::new("ENEMY BOTS", "", Act::Cycle(Setting::Enemies)),
                    Row::new("BOT DIFFICULTY", "", Act::Cycle(Setting::Difficulty)),
                    Row::new("MAP", "", Act::PickMap),
                ];
                r[0].value = Some(mode);
                r[1].value = Some(s.friends.to_string());
                r[2].value = Some(s.enemies.to_string());
                r[3].value = Some(DIFFICULTY[s.difficulty.min(3) as usize].to_owned());
                r[4].value = Some(c.map(&s.map).map_or_else(|| s.map.clone(), |m| m.title.clone()));
                r
            }
        },
        Screen::ModePick => c
            .modes
            .iter()
            .enumerate()
            .map(|(i, m)| Row::new(&m.title, &m.desc, Act::Choose(i)))
            .collect(),
        Screen::Classes | Screen::ClassEdit | Screen::ItemPick => classes::book().map(|b| cac::rows(s, &b)).unwrap_or_default(),
        Screen::Options => {
            let mut r = options::rows(s.page, &options::snapshot());
            if s.page == options::ACCOUNT_TAB
                && let (Some(t), Some(row)) = (&s.typing, r.first_mut())
            {
                row.value = Some(format!("{t}_"));
            }
            r
        }
        Screen::Barracks => {
            let mut r = vec![Row::new("COMBAT RECORD", "Your kills, wins and time played.", Act::Barracks)];
            if c.player.can_prestige {
                r.push(Row::new("ENTER PRESTIGE", "Back to level 1 with the next prestige icon. Press twice to confirm.", Act::Prestige));
            } else {
                r.push(Row::new("PRESTIGE", "Reach level 155 to enter the next prestige.", Act::Prestige));
            }
            r
        }
        Screen::MapPick => {
            let p = pages(c);
            p.get(s.page.min(p.len().saturating_sub(1)))
                .map(|(_, maps)| {
                    maps.iter()
                        .enumerate()
                        .map(|(k, &i)| Row::new(&c.maps[i].title, &c.maps[i].origin, Act::Choose(k)))
                        .collect()
                })
                .unwrap_or_default()
        }
    }
}

impl Shell {
    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.focus = 0;
    }

    fn note(&mut self, text: &str, now: f64) {
        self.note = Some((text.to_owned(), now + 3.5));
    }

    fn back(&mut self) {
        match self.screen {
            Screen::Options | Screen::Barracks => {
                let from = self.cac_from;
                let was = self.screen;
                self.go(from);
                self.page = 0;
                self.focus = match (from, was) {
                    (Screen::Main, _) => 2,
                    (Screen::Multiplayer, Screen::Barracks) => 2,
                    (Screen::Multiplayer, _) => 3,
                    (Screen::Lobby, _) => 1,
                    _ => 0,
                };
            }
            Screen::Classes => {
                let from = self.cac_from;
                self.go(from);
                self.focus = if from == Screen::Multiplayer { 1 } else { 0 };
            }
            Screen::ClassEdit => {
                let n = self.class.max(0) as usize;
                self.go(Screen::Classes);
                self.focus = n;
            }
            Screen::ItemPick => {
                let slot = self.slot;
                self.screen = Screen::ClassEdit;
                self.page = 0;
                // (Back on the row of the slot he picked for.)
                self.focus = classes::book()
                    .and_then(|b| {
                        let c = b.classes.get(self.class as usize)?;
                        cac::rows(self, &b).iter().position(|r| r.act == Act::EditSlot(slot)).filter(|_| c.n >= 0)
                    })
                    .unwrap_or(0);
            }
            Screen::Main => {}
            Screen::Multiplayer => self.go(Screen::Main),
            Screen::Lobby => {
                if bridge::starting_in().is_some() {
                    bridge::request(Request::CancelStart);
                } else {
                    self.go(Screen::Multiplayer);
                    self.focus = 0;
                }
            }
            Screen::MapPick | Screen::ModePick => {
                let from = self.screen;
                self.go(Screen::Lobby);
                self.focus = match (self.tab, from) {
                    (Tab::Settings, Screen::MapPick) => 4,
                    (Tab::Settings, _) => 0,
                    (_, Screen::MapPick) => 1,
                    _ => 2,
                };
            }
        }
    }

    fn cycle(&mut self, what: Setting, by: i32, c: &Catalog) {
        let step = |v: u32, max: u32| ((v as i32 + by).rem_euclid(max as i32 + 1)) as u32;
        match what {
            Setting::Friends => self.friends = step(self.friends, 8),
            Setting::Enemies => self.enemies = step(self.enemies, 9),
            Setting::Difficulty => self.difficulty = step(self.difficulty, 3),
            Setting::Option(k) => options::queue(self.page, k, by),
            Setting::LethalCount | Setting::TacticalCount => {
                if let Some(b) = classes::book() {
                    cac::count(self, &b, what == Setting::LethalCount, by, self.now);
                }
            }
            Setting::Mode => {
                if !c.modes.is_empty() {
                    let i = c.modes.iter().position(|m| m.key == self.mode).unwrap_or(0) as i32;
                    let n = c.modes.len() as i32;
                    self.mode = c.modes[(i + by).rem_euclid(n) as usize].key.clone();
                }
            }
        }
    }

    fn activate(&mut self, act: Act, c: &Catalog, now: f64, exit: &mut MessageWriter<AppExit>) {
        match act {
            Act::Multiplayer => self.go(Screen::Multiplayer),
            Act::Zombies => bridge::request(Request::Zombies),
            Act::Options => {
                self.cac_from = self.screen;
                self.go(Screen::Options);
                self.page = 0;
            }
            Act::Barracks if self.screen != Screen::Barracks => {
                self.cac_from = self.screen;
                self.go(Screen::Barracks);
            }
            Act::Barracks => {}
            Act::Prestige => {
                if !c.player.can_prestige {
                    self.note("Reach level 155 to enter the next prestige.", now);
                } else if self.note.as_ref().is_some_and(|(t, _)| t.starts_with("Press again")) {
                    bridge::request(Request::Prestige);
                    self.note("Prestige entered.", now);
                } else {
                    self.note("Press again to enter the next prestige.", now);
                }
            }
            Act::Quit => {
                exit.write(AppExit::Success);
            }
            Act::CustomGame => {
                self.go(Screen::Lobby);
                self.tab = Tab::Lobby;
            }
            Act::Start => {
                if bridge::starting_in().is_some() {
                    return;
                }
                self.played = true;
                bridge::request(Request::Start(StartSpec {
                    map: self.map.clone(),
                    gametype: self.mode.clone(),
                    bot_friends: self.friends,
                    bot_enemies: self.enemies,
                    bot_difficulty: self.difficulty,
                }));
            }
            Act::PickMap => {
                // The page and tile of the map he has.
                let p = pages(c);
                let at = c.maps.iter().position(|m| m.ui_name == self.map);
                let (page, k) = at
                    .and_then(|i| p.iter().enumerate().find_map(|(pi, (_, maps))| maps.iter().position(|&m| m == i).map(|k| (pi, k))))
                    .unwrap_or((0, 0));
                self.screen = Screen::MapPick;
                self.page = page;
                self.focus = k;
            }
            Act::PickMode => {
                self.screen = Screen::ModePick;
                self.focus = c.modes.iter().position(|m| m.key == self.mode).unwrap_or(0);
            }
            // (His name is typed, not stepped.)
            Act::Cycle(Setting::Option(0)) if self.screen == Screen::Options && self.page == options::ACCOUNT_TAB => {
                self.typing = Some(options::snapshot().player_name);
            }
            Act::Cycle(what) => self.cycle(what, 1, c),
            Act::Classes => {
                self.cac_from = self.screen;
                self.go(Screen::Classes);
            }
            Act::EditClass(n) => {
                self.class = n;
                self.go(Screen::ClassEdit);
            }
            Act::EditSlot(i) => {
                if let Some(b) = classes::book() {
                    cac::open_pick(self, &b, i);
                }
            }
            Act::Choose(i) => match self.screen {
                Screen::ModePick => {
                    if let Some(m) = c.modes.get(i) {
                        self.mode = m.key.clone();
                    }
                    self.back();
                }
                Screen::MapPick => {
                    let p = pages(c);
                    if let Some(&m) = p.get(self.page).and_then(|(_, maps)| maps.get(i)) {
                        self.map = c.maps[m].ui_name.clone();
                    }
                    self.back();
                }
                Screen::ItemPick => {
                    if let Some(b) = classes::book()
                        && cac::choose(self, &b, i, now)
                    {
                        self.back();
                    }
                }
                _ => {}
            },
        }
        let _ = now;
    }
}

/// A clickable part of a screen.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) enum Hit {
    Row(usize),
    Tab(usize),
    Page(usize),
    /// A selector's arrow: the row, -1 or +1.
    Step(usize, i32),
    Back,
}

#[derive(Default)]
struct PadRepeat {
    held: Option<(i32, i32)>,
    next_at: f64,
}

#[allow(clippy::too_many_arguments)]
fn shell_input(
    mut shell: ResMut<Shell>,
    front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pads: Query<&Gamepad>,
    hits: Query<(&Hit, &Interaction)>,
    hud_keys: Option<Res<frame::HudInputView>>,
    time: Res<Time>,
    mut exit: MessageWriter<AppExit>,
    mut repeat: Local<PadRepeat>,
    mut typed: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut sounds: MessageWriter<frame::UiPlaySound>,
    mut heard: Local<Option<(Screen, usize, Tab, usize)>>,
) {
    if !enabled() || front.is_none() || bridge::lui_open() || crate::game_folders::is_open() {
        typed.clear();
        return;
    }
    // His new name, typed: letters, digits and a few marks; Enter keeps it,
    // Esc leaves it as it was.
    if let Some(mut t) = shell.typing.clone() {
        use bevy::input::keyboard::Key;
        for ev in typed.read() {
            if !ev.state.is_pressed() {
                continue;
            }
            match &ev.logical_key {
                Key::Enter => {
                    let name = t.trim().to_owned();
                    if !name.is_empty() {
                        options::set_name(name);
                        sounds.write(frame::UiPlaySound { alias: "uin_navigation_select_main".to_owned() });
                    }
                    shell.typing = None;
                    return;
                }
                Key::Escape => {
                    shell.typing = None;
                    sounds.write(frame::UiPlaySound { alias: "cac_cmn_backout".to_owned() });
                    return;
                }
                Key::Backspace => {
                    t.pop();
                }
                Key::Character(s) => {
                    for ch in s.chars() {
                        if t.chars().count() < 16 && (ch.is_ascii_alphanumeric() || " _-.[]".contains(ch)) {
                            t.push(ch);
                        }
                    }
                }
                Key::Space if t.chars().count() < 16 => t.push(' '),
                _ => {}
            }
        }
        if shell.typing.as_ref() != Some(&t) {
            shell.typing = Some(t);
        }
        return;
    }
    typed.clear();
    // The menu sounds: a move, a new tab or page, a new screen.
    let state = (shell.screen, shell.focus, shell.tab, shell.page);
    let sound_for = |old: (Screen, usize, Tab, usize), new: (Screen, usize, Tab, usize)| -> Option<&'static str> {
        if old.0 != new.0 {
            None
        } else if old.2 != new.2 || old.3 != new.3 {
            Some("uin_main_nav")
        } else if old.1 != new.1 {
            Some("uin_navigation_click")
        } else {
            None
        }
    };
    if let Some(old) = *heard
        && let Some(alias) = sound_for(old, state)
    {
        sounds.write(frame::UiPlaySound { alias: alias.to_owned() });
    }
    *heard = Some(state);
    if hud_keys.as_deref().is_some_and(|k| k.console_open) {
        return;
    }
    let Some(c) = bridge::catalog() else { return };
    let now = time.elapsed_secs_f64();
    shell.now = now;
    classes::want(matches!(shell.screen, Screen::Classes | Screen::ClassEdit | Screen::ItemPick));
    if let Some(n) = bridge::take_note() {
        shell.note(&n, now);
    }
    if shell.note.as_ref().is_some_and(|(_, until)| now > *until) {
        shell.note = None;
    }
    // Keep the lobby's map and mode ones the catalog has.
    if !c.maps.is_empty() && c.map(&shell.map).is_none() {
        shell.map = c.maps[0].ui_name.clone();
    }
    if !c.modes.is_empty() && c.mode(&shell.mode).is_none() {
        shell.mode = c.modes[0].key.clone();
    }
    let rows = rows(&shell, &c);
    let n = rows.len();
    if n > 0 && shell.focus >= n {
        shell.focus = n - 1;
    }
    let pad = |b: GamepadButton| pads.iter().any(|p| p.just_pressed(b));
    // The left stick and the d-pad move, repeating while held.
    let stick = pads.iter().find_map(|p| {
        let x = p.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let y = p.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        let dx = if x > 0.6 { 1 } else if x < -0.6 { -1 } else { 0 };
        let dy = if y > 0.6 { -1 } else if y < -0.6 { 1 } else { 0 };
        (dx != 0 || dy != 0).then_some((dx, dy))
    });
    let mut dir = (0, 0);
    match stick {
        Some(d) if repeat.held != Some(d) => {
            repeat.held = Some(d);
            repeat.next_at = now + 0.38;
            dir = d;
        }
        Some(d) if now >= repeat.next_at => {
            repeat.next_at = now + 0.12;
            dir = d;
        }
        Some(_) => {}
        None => repeat.held = None,
    }
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) || pad(GamepadButton::DPadUp) {
        dir.1 = -1;
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) || pad(GamepadButton::DPadDown) {
        dir.1 = 1;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) || pad(GamepadButton::DPadLeft) {
        dir.0 = -1;
    }
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) || pad(GamepadButton::DPadRight) {
        dir.0 = 1;
    }
    let confirm = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::NumpadEnter) || pad(GamepadButton::South);
    let back = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Backspace) || pad(GamepadButton::East);
    let prev_tab = keys.just_pressed(KeyCode::KeyQ) || pad(GamepadButton::LeftTrigger);
    let next_tab = keys.just_pressed(KeyCode::KeyE) || pad(GamepadButton::RightTrigger);
    // Tabs / pages.
    let tabbed = |s: &mut Shell, by: i32| match s.screen {
        Screen::Lobby => {
            let i = Tab::ALL.iter().position(|t| *t == s.tab).unwrap_or(0) as i32;
            s.tab = Tab::ALL[(i + by).rem_euclid(3) as usize];
            s.focus = 0;
        }
        Screen::MapPick => {
            let np = pages(&c).len().max(1) as i32;
            s.page = (s.page as i32 + by).rem_euclid(np) as usize;
            s.focus = 0;
        }
        Screen::Options => {
            s.page = (s.page as i32 + by).rem_euclid(options::TABS.len() as i32) as usize;
            s.focus = 0;
        }
        Screen::ItemPick => {
            let np = classes::book().map_or(1, |b| cac::picks(s, &b).0.len().max(1)) as i32;
            s.page = (s.page as i32 + by).rem_euclid(np) as usize;
            s.focus = 0;
        }
        _ => {}
    };
    if prev_tab {
        tabbed(&mut shell, -1);
    }
    if next_tab {
        tabbed(&mut shell, 1);
    }
    // Moving.
    if n > 0 && dir != (0, 0) {
        if matches!(shell.screen, Screen::MapPick | Screen::ItemPick) {
            let f = shell.focus as i32;
            let cols = GRID_COLS.load(std::sync::atomic::Ordering::Relaxed).max(1) as i32;
            let mut t = f + dir.0 + dir.1 * cols;
            if t < 0 || t >= n as i32 {
                // Off the page's side: the next page.
                if dir.0 != 0 && dir.1 == 0 {
                    tabbed(&mut shell, dir.0);
                    t = 0;
                } else {
                    t = f;
                }
            }
            shell.focus = t.clamp(0, n as i32 - 1) as usize;
        } else {
            if dir.1 != 0 {
                shell.focus = ((shell.focus as i32 + dir.1).rem_euclid(n as i32)) as usize;
            }
            if dir.0 != 0
                && let Some(Act::Cycle(what)) = rows.get(shell.focus).map(|r| r.act)
            {
                shell.cycle(what, dir.0, &c);
            }
        }
    }
    // The mouse: the row under it is in focus; a click works it.
    let clicked = mouse.just_pressed(MouseButton::Left);
    for (hit, interaction) in &hits {
        if *interaction == Interaction::None {
            continue;
        }
        match *hit {
            Hit::Row(i) if i < n => {
                if shell.focus != i {
                    shell.focus = i;
                }
                if clicked {
                    let act = rows[i].act;
                    sounds.write(frame::UiPlaySound { alias: "uin_navigation_select_main".to_owned() });
                    shell.activate(act, &c, now, &mut exit);
                    return;
                }
            }
            Hit::Tab(i) if clicked => {
                shell.tab = Tab::ALL[i.min(2)];
                shell.focus = 0;
                return;
            }
            Hit::Page(i) if clicked => {
                shell.page = i;
                shell.focus = 0;
                return;
            }
            Hit::Step(i, by) if clicked => {
                if let Some(Act::Cycle(what)) = rows.get(i).map(|r| r.act) {
                    shell.focus = i;
                    shell.cycle(what, by, &c);
                }
                return;
            }
            Hit::Back if clicked => {
                shell.back();
                return;
            }
            _ => {}
        }
    }
    if mouse.just_pressed(MouseButton::Right) {
        shell.back();
        return;
    }
    if confirm && let Some(r) = rows.get(shell.focus) {
        let act = r.act;
        sounds.write(frame::UiPlaySound { alias: "uin_navigation_select_main".to_owned() });
        shell.activate(act, &c, now, &mut exit);
    } else if back && shell.screen != Screen::Main {
        sounds.write(frame::UiPlaySound { alias: "cac_cmn_backout".to_owned() });
        shell.back();
    }
    // (Moves made this frame - keys, the stick, the mouse over a row - are
    // heard next frame against `heard`; a new screen makes no move sound.)
}

/// The screens are up while the front end is (`covers_lui`), and the
/// front end comes back from a match to the lobby he started it from.
fn shell_shown(mut shell: ResMut<Shell>, front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>, mut was: Local<bool>) {
    let on = enabled() && front.is_some();
    bridge::set_shown(on);
    if on && !*was && shell.played {
        shell.screen = Screen::Lobby;
        shell.tab = Tab::Lobby;
        shell.focus = 0;
    }
    *was = on;
}

pub(crate) fn window_scale(windows: &Query<&Window, With<PrimaryWindow>>) -> (Vec2, f32) {
    windows.single().map_or((Vec2::new(1920.0, 1080.0), 1.0), |w| {
        let size = Vec2::new(w.width(), w.height());
        (size, (size.y / 1080.0).max(0.3))
    })
}

pub(crate) fn register_shell(app: &mut App) {
    app.init_resource::<Shell>()
        .init_resource::<pics::ShellPics>()
        .add_systems(Startup, theme::load_fonts)
        .add_systems(
            Update,
            (
                shell_shown,
                online::poll_online,
                shell_input,
                options::sync_settings,
                background::shell_background,
                view::shell_view,
                loading::shell_loading,
                ingame::ingame_input,
                ingame::ingame_view,
                ingame::tag_hits,
            )
                .chain()
                .in_set(ClientSet::Ui),
        );
}
