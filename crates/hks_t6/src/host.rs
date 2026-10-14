//! bo2zm M4: the engine around Black Ops II's UI scripts, shared by the
//! game and the headless harness (`t6lua`): the scripts by name and
//! `require`, the `Engine` / `UIExpression` / `Dvar` tables the scripts
//! ask (answered from the values the host keeps: dvars, game type
//! settings, visibility bits, localized text, text widths), the LUI root
//! (`LUI.UIRoot`, sized like the engine's: 720 units high, centred), and
//! the frame step that advances animations and sends their
//! `transition_complete_<name>` events.
//!
//! A field the scripts ask that the host has no answer for is a function
//! that returns nothing, counted in `asked` (what to bind next).

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use crate::lui::{self, Rect};
use crate::value::{Table, TableRef, UserData, Value};
use crate::vm::{LuaError, Vm};

/// How wide a line of text is: (text, font file, height in units) ->
/// width in units.
pub type MeasureFn = dyn Fn(&str, &str, f32) -> f32;

/// The values the scripts read from the engine. The game writes these as
/// its state changes; the scripts read them when they run.
#[derive(Default)]
pub struct EngineValues {
    /// Dvars by name (`UIExpression.DvarString` / `DvarInt` / `DvarBool` /
    /// `DvarFloat`, `Dvar.<name>:get()`).
    pub dvars: HashMap<String, String>,
    /// `Engine.GetGametypeSetting(name)`.
    pub settings: HashMap<String, f32>,
    /// The settings as the game starts them (`Engine.IsGametypeSettingDefault`:
    /// Custom Games marks a row he changed).
    pub default_settings: HashMap<String, f32>,
    /// The visibility bits that are set (`CoD.BIT_*` numbers).
    pub bits: HashSet<i32>,
    /// bo2zm: his last input was a pad (`Engine.LastInput_Gamepad`): BO2's
    /// prompts show the pad's button pictures, not his keys.
    pub gamepad: bool,
    /// Localized text by key (`Engine.Localize`), upper-case keys.
    pub localize: HashMap<String, String>,
    /// String tables by name (`mp/zombiemode.csv`): rows of cells
    /// (`UIExpression.TableLookup`).
    pub tables: HashMap<String, Vec<Vec<String>>>,
    /// The local player's team number (`CoD.TEAM_ALLIES`).
    pub team: i32,
    /// The scoreboard's rows: name, then the columns' values.
    pub players: Vec<Vec<String>>,
    /// Each scoreboard row's client number.
    pub clients: Vec<i32>,
    /// The scoreboard's column names (`Engine.GetScoreBoardColumnName`).
    pub columns: Vec<String>,
    /// The safe area's size in root units (the root's own size).
    pub safe_area: (f32, f32),
    /// Each bound command's keys, as shown (`+actionslot 4` -> `4`).
    pub binds: HashMap<String, Vec<String>>,
    /// bo2mp vehicle screens: while he rides, the commands BO2 puts on the
    /// keys he has for a vehicle button's command (CG_UpdateVehicleBindings:
    /// `+vehiclemoveup` on the keys of `+frag` for the Dragonfire's
    /// RSHLDR), as `vehicle command -> command it takes its keys from`.
    pub vehicle_binds: HashMap<String, String>,
    /// The session's modes (`CoD.SESSIONMODE_*` numbers): an offline game.
    pub session_modes: Vec<f32>,
    /// An enum dvar's choices (`Dvar.<name>:getDomainEnumStrings()`).
    pub enums: HashMap<String, Vec<String>>,
    /// His profile settings (`Engine.SetProfileVar`, `UIExpression.Profile*`)
    /// and hardware profile (`Engine.*HardwareProfileValue*`).
    pub profile: HashMap<String, String>,
    /// bo2mp: the zones' config files by lower-case name, for the menus'
    /// `exec <file>` (`run_command`).
    pub configs: HashMap<String, String>,
    /// bo2mp: the stats commands those lines ran (`STAT_COMMANDS`), in
    /// order, for the stats' owner to carry out (`hks_t6::mp`).
    pub stat_commands: Vec<String>,
    /// bo2zm M4: his player stats by path (`Engine.GetPlayerStats(c)`'s
    /// `cacLoadouts.resetWarningDisplayed`), each 0 until set.
    pub stats: HashMap<String, Value>,
    /// bo2zm M4: the maps this game plays (`Engine.GetMaps`), load names.
    pub maps: Vec<String>,
    /// bo2zm M4: the front end's game modes (`Engine.GameModeSetMode`).
    pub game_modes: Vec<f32>,
    /// bo2zm M4: the playlist picked (`Engine.SetPlaylistID`), 0 = none.
    pub playlist: usize,
    /// bo2zm M4: these menus are the front end (not in a game).
    pub front_end: bool,
    /// bo2zm M4: his name (his settings' player name), the lobby's card.
    pub player_name: String,
    /// bo2zm: when (LUI ms) he entered a Public Match lobby
    /// (`xstartparty`); None outside one. It looks for players a while,
    /// counts down, then starts the match with him alone.
    pub public_lobby_since: Option<f64>,
    /// bo2zm H28: the other players in his Public Match lobby (their
    /// names), found through the relay.
    pub party: Vec<String>,
    /// bo2zm H28: he joined someone else's lobby: its host starts the match.
    pub party_follower: bool,
    /// bo2zm H28: his Friends list (`Engine.GetPlayerInfoByIndex` list 0):
    /// the other players' lobbies on the relay.
    pub friends: Vec<Friend>,
    /// bo2zm: each weapon the HUD was told of, by name, with its Black Ops
    /// II `weapType` (`Engine.IsWeaponType`: 7 melee, 4 gas).
    pub weapon_types: HashMap<String, i32>,
}

/// bo2zm H28: one row of his Friends list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Friend {
    /// The menus pass it back (`joinplayersessionbyxuid <xuid>`): a decimal
    /// string, never one of the lobby rows' ("1", "2", ...).
    pub xuid: String,
    pub name: String,
    /// The line under the name (where the friend is).
    pub status: String,
    /// His lobby has room and has not started: the row's JOIN.
    pub joinable: bool,
}

/// The table the Friends list and the player card read for one friend.
fn friend_row(f: &Friend) -> TableRef {
    let t = Table::new_ref();
    {
        let mut t = t.borrow_mut();
        for key in ["name", "gamertag", "clean_gamertag"] {
            t.set_str(key, Value::str(&f.name));
        }
        t.set_str("xuid", Value::str(&f.xuid));
        t.set_str("playerXuid", Value::str(&f.xuid));
        t.set_str("status", Value::str(&f.status));
        // A string, as the rows read it ("0" or nil hides the rank).
        t.set_str("rank", Value::str("1"));
        t.set_str("rankIcon", Value::str("menu_zm_rank_1"));
        t.set_str("prestige", Value::Num(0.0));
        t.set_str("clantag", Value::str(""));
        // The row's rank icon column (rankicontable_zm: 5 + days); nil is a
        // script error and the Friends list never opens.
        t.set_str("daysPlayedInLast5Days", Value::Num(0.0));
    }
    t
}

/// bo2mp: BO2's console commands that write his stats (mp/prestige_reset.cfg,
/// mp/reset_classes.cfg, mp/reset_classes_offline.cfg and the Prestige
/// menus' `Engine.ExecNow`): `run_command` queues them in
/// `EngineValues::stat_commands`.
pub const STAT_COMMANDS: [&str; 10] = [
    "equipdefaultclass",
    "equipdefaultclasstoprofile",
    "setstatfromlocstring",
    "setprofilelocclass",
    "statwriteddl",
    "prestigestatsreset",
    "prestigerequest",
    "prestigestatsresetall",
    "prestigerespec",
    "prestigeaddcac",
];

/// A config line's value: a number or word as written, or BO2's
/// `( dvarInt( <name> ) )` / `( dvarBool( <name> ) )` read from the dvars
/// (mp/prestige_reset.cfg puts `systemlink` back that way).
fn config_value(values: &RefCell<EngineValues>, text: &str) -> String {
    let t = text.trim().trim_matches('"');
    let squeezed: String = t.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_lowercase();
    for f in ["dvarint", "dvarbool"] {
        if let Some(name) = squeezed.strip_prefix(&format!("({f}(")).and_then(|r| r.strip_suffix("))")) {
            let v = values.borrow();
            let hit = v.dvars.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, x)| x.clone());
            let n = hit.and_then(|x| x.trim().parse::<f32>().ok()).unwrap_or(0.0);
            return format!("{}", n as i64);
        }
    }
    t.to_owned()
}

/// bo2mp: carry out a console command line the menus run
/// (`Engine.Exec` / `Engine.ExecNow`) that changes engine values here:
/// `exec <file>` runs that config file's lines, `set` / `seta` / `sets`
/// <dvar> <value> sets a dvar, a stats command (`STAT_COMMANDS`) is queued
/// for the stats' owner. Others (`reset`, `restoreDvars`, binds) are left
/// to the owner. Returns whether it ran one.
pub fn run_command(values: &RefCell<EngineValues>, line: &str) -> bool {
    run_command_depth(values, line, 0)
}

fn run_command_depth(values: &RefCell<EngineValues>, line: &str, depth: usize) -> bool {
    let mut ran = false;
    for cmd in line.split(';') {
        let cmd = cmd.split("//").next().unwrap_or("").trim();
        let mut words = cmd.split_whitespace();
        let Some(verb) = words.next() else { continue };
        match verb.to_ascii_lowercase().as_str() {
            "exec" => {
                let Some(file) = words.next() else { continue };
                let file = file.trim_matches('"').to_ascii_lowercase();
                let text = values.borrow().configs.get(&file).cloned();
                if let Some(text) = text
                    && depth < 8
                {
                    for l in text.lines() {
                        run_command_depth(values, l, depth + 1);
                    }
                    ran = true;
                }
            }
            // A game type's setting (mp/gamesettings_<type>.cfg lines,
            // run when the game type is set).
            "gametype_setting" => {
                let (Some(name), Some(value)) = (words.next(), words.next().and_then(|x| x.parse::<f32>().ok())) else {
                    continue;
                };
                let mut v = values.borrow_mut();
                v.settings.insert(name.to_owned(), value);
                v.settings.insert(name.to_ascii_lowercase(), value);
                ran = true;
            }
            "set" | "seta" | "sets" => {
                let Some(name) = words.next() else { continue };
                let value = config_value(values, &words.collect::<Vec<_>>().join(" "));
                values.borrow_mut().dvars.insert(name.to_owned(), value);
                ran = true;
            }
            v if STAT_COMMANDS.contains(&v) => {
                values.borrow_mut().stat_commands.push(cmd.to_owned());
                ran = true;
            }
            _ => {}
        }
    }
    ran
}

impl EngineValues {
    /// The keys a command is on: his own binds for it, else (in a vehicle)
    /// the keys of the command its vehicle button stands for.
    pub fn keys_of(&self, command: &str) -> Option<&Vec<String>> {
        self.binds
            .get(command)
            .or_else(|| self.vehicle_binds.get(command).and_then(|from| self.binds.get(from)))
    }
}

/// bo2zm: a Public Match lobby looks for players this long (ms) ...
pub const PUBLIC_SEARCH_MS: f64 = 6000.0;
/// ... then counts down this long (ms) and starts the match.
pub const PUBLIC_COUNTDOWN_MS: f64 = 10000.0;
/// A Zombies match holds four players.
pub const ZOMBIES_MAX_PLAYERS: usize = 4;

/// What the scripts asked the engine to do (the host's owner carries it
/// out): a menu answer for the server, a console command, a sound.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineCall {
    /// `Engine.SendMenuResponse(controller, menu, response)`.
    MenuResponse(String, String),
    /// `Engine.Exec(controller, command)`.
    Exec(String),
    /// `Engine.PlaySound(alias)`.
    PlaySound(String),
    /// `Engine.BindCommand(controller, command, index)`: bind the next key
    /// he presses (then send `key_bound`).
    BindCommand(String, usize),
    /// `Engine.BlurWorld(controller, amount)`: blur the world behind the
    /// menus (0 = none).
    BlurWorld(f32),
    /// `Engine.FetchLeagueTeams(...)`: the engine answers with
    /// `league_team_info_fetched`.
    FetchLeagueTeams,
    /// `Engine.PlayMenuMusic(alias)`: the menus' music (the front end's
    /// `mus_mp_frontend`).
    PlayMusic(String),
    /// An event the engine sends the root when something it was asked for
    /// is ready (Theater's film browser: `fileshare_slots_available`,
    /// `fileshare_search_complete`), with its number fields.
    Event(String, Vec<(String, f32)>),
}

/// One element as drawn: its rectangle in root units (0,0 = the root's top
/// left), alpha including its parents', and what it shows.
#[derive(Clone, Debug)]
pub struct Drawn {
    pub id: usize,
    pub kind: &'static str,
    pub rect: Rect,
    pub alpha: f32,
    pub rgb: [f32; 3],
    pub material: Option<String>,
    pub text: Option<String>,
    pub font: Option<String>,
    /// LUI.Alignment as drawn: 1 left, 2 centre, 3 right. A text that set
    /// none takes its box's anchors: pinned left only reads from the left,
    /// right only from the right, else from the middle (BO2's "+100"
    /// score popup and rank-up text, in boxes as wide as their parent;
    /// the "N POINTS TO WIN" line, a zero-width box pinned left).
    pub alignment: i32,
    /// Horizontal anchors (left, right): where unaligned text sits.
    pub anchors: (bool, bool),
    pub z_rot: f32,
    /// bo2mp: turned out of the screen's plane (a 3D widget: the front
    /// end's holotable grid).
    pub x_rot: f32,
    pub y_rot: f32,
    /// A dashes bar's (count, lit, change) and units per dash (bo2zm: also
    /// Settings' voice `meter`, change 0).
    pub dashes: (i32, i32, i32),
    pub dash_pitch: f32,
    /// `setupTiles(n)`: the picture repeats in tiles n units wide.
    pub tiles: f32,
    /// bo2zm M4: the material's shader vectors (`setShaderVector`); bo2mp:
    /// vector 0's x is the multiplayer globe's reveal (0 hidden, 1
    /// wireframe, 2 map in).
    pub shader: [[f32; 4]; 4],
    /// What is drawn under it shows blurred (`setBlur`).
    pub blur: bool,
    /// bo2mp: the main menu covered by Options or the Quit prompt: drawn
    /// blurred and dimmed, without its text.
    pub behind: bool,
    /// The box it is clipped to (its ancestors' `setUseStencil`), root
    /// units; None = not clipped.
    pub clip: Option<Rect>,
}

pub struct Host {
    pub vm: Vm,
    pub values: Rc<RefCell<EngineValues>>,
    /// Engine fields the scripts asked that nothing answers, with counts.
    pub asked: Rc<RefCell<BTreeMap<String, usize>>>,
    /// Modules `require` found no script for.
    pub missing: Rc<RefCell<Vec<String>>>,
    /// Engine calls since the owner last took them (`take_calls`).
    calls: Rc<RefCell<Vec<EngineCall>>>,
    /// The engine's roots: `UIRoot0` (the player's) and `UIRootFull`
    /// (the whole screen), drawn in that order.
    roots: Vec<Value>,
    /// Root size in units (720 high; the width follows the screen).
    root_size: (f32, f32),
    pub errors: Vec<String>,
    /// Steps one entry into the scripts may take.
    pub step_budget: u64,
    /// Text width in root units (font, height): GetTextDimensions and
    /// tight text.
    measure: Rc<MeasureFn>,
}

/// A script name as a lookup key: lower case, no separators, no
/// extension (`ui_mp/t6/hud.lua`, `ui_mp_t6_hud.lua` and
/// `ui_mp__t6__hud.lua` are one key).
fn script_key(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".lua").unwrap_or(&lower);
    stem.chars()
        .filter(|c| !matches!(c, '/' | '\\' | '_' | '.'))
        .collect()
}

/// bo2zm M4: buttons the menus leave out, as (label, event): the main
/// menu's Campaign (his 10-06 call: "No campaign button"; there is no
/// campaign here), and the lobbies' COD TV and Leaderboards (his 10-06
/// call: "COD TV, we can just remove that. Leaderboard, we could probably
/// just remove that"; both need BO2's online service). Theater has its
/// own COD TV (event `open_codtv`).
const DROPPED_BUTTONS: [(&str, &str); 4] = [
    ("MENU_SINGLEPLAYER_CAPS", "open_sp_switch_popup"),
    ("MENU_COD_TV_CAPS", "open_cod_tv"),
    ("MENU_COD_TV_CAPS", "open_codtv"),
    ("MPUI_LEADERBOARDS_CAPS", "open_barracks"),
];

/// bo2zm M4: the gap the Zombies lobby puts above COD TV and Leaderboards
/// goes with them (else two gaps meet above Options).
const DROPPED_SPACERS: [&str; 1] = ["theaterSpacer"];

/// A jump from `start` to `end` in place of the op at `start`: the run
/// between becomes as if the script never had it.
fn jump_over(p: &mut crate::Proto, start: usize, end: usize) {
    let jmp = crate::OP_NAMES.iter().position(|n| *n == "JMP").unwrap_or(28) as u32;
    let sbx = (end as i64 - (start as i64 + 1) + 0xFFFF) as u32 & 0x1_FFFF;
    p.code[start] = (jmp << 25) | (sbx << 8);
}

/// bo2zm M4: leave out the menus' [`DROPPED_BUTTONS`] and
/// [`DROPPED_SPACERS`] in `p` and every function inside it. BO2's scripts
/// add a button as `buttonList:addButton(Engine.Localize(label), ...)`
/// and give it its action with `setActionEventName(event)`; that run
/// becomes one jump over it (the list lays out and links the rest as it
/// does for any button it leaves out). A spacer is `body.<name> =
/// buttonList:addSpacer(...)`; that run goes the same way, leaving the
/// field empty.
fn drop_buttons(p: &mut crate::Proto) {
    let is_str = |p: &crate::Proto, k: usize, want: &str| {
        matches!(p.consts.get(k), Some(crate::Const::String(s)) if s.as_slice() == want.as_bytes())
    };
    for (label, event) in DROPPED_BUTTONS {
        let ops: Vec<crate::Ins> = p.code.iter().map(|w| crate::decode(*w)).collect();
        let name = |i: usize| crate::op_name(ops[i].op);
        let Some(l) = (0..ops.len())
            .find(|&i| name(i) == "LOADK" && is_str(p, ops[i].bx as usize, label))
        else {
            continue;
        };
        let start = (0..l).rev().take(16).find(|&i| {
            name(i).starts_with("GETFIELD") && is_str(p, ops[i].c as usize, "buttonList")
        });
        let end = (l..ops.len())
            .take(40)
            .find(|&i| name(i) == "LOADK" && is_str(p, ops[i].bx as usize, event))
            .and_then(|e| {
                (e..ops.len()).take(4).find(|&i| name(i).starts_with("CALL")).map(|i| i + 1)
            });
        if let (Some(start), Some(end)) = (start, end) {
            jump_over(p, start, end);
        }
    }
    for field in DROPPED_SPACERS {
        let ops: Vec<crate::Ins> = p.code.iter().map(|w| crate::decode(*w)).collect();
        let name = |i: usize| crate::op_name(ops[i].op);
        let Some(set) = (0..ops.len())
            .find(|&i| name(i).starts_with("SETFIELD") && is_str(p, ops[i].b as usize, field))
        else {
            continue;
        };
        let spacer = (0..set).rev().take(16).find(|&i| {
            name(i) == "SELF" && is_str(p, (ops[i].c & 0xFF) as usize, "addSpacer")
        });
        let start = spacer.and_then(|k| {
            (0..k).rev().take(4).find(|&i| {
                name(i).starts_with("GETFIELD") && is_str(p, ops[i].c as usize, "buttonList")
            })
        });
        if let Some(start) = start {
            jump_over(p, start, set + 1);
        }
    }
    // Theater's half-height gap above COD TV (no field: `addSpacer` just
    // before its `Engine.IsBetaBuild()` check) goes with COD TV.
    if p.consts.iter().any(|k| matches!(k, crate::Const::String(s) if s.as_slice() == b"open_codtv")) {
        let ops: Vec<crate::Ins> = p.code.iter().map(|w| crate::decode(*w)).collect();
        let name = |i: usize| crate::op_name(ops[i].op);
        let gap = (0..ops.len()).find_map(|i| {
            if !(name(i) == "SELF" && is_str(p, (ops[i].c & 0xFF) as usize, "addSpacer")) {
                return None;
            }
            let end = (i..ops.len()).take(12).find(|&j| name(j).starts_with("CALL"))? + 1;
            (end..ops.len()).take(4).find(|&j| {
                name(j).starts_with("GETFIELD") && is_str(p, ops[j].c as usize, "IsBetaBuild")
            })?;
            let start = (0..i).rev().take(4).find(|&j| {
                name(j).starts_with("GETFIELD") && is_str(p, ops[j].c as usize, "buttonList")
            })?;
            Some((start, end))
        });
        if let Some((start, end)) = gap {
            jump_over(p, start, end);
        }
        // Its focus calls on the dropped buttons
        // (`body.codtvButton:processEvent({name = "gain_focus"})`) go too.
        loop {
            let ops: Vec<crate::Ins> = p.code.iter().map(|w| crate::decode(*w)).collect();
            let name = |i: usize| crate::op_name(ops[i].op);
            let found = (0..ops.len()).find_map(|g| {
                let dropped = ["codtvButton", "barracksButton"];
                if !(name(g) == "GETFIELD_R1" && dropped.iter().any(|f| is_str(p, ops[g].c as usize, f))) {
                    return None;
                }
                (g..ops.len()).take(4).find(|&i| {
                    name(i) == "SELF" && is_str(p, (ops[i].c & 0xFF) as usize, "processEvent")
                })?;
                let start = (0..g).rev().take(4).find(|&i| {
                    name(i).starts_with("GETFIELD") && is_str(p, ops[i].c as usize, "body")
                })?;
                let end = (g..ops.len()).take(10).find(|&i| name(i).starts_with("CALL"))? + 1;
                Some((start, end))
            });
            let Some((start, end)) = found else { break };
            jump_over(p, start, end);
        }
    }
    // The lobby puts its list back together when the party changes
    // (`buttonList:addElement(body.theaterSpacer)`): a dropped spacer is
    // left out there too.
    for field in DROPPED_SPACERS {
        loop {
            let ops: Vec<crate::Ins> = p.code.iter().map(|w| crate::decode(*w)).collect();
            let name = |i: usize| crate::op_name(ops[i].op);
            let found = (0..ops.len()).find_map(|g| {
                if !(name(g) == "GETFIELD_R1" && is_str(p, ops[g].c as usize, field)) {
                    return None;
                }
                let add = (0..g).rev().take(8).find(|&i| {
                    name(i) == "SELF" && is_str(p, (ops[i].c & 0xFF) as usize, "addElement")
                })?;
                let start = (0..add).rev().take(4).find(|&i| {
                    name(i).starts_with("GETFIELD") && is_str(p, ops[i].c as usize, "buttonList")
                })?;
                let end = (g..ops.len()).take(4).find(|&i| name(i).starts_with("CALL"))? + 1;
                Some((start, end))
            });
            let Some((start, end)) = found else { break };
            jump_over(p, start, end);
        }
        // ... and so is its `body.theaterSpacer:closeAndRefocus(...)` when he
        // joined someone else's party (a nil there is a script error).
        loop {
            let ops: Vec<crate::Ins> = p.code.iter().map(|w| crate::decode(*w)).collect();
            let name = |i: usize| crate::op_name(ops[i].op);
            let found = (0..ops.len()).find_map(|g| {
                if !(name(g) == "GETFIELD_R1" && is_str(p, ops[g].c as usize, field)) {
                    return None;
                }
                let call = (g..ops.len()).take(4).find(|&i| name(i) == "SELF" && ops[i].b == ops[g].a as u16)?;
                let end = (call..ops.len()).take(10).find(|&i| name(i).starts_with("CALL"))? + 1;
                Some((g, end))
            });
            let Some((start, end)) = found else { break };
            jump_over(p, start, end);
        }
    }
    for c in p.protos.iter_mut() {
        drop_buttons(std::rc::Rc::make_mut(c));
    }
}

fn arg(a: &[Value], i: usize) -> Value {
    a.get(i).cloned().unwrap_or(Value::Nil)
}

/// The mouse keys BO2 shows as a button picture in the Controls rows, with
/// the FontIcon entry that is its glyph (BO2's own icon set names the
/// picture and its size; the text carries `^B<entry>^`).
pub const MOUSE_GLYPHS: [(&str, &str); 5] = [
    ("MOUSE1", "mouseButtonLeft"),
    ("MOUSE2", "mouseButtonRight"),
    ("MOUSE3", "mouseButtonMiddle"),
    ("MWHEELUP", "mouseWheelUp"),
    ("MWHEELDOWN", "mouseWheelDown"),
];

impl Host {
    /// A VM with the standard library, LUI's natives and the engine tables,
    /// and `scripts` ((rawfile name, bytes); a later name wins) behind
    /// `require`. Nothing runs yet (`load_base`).
    #[allow(clippy::too_many_lines)]
    pub fn new(scripts: Vec<(String, Vec<u8>)>, measure: Box<MeasureFn>) -> Host {
        let measure: Rc<MeasureFn> = Rc::from(measure);
        let measure_out = measure.clone();
        let mut vm = Vm::new();
        crate::stdlib::open(&mut vm);
        let asked: Rc<RefCell<BTreeMap<String, usize>>> = Rc::default();
        let values: Rc<RefCell<EngineValues>> = Rc::default();
        let missing: Rc<RefCell<Vec<String>>> = Rc::default();
        // What the menus ask of the engine, queued for the owner.
        let calls: Rc<RefCell<Vec<EngineCall>>> = Rc::default();
        for name in ["Engine", "UIExpression"] {
            let v = stub_table(&mut vm, name, asked.clone());
            vm.set_global(name, v);
        }
        lui::install(&mut vm);

        // Dvar.<name>: :get() reads the host's dvar, :set(v) writes it.
        {
            let dvars = Table::new_ref();
            let meta = Table::new_ref();
            let vals = values.clone();
            let index = vm.native("dvar_index", move |vm, a| {
                let name = arg(&a, 1).to_string();
                let d = Table::new_ref();
                let (v1, v2) = (vals.clone(), vals.clone());
                let n1 = name.clone();
                let get = vm.native("dvar_get", move |_, _| {
                    // bo2zm: any spelling of its name (`dvar_get`).
                    let v = dvar_get(&v1.borrow().dvars, &n1);
                    // A colour dvar ("0.6 0.64 0.69") answers its r, g, b (the team colours the
                    // scripts read as `local r, g, b = Dvar.g_TeamColor_MyTeam:get()`).
                    if n1.starts_with("g_TeamColor_")
                        && let Some(rgb) = v.as_deref().map(|s| s.split_whitespace().filter_map(|x| x.parse::<f32>().ok()).collect::<Vec<_>>()).filter(|c| c.len() >= 3)
                    {
                        return Ok(rgb.into_iter().take(3).map(Value::Num).collect());
                    }
                    // (The engine's boolean dvars answer true/false; the
                    // scripts compare them with `== true`.)
                    let boolean = matches!(
                        n1.as_str(),
                        "ui_inGameStoreVisible" | "demo_recordPrivateMatch" | "tu7_scoreboardPingAsNumbers" | "party_isPreviousMapVoted"
                    );
                    Ok(v.map(|s| match s.parse::<f32>() {
                        Ok(n) if boolean => Value::Bool(n != 0.0),
                        Ok(n) => Value::Num(n),
                        Err(_) => Value::str(&s),
                    })
                    .into_iter()
                    .collect())
                });
                let set = vm.native("dvar_set", move |_, a| {
                    dvar_set(&mut v2.borrow_mut().dvars, &name, arg(&a, 1).to_string());
                    Ok(vec![])
                });
                // getDomainEnumStrings(): an enum dvar's choices
                // (`enums`, e.g. r_mode's resolutions).
                let v3 = vals.clone();
                let n3 = arg(&a, 1).to_string();
                let domain = vm.native("dvar_domain", move |_, _| {
                    let t = Table::new_ref();
                    for (i, c) in v3.borrow().enums.get(&n3).into_iter().flatten().enumerate() {
                        t.borrow_mut()
                            .set(Value::Num((i + 1) as f32), Value::str(c));
                    }
                    Ok(vec![Value::Table(t)])
                });
                d.borrow_mut().set_str("get", get);
                d.borrow_mut().set_str("set", set);
                d.borrow_mut().set_str("getDomainEnumStrings", domain);
                Ok(vec![Value::Table(d)])
            });
            meta.borrow_mut().set_str("__index", index);
            dvars.borrow_mut().meta = Some(meta);
            vm.set_global("Dvar", Value::Table(dvars));
        }

        // UIExpression: dvars and visibility bits.
        let ui = vm.global("UIExpression");
        let pad = vm.native("IsUsingGamepad", |_, _| {
            Ok(vec![Value::Num(if gamepad_active() { 1.0 } else { 0.0 })])
        });
        set_field(&ui, "IsUsingGamepad", pad);
        let dvar = |vals: &Rc<RefCell<EngineValues>>, a: &[Value]| {
            dvar_get(&vals.borrow().dvars, &arg(a, 1).to_string()).unwrap_or_default()
        };
        let v = values.clone();
        let f = vm.native("DvarString", move |_, a| {
            Ok(vec![Value::str(&dvar(&v, &a))])
        });
        set_field(&ui, "DvarString", f);
        for name in ["DvarInt", "DvarFloat"] {
            let v = values.clone();
            let f = vm.native(name, move |_, a| {
                Ok(vec![Value::Num(
                    dvar(&v, &a).trim().parse::<f32>().unwrap_or(0.0),
                )])
            });
            set_field(&ui, name, f);
        }
        let v = values.clone();
        let f = vm.native("DvarBool", move |_, a| {
            let s = dvar(&v, &a);
            let on =
                matches!(s.trim(), "1" | "true") || s.trim().parse::<f32>().is_ok_and(|n| n != 0.0);
            Ok(vec![Value::Num(if on { 1.0 } else { 0.0 })])
        });
        set_field(&ui, "DvarBool", f);
        let v = values.clone();
        let f = vm.native("IsVisibilityBitSet", move |_, a| {
            let bit = arg(&a, 1).as_num().map_or(-1, |n| n as i32);
            Ok(vec![Value::Num(if v.borrow().bits.contains(&bit) {
                1.0
            } else {
                0.0
            })])
        });
        set_field(&ui, "IsVisibilityBitSet", f);
        // ToUpper(controller, text).
        let f = vm.native("ToUpper", |_, a| {
            let text = if a.len() >= 2 { arg(&a, 1) } else { arg(&a, 0) };
            Ok(vec![Value::str(&text.to_string().to_uppercase())])
        });
        set_field(&ui, "ToUpper", f);
        // A zombies game on PC (the multiplayer exe runs zombies).
        let f = vm.native("GetCurrentPlatform", |_, _| Ok(vec![Value::str("pc")]));
        set_field(&ui, "GetCurrentPlatform", f);
        let f = vm.native("GetCurrentExe", |_, _| Ok(vec![Value::str("multiplayer")]));
        set_field(&ui, "GetCurrentExe", f);
        let f = vm.native("SessionMode_IsZombiesGame", |_, _| {
            Ok(vec![Value::Num(1.0)])
        });
        set_field(&ui, "SessionMode_IsZombiesGame", f);
        // TableLookup(controller, table, column, value[, column, value...],
        // return column): the first row whose cells match every pair
        // (numbers as their text). `maps/<x>` (CoD.mapsTable) is the zombies
        // game's `zm/<x>`.
        let v = values.clone();
        let f = vm.native("TableLookup", move |_, a| {
            // The tables are kept by lowercase name ("mp/rankIconTable_zm.csv").
            let table = arg(&a, 1).to_string().replace('\\', "/").to_ascii_lowercase();
            let rest: Vec<Value> = a.iter().skip(2).cloned().collect();
            let Some((ret, pairs)) = rest.split_last() else {
                return Ok(vec![Value::str("")]);
            };
            let ret = ret.as_num().map_or(0, |n| n as usize);
            let pairs: Vec<(usize, String)> = pairs
                .chunks_exact(2)
                .map(|p| (p[0].as_num().map_or(0, |n| n as usize), p[1].to_string()))
                .collect();
            let v = v.borrow();
            // Table names are case-blind (`mp/rankIconTable.csv`).
            let rows = v.tables.get(&table).or_else(|| {
                v.tables
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&table))
                    .map(|(_, r)| r)
                    .or_else(|| {
                        table
                            .strip_prefix("maps/")
                            .and_then(|x| v.tables.get(&format!("zm/{x}")))
                    })
            });
            let hit = rows
                .and_then(|rows| {
                    rows.iter().find(|r| {
                        pairs
                            .iter()
                            .all(|(c, want)| r.get(*c).is_some_and(|x| x == want))
                    })
                })
                .and_then(|r| r.get(ret).cloned())
                .unwrap_or_default();
            Ok(vec![Value::str(&hit)])
        });
        set_field(&ui, "TableLookup", f);
        // FormatNumberWithCommas(controller, n): `1,234`.
        let f = vm.native("FormatNumberWithCommas", |_, a| {
            let n = arg(&a, 1).as_num().unwrap_or(0.0).round() as i64;
            let digits = n.unsigned_abs().to_string();
            let mut out = String::new();
            for (i, c) in digits.chars().enumerate() {
                if i > 0 && (digits.len() - i) % 3 == 0 {
                    out.push(',');
                }
                out.push(c);
            }
            if n < 0 {
                out.insert(0, '-');
            }
            Ok(vec![Value::str(&out)])
        });
        set_field(&ui, "FormatNumberWithCommas", f);
        // The maps table a zombies game reads (CoD.mapsTable).
        let f = vm.native("GetCurrentMapTableName", |_, _| {
            Ok(vec![Value::str("zm/mapstable.csv")])
        });
        set_field(&ui, "GetCurrentMapTableName", f);

        // Engine: the safe area, settings, localized text.
        let engine = vm.global("Engine");
        // The safe area: the whole root (a PC screen has no overscan), in
        // root units. ForController: its edges around the centre;
        // GetUserSafeArea: its width and height.
        let v = values.clone();
        let f = vm.native("GetUserSafeAreaForController", move |_, _| {
            let (w, h) = v.borrow().safe_area;
            Ok(vec![
                Value::Num(-w / 2.0),
                Value::Num(-h / 2.0),
                Value::Num(w / 2.0),
                Value::Num(h / 2.0),
            ])
        });
        set_field(&engine, "GetUserSafeAreaForController", f);
        let v = values.clone();
        let f = vm.native("GetUserSafeArea", move |_, _| {
            let (w, h) = v.borrow().safe_area;
            Ok(vec![Value::Num(w), Value::Num(h)])
        });
        set_field(&engine, "GetUserSafeArea", f);
        let v = values.clone();
        let f = vm.native("GetGametypeSetting", move |_, a| {
            Ok(vec![Value::Num(
                v.borrow()
                    .settings
                    .get(&arg(&a, 0).to_string())
                    .copied()
                    .unwrap_or(0.0),
            )])
        });
        set_field(&engine, "GetGametypeSetting", f);
        // SetGametypeSetting(name, value): Custom Games' options.
        let v = values.clone();
        let f = vm.native("SetGametypeSetting", move |_, a| {
            if let Some(x) = arg(&a, 1).as_num() {
                v.borrow_mut().settings.insert(arg(&a, 0).to_string(), x);
            }
            Ok(vec![])
        });
        set_field(&engine, "SetGametypeSetting", f);
        let v = values.clone();
        let f = vm.native("IsGametypeSettingDefault", move |_, a| {
            let name = arg(&a, 0).to_string();
            let v = v.borrow();
            Ok(vec![Value::Bool(
                v.settings.get(&name) == v.default_settings.get(&name),
            )])
        });
        set_field(&engine, "IsGametypeSettingDefault", f);
        let v = values.clone();
        let strict = std::env::var_os("BO2MP_STRICT_LOCALIZE").is_some();
        let f = vm.native("Localize", move |_, a| {
            // BO2MP_STRICT_LOCALIZE=1: Localize(nil) is an error with its
            // trace (debugging aid: which engine answer came back nil).
            if strict && matches!(arg(&a, 0), Value::Nil) {
                return Err(LuaError::new("Localize(nil)".to_owned()));
            }
            let key = arg(&a, 0).to_string();
            // A "@KEY" reference (the notices' dvars hold them, set by the
            // engine's party code) is the same KEY.
            let up = key.strip_prefix('@').unwrap_or(&key).to_ascii_uppercase();
            let mut text = v
                .borrow()
                .localize
                .get(&up)
                .cloned()
                .or_else(|| caps_word(&up))
                .unwrap_or(key);
            // "&&1".. take the arguments; an argument that is a text key is
            // its text (Create-a-Class's camo challenges pass the weapon's
            // name key: "Get 5 Headshot medals with MTAR.").
            for (i, x) in a.iter().enumerate().skip(1) {
                let s = x.to_string();
                let s = match x {
                    Value::Str(_) => v.borrow().localize.get(&s.to_ascii_uppercase()).cloned().unwrap_or(s),
                    _ => s,
                };
                text = text.replace(&format!("&&{i}"), &s);
            }
            // bo2mp vehicle screens: a `[{+command}]` in the text is the key
            // his binds put that command on, by its KEY_* name ("Hold [F] to
            // Exit", the VTOL Warship's MP_CHOPPER_GUNNER_* prompts); a
            // command with no key shows BO2's own KEY_UNBOUND text
            // ("UNBOUND", the word its Controls rows show for one). The
            // sticks are keyboard-and-mouse words: the look stick is the
            // mouse (MENU_MOUSE_LOOK, the Hellstorm's "Steer"), the move
            // stick the four movement keys.
            let key_name = |cmd: &str| {
                let key = v.borrow().keys_of(cmd).and_then(|k| k.first().cloned());
                match key {
                    Some(key) => {
                        let up = key.to_uppercase();
                        v.borrow().localize.get(&format!("KEY_{up}")).cloned().unwrap_or(up)
                    }
                    None => v.borrow().localize.get("KEY_UNBOUND").cloned().unwrap_or_else(|| "UNBOUND".to_owned()),
                }
            };
            let mut from = 0;
            while let Some(start) = text[from..].find("[{").map(|s| s + from) {
                let Some(len) = text[start..].find("}]") else { break };
                let cmd = text[start + 2..start + len].to_owned();
                let name = if cmd.eq_ignore_ascii_case("+lookstick") {
                    v.borrow().localize.get("MENU_MOUSE_LOOK").cloned().unwrap_or_else(|| "Mouse Look".to_owned())
                } else if cmd.eq_ignore_ascii_case("+movestick") {
                    ["+forward", "+back", "+moveleft", "+moveright"].map(key_name).join(",")
                } else {
                    key_name(&cmd)
                };
                text.replace_range(start..start + len + 2, &format!("[{name}]"));
                from = start + name.len() + 2;
            }
            Ok(vec![Value::str(&text)])
        });
        set_field(&engine, "Localize", f);
        let falsy = vm.native("false", |_, _| Ok(vec![Value::Bool(false)]));
        // (GameModeIsMode: bo2zm's below; bo2mp's `mp` binds its own.)
        for name in ["IsSplitscreen", "IsDemoShoutcaster", "IsInGame"] {
            set_field(&engine, name, falsy.clone());
        }
        // SessionModeIsMode(mode): the session's modes (`session_modes`).
        let v = values.clone();
        let f = vm.native("SessionModeIsMode", move |_, a| {
            let m = arg(&a, 0).as_num().unwrap_or(f32::NAN);
            // bo2zm: a Zombies session always (CoD.SESSIONMODE_ZOMBIES, 4);
            // bo2mp: in the multiplayer front end only when set.
            let zombies = m == 4.0 && !lui::mp_front_end();
            Ok(vec![Value::Bool(
                zombies || v.borrow().session_modes.contains(&m),
            )])
        });
        set_field(&engine, "SessionModeIsMode", f);
        // bo2zm M4: the front end's game modes (CoD.GAMEMODE_*: public
        // match 0 - Solo is one -, private match 1, ...) and session modes
        // (CoD.SESSIONMODE_*: offline 0, system link 1, online 2, private 3),
        // as its menus set and ask them.
        let v = values.clone();
        let f = vm.native("GameModeIsMode", move |_, a| {
            let m = arg(&a, 0).as_num().unwrap_or(f32::NAN);
            Ok(vec![Value::Bool(v.borrow().game_modes.contains(&m))])
        });
        set_field(&engine, "GameModeIsMode", f);
        let v = values.clone();
        let f = vm.native("GameModeSetMode", move |_, a| {
            let m = arg(&a, 0).as_num().unwrap_or(f32::NAN);
            let on = a.len() < 2 || arg(&a, 1).truthy();
            let mut v = v.borrow_mut();
            v.game_modes.retain(|x| *x != m);
            if on {
                v.game_modes.push(m);
            }
            party_size(&mut v);
            Ok(vec![])
        });
        set_field(&engine, "GameModeSetMode", f);
        let v = values.clone();
        let f = vm.native("GameModeResetModes", move |_, _| {
            let mut v = v.borrow_mut();
            v.game_modes.clear();
            party_size(&mut v);
            Ok(vec![])
        });
        set_field(&engine, "GameModeResetModes", f);
        for (name, mode) in [
            ("SessionModeSetOffline", 0.0),
            ("SessionModeSetSystemLink", 1.0),
            ("SessionModeSetOnlineGame", 2.0),
            ("SessionModeSetPrivate", 3.0),
        ] {
            let v = values.clone();
            let f = vm.native(name, move |_, a| {
                let on = a.is_empty() || arg(&a, 0).truthy();
                let mut v = v.borrow_mut();
                v.session_modes.retain(|x| *x != mode);
                if on {
                    v.session_modes.push(mode);
                }
                Ok(vec![])
            });
            set_field(&engine, name, f);
        }
        let v = values.clone();
        let f = vm.native("SessionModeResetModes", move |_, _| {
            v.borrow_mut().session_modes.clear();
            Ok(vec![])
        });
        set_field(&engine, "SessionModeResetModes", f);
        let zero = vm.native("zero", |_, _| Ok(vec![Value::Num(0.0)]));
        set_field(&engine, "GetAspectRatio", zero.clone());
        // bo2zm H28: the Friends list (`CoD.playerListType.friend`, 0) holds
        // the relay's other lobbies; the other lists (recent players, game
        // invites) are empty.
        let v = values.clone();
        let f = vm.native("GetPlayerCount", move |_, a| {
            let n = if arg(&a, 1).as_num() == Some(0.0) {
                v.borrow().friends.len()
            } else {
                0
            };
            Ok(vec![Value::Num(n as f32)])
        });
        set_field(&engine, "GetPlayerCount", f);
        let v = values.clone();
        let f = vm.native("GetPlayerInfoByIndex", move |_, a| {
            let v = v.borrow();
            let i = arg(&a, 1).as_num().unwrap_or(-1.0);
            let row = (arg(&a, 2).as_num() == Some(0.0) && i >= 0.0)
                .then(|| v.friends.get(i as usize))
                .flatten();
            Ok(vec![row.map_or(Value::Nil, |f| Value::Table(friend_row(f)))])
        });
        set_field(&engine, "GetPlayerInfoByIndex", f);
        // The list redrawn keeps the picked row: where that friend is now
        // (from 0; the first row once he is gone, the script's default).
        let v = values.clone();
        let f = vm.native("GetPlayerIndexByXuid", move |_, a| {
            let x = arg(&a, 1).to_string();
            let i = (arg(&a, 2).as_num() == Some(0.0))
                .then(|| v.borrow().friends.iter().position(|f| f.xuid == x))
                .flatten()
                .unwrap_or(0);
            let t = Table::new_ref();
            t.borrow_mut().set_str("index", Value::Num(i as f32));
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetPlayerIndexByXuid", f);
        // Nobody on the list is already in his party (the JOIN button).
        let f = vm.native("IsMemberInParty", |_, _| Ok(vec![Value::Bool(false)]));
        set_field(&engine, "IsMemberInParty", f);
        // CoD.canJoinSession: a friend's lobby with room is FRIEND_JOINABLE (1).
        let v = values.clone();
        let f = vm.native("IsPlayerJoinable", move |_, a| {
            let x = arg(&a, 1).to_string();
            let open = v.borrow().friends.iter().any(|f| f.xuid == x && f.joinable);
            Ok(vec![Value::Num(if open { 1.0 } else { 0.0 })])
        });
        set_field(&ui, "IsPlayerJoinable", f);
        set_field(&engine, "PartyConnectingToDedicated", falsy.clone());
        // One local player (the front end's per-controller loops).
        // Offline, solo: nobody is signed in to an online service.
        for name in [
            "AnySignedInToLive",
            "IsSignedInToLive",
            "IsSignedInToDemonware",
        ] {
            set_field(&engine, name, falsy.clone());
            set_field(&ui, name, falsy.clone());
        }
        let one_pad = vm.native("one_controller", |_, _| Ok(vec![Value::Num(1.0)]));
        for name in [
            "GetMaxControllerCount",
            "GetMaxLocalControllers",
            "GetUsedControllerCount",
        ] {
            set_field(&engine, name, one_pad.clone());
            set_field(&ui, name, one_pad.clone());
        }
        // ONLINE's checks (the PC front end's online service): the
        // connection is up, nothing is banned, the Steam ticket and the
        // service's fetches are done, every rating allowed.
        let truthy = vm.native("true", |_, _| Ok(vec![Value::Bool(true)]));
        set_field(&engine, "CheckNetConnection", truthy.clone());
        // The lobby rows show his rank icon (not hidden as a stranger's).
        set_field(&engine, "PartyShowTruePlayerInfo", truthy.clone());
        // A lobby row's player popup (CoD.Lobby.PlayerSelected) opens only
        // when this is true: KICK, MUTE and the rest are on it.
        set_field(&engine, "PartyShowTruePlayerInfoByXuid", truthy.clone());
        // His own xuid is his lobby row's ("1"): no KICK or REPORT on
        // himself (CoD.canKickPlayer); no one here is a guest.
        let f = vm.native("GetXUID", |_, _| Ok(vec![Value::str("1")]));
        set_field(&ui, "GetXUID", f);
        set_field(&engine, "IsGuestByXuid", falsy.clone());
        // The popup's MUTE (codbase's mute check reads .inCurrentSession):
        // everyone on a lobby row is in his session.
        let v = values.clone();
        let f = vm.native("IsPlayerInCurrentSession", move |_, a| {
            let x = arg(&a, 1).to_string();
            let inside = lobby_row(&v.borrow(), &x).is_some();
            let t = Table::new_ref();
            t.borrow_mut().set_str("inCurrentSession", Value::Bool(inside));
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "IsPlayerInCurrentSession", f);
        for name in ["IsFeatureBanned", "IsVacBanned", "WaitingForSteamTicket"] {
            set_field(&engine, name, falsy.clone());
        }
        for name in ["IsDemonwareFetchingDone", "IsContentRatingAllowed"] {
            set_field(&ui, name, one_pad.clone());
        }
        set_field(&ui, "IsAnyControllerMPRestricted", zero.clone());
        // Not a LAN game: the lobby rows show his rank icon (alpha 0 there).
        set_field(&ui, "SessionMode_IsSystemlinkGame", zero.clone());
        // His party (`xstartprivateparty`): he hosts it unless he joined
        // someone's lobby (bo2zm H28), and is alone in it until others come.
        for name in ["InPrivateParty", "PrivatePartyHostInLobby"] {
            set_field(&ui, name, one_pad.clone());
        }
        let v = values.clone();
        let f = vm.native("PrivatePartyHost", move |_, _| {
            Ok(vec![Value::Num(if v.borrow().party_follower { 0.0 } else { 1.0 })])
        });
        set_field(&ui, "PrivatePartyHost", f);
        // BO2's game lobby (g_lobbyData.inParty / .areWeHost): in a friend's
        // he is in it and not its host, so his Custom Games lobby has no
        // START MATCH (CoD.PrivateGameLobby.IsHost). A host with others in
        // his lobby is in one too: the KICK on their rows (canKickPlayer).
        let v = values.clone();
        let f = vm.native("InLobby", move |_, _| {
            let v = v.borrow();
            let inlobby = v.party_follower || !v.party.is_empty();
            Ok(vec![Value::Num(if inlobby { 1.0 } else { 0.0 })])
        });
        set_field(&ui, "InLobby", f);
        let v = values.clone();
        let f = vm.native("GameHost", move |_, _| {
            Ok(vec![Value::Num(if v.borrow().party_follower { 0.0 } else { 1.0 })])
        });
        set_field(&ui, "GameHost", f);
        let v = values.clone();
        let f = vm.native("AloneInPartyIgnoreSplitscreen", move |_, _| {
            Ok(vec![Value::Num(if v.borrow().party.is_empty() { 1.0 } else { 0.0 })])
        });
        set_field(&ui, "AloneInPartyIgnoreSplitscreen", f);
        // Him plus the others in his Public Match lobby (bo2zm H28: the
        // lobby's "2 Players (4 Max)" title and its rows count these).
        let v = values.clone();
        let party_count = vm.native("party_count", move |_, _| {
            Ok(vec![Value::Num((1 + v.borrow().party.len()) as f32)])
        });
        for name in [
            "PartyGetMemberCount",
            "PartyGetPlayerCount",
            "PartyGetTeamMemberCount",
        ] {
            set_field(&engine, name, party_count.clone());
        }
        // The party waits for him to start (the solo lobby's Start Match).
        // "Host is ready to start" means the match is already launching: every
        // lobby greys Start Match and Map while it answers true.
        set_field(&engine, "PartyIsWaiting", truthy.clone());
        let f = vm.native("false", |_, _| Ok(vec![Value::Bool(false)]));
        set_field(&engine, "PartyHostIsReadyToStart", f);
        // The map he picked is valid and he owns it: without these the Custom
        // Games lobby reset his map to TranZit and greyed Start Match and Map.
        set_field(&engine, "IsMapValid", truthy.clone());
        set_field(&engine, "DoesPartyHaveDLCForMap", truthy.clone());
        let c = calls.clone();
        let f = vm.native("PartyHostToggleStart", move |_, _| {
            c.borrow_mut().push(EngineCall::Exec("xpartygo".to_owned()));
            Ok(vec![])
        });
        set_field(&engine, "PartyHostToggleStart", f);
        // bo2zm: the lobby's status line (`Engine.GetGameLobbyStatus`:
        // text, seconds left). A Public Match lobby waits for the players
        // it has room for, counts down, then starts the match (nobody else
        // joins offline); elsewhere nothing (his ready / party leader line).
        let v = values.clone();
        let c = calls.clone();
        let f = vm.native("GetGameLobbyStatus", move |_, _| {
            let mut v = v.borrow_mut();
            let Some(since) = v.public_lobby_since else {
                return Ok(vec![]);
            };
            let loc = |v: &EngineValues, k: &str| v.localize.get(k).cloned().unwrap_or_default();
            let gone = lui::now_ms() - since;
            if gone < PUBLIC_SEARCH_MS {
                let room = ZOMBIES_MAX_PLAYERS.saturating_sub(1 + v.party.len());
                let text = loc(&v, "MENU_WAITING_FOR_MORE_PLAYERS")
                    .replace("&&1", &room.to_string());
                return Ok(vec![Value::str(&text)]);
            }
            let left = ((PUBLIC_SEARCH_MS + PUBLIC_COUNTDOWN_MS - gone) / 1000.0).ceil();
            // In someone else's lobby its host starts the match.
            if left <= 0.0 && v.party_follower {
                let text = loc(&v, "ZMUI_WAITING_FOR_HOST_TO_START");
                return Ok(vec![Value::str(&text)]);
            }
            if left <= 0.0 {
                v.public_lobby_since = None;
                c.borrow_mut().push(EngineCall::Exec("xpartygo".to_owned()));
                return Ok(vec![]);
            }
            // "Game starting in &&1": the lobby adds ":  <seconds>".
            let text = loc(&v, "MENU_GAME_STARTING_IN").replace(" &&1", "").replace("&&1", "");
            Ok(vec![Value::str(&text), Value::Num(left as f32)])
        });
        set_field(&engine, "GetGameLobbyStatus", f);
        // His player stats: any path answers (0 until set); the class-reset
        // warnings were seen long ago (a new profile would show them once).
        {
            let mut v = values.borrow_mut();
            for path in [
                "cacLoadouts.resetWarningDisplayed",
                "cacLoadouts.classWarningDisplayed",
            ] {
                v.stats.insert(path.to_owned(), Value::Num(1.0));
            }
        }
        // The maps the globe offers (`Engine.GetMaps`): the ones this game
        // plays, by load name (`maps`, e.g. zm_nuked); the globe reads the
        // rest (name, place on the globe) from the maps table.
        let v = values.clone();
        let f = vm.native("GetMaps", move |_, _| {
            let t = Table::new_ref();
            for (i, name) in v.borrow().maps.iter().enumerate() {
                let m = Table::new_ref();
                m.borrow_mut().set_str("loadName", Value::str(name));
                m.borrow_mut().set_str("filter", Value::str(""));
                t.borrow_mut()
                    .set(Value::Num((i + 1) as f32), Value::Table(m));
            }
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetMaps", f);
        let v = values.clone();
        let f = vm.native("GetPlayersInLobby", move |_, _| {
            Ok(vec![lobby_members(&v.borrow())])
        });
        set_field(&engine, "GetPlayersInLobby", f);
        // No friends online (offline, one PC): the joinable lists are empty.
        let empty = vm.native("empty_list", |_, _| {
            Ok(vec![Value::Table(Table::new_ref())])
        });
        for name in [
            "GetTitleFriendsOfAllLocalPlayers",
            "GetFriendsOfAllLocalPlayers",
            "GetRecentPlayers",
            // No league team: the lobby's player card skips the team row.
            "GetLeagueTeamInfo",
        ] {
            set_field(&engine, name, empty.clone());
        }
        // The player card (a row picked in the lobby's player list): him,
        // with no status line.
        let v = values.clone();
        let f = vm.native("GetPlayerInfoByXuid", move |_, a| {
            // bo2zm H28: a friend's (the Friends list's player card).
            let x = arg(&a, 1).to_string();
            if let Some(f) = v.borrow().friends.iter().find(|f| f.xuid == x) {
                return Ok(vec![Value::Table(friend_row(f))]);
            }
            // The picked lobby row's (the host's popup on a joiner's row
            // titles it with that player's name); his own otherwise.
            let p = lobby_row(&v.borrow(), &x).or_else(|| lobby_row(&v.borrow(), "1"));
            let p = p.unwrap_or_else(Table::new_ref);
            p.borrow_mut().set_str("status", Value::str(""));
            // A string, as the card reads it (it looks up rank - 1 in the
            // rank icon table; "0" would become "1").
            p.borrow_mut().set_str("rank", Value::str("1"));
            Ok(vec![Value::Table(p)])
        });
        set_field(&engine, "GetPlayerInfoByXuid", f);
        // His own card: nobody can join him from the card (no "Joinable"
        // line); a friend's lobby with room can be (bo2zm H28).
        let v = values.clone();
        let f = vm.native("IsPlayerJoinable", move |_, a| {
            let open = a.iter().any(|x| {
                let x = x.to_string();
                v.borrow().friends.iter().any(|f| f.xuid == x && f.joinable)
            });
            let t = Table::new_ref();
            t.borrow_mut().set_str("isJoinable", Value::Bool(open));
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "IsPlayerJoinable", f);
        // Theater's Select Film (codtv.lua, root "recents"): folder 1 holds
        // the three shelves of real 15b, Recent Games, Films and Edited
        // Films (folders 2-4, Demonware ones). Each search finds nothing:
        // the game's own "Empty" tile and "There are no items available in
        // this category." No films are recorded here.
        let f = vm.native("IsCodtvContentLoaded", |_, _| Ok(vec![Value::Bool(true)]));
        set_field(&engine, "IsCodtvContentLoaded", f);
        let f = vm.native("GetCodtvRoot", |_, a| {
            Ok(vec![if arg(&a, 0).to_string() == "recents" {
                Value::Num(1.0)
            } else {
                Value::Nil
            }])
        });
        set_field(&engine, "GetCodtvRoot", f);
        const SHELVES: [&str; 3] = [
            "MENU_FILESHARE_RECENTGAMES_CAPS",
            "MENU_FILMS_CAPS",
            "MENU_EDITED_FILMS",
        ];
        let v = values.clone();
        let f = vm.native("GetCodtvContent", move |_, a| {
            let loc = |k: &str| v.borrow().localize.get(k).cloned().unwrap_or_else(|| k.to_owned());
            let i = match a.first() {
                Some(Value::Num(n)) => *n as usize,
                _ => return Ok(vec![Value::Nil]),
            };
            let t = Table::new_ref();
            if i == 1 {
                let mut t = t.borrow_mut();
                t.set_str("name", Value::str(&loc("MENU_FILESHARE_SELECT_FILM_CAPS")));
                t.set_str("type", Value::str("folder"));
                t.set_str("subfolderCount", Value::Num(SHELVES.len() as f32));
                for (n, key) in SHELVES.iter().enumerate() {
                    let c = Table::new_ref();
                    c.borrow_mut().set_str("name", Value::str(&loc(key)));
                    c.borrow_mut().set_str("folderIndex", Value::Num((n + 2) as f32));
                    t.set(Value::Num((n + 1) as f32), Value::Table(c));
                }
            } else if let Some(key) = SHELVES.get(i.wrapping_sub(2)) {
                let mut t = t.borrow_mut();
                t.set_str("name", Value::str(&loc(key)));
                t.set_str("type", Value::str("dwfolder"));
                t.set_str("subfolderCount", Value::Num(0.0));
            } else {
                return Ok(vec![Value::Nil]);
            }
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetCodtvContent", f);
        let c = calls.clone();
        let f = vm.native("LoadCodtvDWContent", move |_, a| {
            // (controller, folderIndex, startIndex, userData)
            let folder = match a.get(1) {
                Some(Value::Num(n)) => *n,
                _ => 0.0,
            };
            // bo2mp: the multiplayer front end raises it from its own
            // command (the UI answers `bo2mpDwSearch <folder>`).
            if lui::mp_front_end() {
                c.borrow_mut().push(EngineCall::Exec(format!("bo2mpDwSearch {}", folder as i64)));
                return Ok(vec![Value::Bool(true)]);
            }
            c.borrow_mut().push(EngineCall::Event(
                "fileshare_search_complete".to_owned(),
                vec![
                    ("controller".to_owned(), 0.0),
                    ("contextid".to_owned(), folder),
                    ("numresults".to_owned(), 0.0),
                    ("startIndex".to_owned(), 0.0),
                ],
            ));
            Ok(vec![Value::Bool(true)])
        });
        set_field(&engine, "LoadCodtvDWContent", f);
        // A map's start locations and a start location's game modes, from
        // zm/gametypestable.csv: rows `5,index,map,ref,name,desc,picture`
        // and `6,index,map,location,mode,dlc` (Nuketown: `nuked`,
        // `zstandard`); game mode names from rows `0,mode,name`.
        let v = values.clone();
        let f = vm.native("GetStartLocsZombie", move |_, a| {
            let map = arg(&a, 0).to_string();
            let v = v.borrow();
            let t = Table::new_ref();
            let rows = v
                .tables
                .get("zm/gametypestable.csv")
                .cloned()
                .unwrap_or_default();
            let cell = |r: &Vec<String>, i: usize| r.get(i).cloned().unwrap_or_default();
            let mut n = 0.0;
            for r in rows
                .iter()
                .filter(|r| cell(r, 0) == "5" && cell(r, 2) == map && offered(&map, &cell(r, 3), None))
            {
                let e = Table::new_ref();
                let name = cell(r, 4);
                let shown = v
                    .localize
                    .get(&name.to_ascii_uppercase())
                    .cloned()
                    .unwrap_or(name);
                e.borrow_mut().set_str("ref", Value::str(&cell(r, 3)));
                e.borrow_mut().set_str("name", Value::str(&shown));
                e.borrow_mut()
                    .set_str("index", Value::Num(cell(r, 1).parse().unwrap_or(0.0)));
                e.borrow_mut().set_str("playerCount", Value::Num(LOBBY_COUNT));
                n += 1.0;
                t.borrow_mut().set(Value::Num(n), Value::Table(e));
            }
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetStartLocsZombie", f);
        let v = values.clone();
        let f = vm.native("GetGamemodesZombie", move |_, a| {
            let (map, loc) = (arg(&a, 0).to_string(), arg(&a, 1).to_string());
            let v = v.borrow();
            let t = Table::new_ref();
            let rows = v
                .tables
                .get("zm/gametypestable.csv")
                .cloned()
                .unwrap_or_default();
            let cell = |r: &Vec<String>, i: usize| r.get(i).cloned().unwrap_or_default();
            let mut n = 0.0;
            for r in rows
                .iter()
                .filter(|r| {
                    cell(r, 0) == "6"
                        && cell(r, 2) == map
                        && cell(r, 3) == loc
                        && offered(&map, &loc, Some(&cell(r, 4)))
                })
            {
                let mode = cell(r, 4);
                let key = rows
                    .iter()
                    .find(|g| cell(g, 0) == "0" && cell(g, 1) == mode)
                    .map(|g| cell(g, 2))
                    .unwrap_or_default();
                let shown = v
                    .localize
                    .get(&key.to_ascii_uppercase())
                    .cloned()
                    .unwrap_or(key);
                let e = Table::new_ref();
                e.borrow_mut().set_str("ref", Value::str(&mode));
                e.borrow_mut().set_str("name", Value::str(&shown));
                e.borrow_mut()
                    .set_str("index", Value::Num(cell(r, 1).parse().unwrap_or(0.0)));
                e.borrow_mut().set_str("playerCount", Value::Num(LOBBY_COUNT));
                n += 1.0;
                t.borrow_mut().set(Value::Num(n), Value::Table(e));
            }
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetGamemodesZombie", f);
        // His copy holds every map pack's content the menus ask about.
        set_field(&engine, "HasDLCContent", truthy.clone());
        // The playlists (BO2 downloads them; built here from the maps this
        // game plays and the gametypes table): each map once per filter
        // (`playermatch`, `solomatch`: a super category, its load name
        // `_solo` for Solo), its start locations (categories, `nuked_solo`)
        // and their game modes (playlists, by id).
        let v = values.clone();
        let f = vm.native("GetPlaylistSuperCategories", move |_, _| {
            let rows = playlist_rows(&v.borrow());
            let t = Table::new_ref();
            let mut seen = Vec::new();
            for r in &rows {
                if seen.contains(&r.super_index) {
                    continue;
                }
                seen.push(r.super_index);
                let e = Table::new_ref();
                let solo = if r.filter == "solomatch" { "_solo" } else { "" };
                e.borrow_mut()
                    .set_str("loadName", Value::str(&format!("{}{solo}", r.map)));
                e.borrow_mut().set_str("name", Value::str(&r.map_name));
                e.borrow_mut().set_str("filter", Value::str(r.filter));
                e.borrow_mut()
                    .set_str("index", Value::Num(r.super_index as f32));
                e.borrow_mut().set_str("playerCount", Value::Num(LOBBY_COUNT));
                t.borrow_mut()
                    .set(Value::Num(seen.len() as f32), Value::Table(e));
            }
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetPlaylistSuperCategories", f);
        let v = values.clone();
        let f = vm.native("GetPlaylistSuperCategoryID", move |_, a| {
            let id = a.iter().rev().find_map(Value::as_num).unwrap_or(0.0) as usize;
            let rows = playlist_rows(&v.borrow());
            Ok(vec![
                rows.iter()
                    .find(|r| r.id == id)
                    .map_or(Value::Nil, |r| Value::Num(r.super_index as f32)),
            ])
        });
        set_field(&engine, "GetPlaylistSuperCategoryID", f);
        let v = values.clone();
        let f = vm.native("GetPlaylistCategories", move |_, a| {
            let sup = a.iter().rev().find_map(Value::as_num).unwrap_or(0.0) as usize;
            let vb = v.borrow();
            let rows = playlist_rows(&vb);
            let t = Table::new_ref();
            let mut cats: Vec<(String, TableRef)> = Vec::new();
            for r in rows.iter().filter(|r| r.super_index == sup) {
                let solo = if r.filter == "solomatch" { "_solo" } else { "" };
                let cref = format!("{}{solo}", r.loc);
                let cat = match cats.iter().find(|(c, _)| *c == cref) {
                    Some((_, c)) => c.clone(),
                    None => {
                        let c = Table::new_ref();
                        c.borrow_mut().set_str("ref", Value::str(&cref));
                        c.borrow_mut().set_str("name", Value::str(&r.loc_name));
                        c.borrow_mut().set_str("description", Value::str(&r.loc_desc));
                        c.borrow_mut().set_str("filter", Value::str(r.filter));
                        c.borrow_mut()
                            .set_str("index", Value::Num((cats.len() + 1) as f32));
                        c.borrow_mut().set_str("playerCount", Value::Num(LOBBY_COUNT));
                        c.borrow_mut()
                            .set_str("playlists", Value::Table(Table::new_ref()));
                        cats.push((cref.clone(), c.clone()));
                        t.borrow_mut()
                            .set(Value::Num(cats.len() as f32), Value::Table(c.clone()));
                        c
                    }
                };
                let list = cat.borrow().get_str("playlists");
                if let Value::Table(list) = list {
                    let p = Table::new_ref();
                    p.borrow_mut().set_str("index", Value::Num(r.id as f32));
                    p.borrow_mut().set_str("ref", Value::str(&r.mode));
                    p.borrow_mut().set_str("name", Value::str(&r.mode_name));
                    p.borrow_mut().set_str("description", Value::str(&r.loc_desc));
                    p.borrow_mut().set_str("playerCount", Value::Num(LOBBY_COUNT));
                    let n = list.borrow().len() + 1;
                    list.borrow_mut().set(Value::Num(n as f32), Value::Table(p));
                }
            }
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetPlaylistCategories", f);
        // The playlist picked (`Engine.SetPlaylistID`) and its facts.
        let v = values.clone();
        let f = vm.native("GetPlaylistID", move |_, _| {
            Ok(vec![Value::Num(v.borrow().playlist as f32)])
        });
        set_field(&engine, "GetPlaylistID", f.clone());
        set_field(&ui, "GetPlaylistID", f);
        let v = values.clone();
        let f = vm.native("SetPlaylistID", move |_, a| {
            let id = a.iter().rev().find_map(|x| {
                x.as_num()
                    .or_else(|| x.as_str().and_then(|s| s.parse().ok()))
            });
            if let Some(id) = id {
                let mut vb = v.borrow_mut();
                vb.playlist = id as usize;
                // BO2's engine applies the playlist: its map, start location
                // and game type (Tranzit > Town > Survival = zstandard town).
                let row = playlist_rows(&vb).into_iter().find(|r| r.id == id as usize);
                if let Some(r) = row {
                    for (k, x) in [
                        ("ui_mapname", r.map),
                        ("ui_zm_mapstartlocation", r.loc),
                        ("ui_gametype", r.mode.clone()),
                    ] {
                        dvar_set(&mut vb.dvars, k, x);
                    }
                    // ...and the mode's group, which the lobby picture is
                    // named by (`menu_zm_prison_zencounter_cellblock`).
                    if let Some(g) = mode_group(&vb.tables, &r.mode) {
                        dvar_set(&mut vb.dvars, "ui_zm_gamemodegroup", g);
                    }
                }
            }
            Ok(vec![])
        });
        set_field(&engine, "SetPlaylistID", f);
        for (name, field) in [
            ("GetPlaylistGameType", 0usize),
            ("GetPlaylistCategoryFilter", 1),
            ("GetPlaylistCategoryName", 2),
            ("GetPlaylistName", 3),
        ] {
            let v = values.clone();
            let f = vm.native(name, move |_, a| {
                let vb = v.borrow();
                let id = a
                    .iter()
                    .rev()
                    .find_map(Value::as_num)
                    .map_or(vb.playlist, |n| n as usize);
                let rows = playlist_rows(&vb);
                let Some(r) = rows.iter().find(|r| r.id == id) else {
                    return Ok(vec![Value::str("")]);
                };
                let out = match field {
                    0 => r.mode.clone(),
                    1 => r.filter.to_owned(),
                    2 => r.loc_name.clone(),
                    _ => r.mode_name.clone(),
                };
                Ok(vec![Value::str(&out)])
            });
            set_field(&engine, name, f.clone());
            set_field(&ui, name, f);
        }
        let stats = stat_node(&mut vm, values.clone(), String::new());
        let f = vm.native("GetPlayerStats", move |_, _| Ok(vec![stats.clone()]));
        set_field(&engine, "GetPlayerStats", f);
        let v = values.clone();
        let f = vm.native("LastInput_Gamepad", move |_, _| {
            // bo2mp: whether he plays with a controller now (the menus'
            // button glyphs and prompts follow it); bo2zm: his last input.
            Ok(vec![Value::Bool(gamepad_active() || v.borrow().gamepad)])
        });
        set_field(&engine, "LastInput_Gamepad", f);
        let nothing = vm.native("nothing", |_, _| Ok(vec![]));
        // Engine.SetDvar(name, value) (the front end's ui_mapname, ...).
        let v = values.clone();
        let f = vm.native("SetDvar", move |_, a| {
            dvar_set(
                &mut v.borrow_mut().dvars,
                &arg(&a, 0).to_string(),
                arg(&a, 1).to_string(),
            );
            Ok(vec![])
        });
        set_field(&engine, "SetDvar", f);
        for name in ["SetForceMouseRootFull", "StopEditingPresetClass"] {
            set_field(&engine, name, nothing.clone());
        }
        let c = calls.clone();
        let f = vm.native("PlayMenuMusic", move |_, a| {
            c.borrow_mut()
                .push(EngineCall::PlayMusic(arg(&a, 0).to_string()));
            Ok(vec![])
        });
        set_field(&engine, "PlayMenuMusic", f);
        let c = calls.clone();
        let f = vm.native("SendMenuResponse", move |_, a| {
            c.borrow_mut().push(EngineCall::MenuResponse(
                arg(&a, 1).to_string(),
                arg(&a, 2).to_string(),
            ));
            Ok(vec![])
        });
        set_field(&engine, "SendMenuResponse", f);
        let c = calls.clone();
        let f = vm.native("Exec", move |_, a| {
            // Exec(controller, command) or Exec(command).
            let cmd = a
                .iter()
                .rev()
                .find_map(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            // Theater's film browser asks for its save slots: there are
            // none to fill, but the answer comes (BO2 asks Demonware).
            if cmd == "fileshareGetSlots" && !lui::mp_front_end() {
                c.borrow_mut().push(EngineCall::Event(
                    "fileshare_slots_available".to_owned(),
                    vec![("controller".to_owned(), 0.0), ("valid".to_owned(), 1.0)],
                ));
            }
            c.borrow_mut().push(EngineCall::Exec(cmd));
            Ok(vec![])
        });
        set_field(&engine, "Exec", f.clone());
        // ExecNow: the same, done at once (the solo lobby's Start Match:
        // `xpartyready 1`).
        set_field(&engine, "ExecNow", f);
        let c = calls.clone();
        let f = vm.native("PlaySound", move |_, a| {
            c.borrow_mut()
                .push(EngineCall::PlaySound(arg(&a, 0).to_string()));
            Ok(vec![])
        });
        set_field(&engine, "PlaySound", f);
        // A solo game pauses (one local player, no one else in the game).
        let f = vm.native("CanPauseZombiesGame", |_, _| Ok(vec![Value::Bool(true)]));
        set_field(&engine, "CanPauseZombiesGame", f);
        let c = calls.clone();
        let f = vm.native("BlurWorld", move |_, a| {
            let amount = match a.get(1) {
                Some(Value::Num(n)) => *n,
                _ => 0.0,
            };
            c.borrow_mut().push(EngineCall::BlurWorld(amount));
            Ok(vec![])
        });
        set_field(&engine, "BlurWorld", f);
        let c = calls.clone();
        let f = vm.native("FetchLeagueTeams", move |_, _| {
            c.borrow_mut().push(EngineCall::FetchLeagueTeams);
            Ok(vec![])
        });
        set_field(&engine, "FetchLeagueTeams", f);
        // SystemNeedsUpdate(controller, system): a menu changed what a
        // system shows (Change Map: "lobby", "game_options"); the engine
        // then tells the menus `<system>` updates (the UI raises them).
        let c = calls.clone();
        let f = vm.native("SystemNeedsUpdate", move |_, a| {
            let system = arg(&a, 1).as_str().map(str::to_owned).unwrap_or_default();
            c.borrow_mut().push(EngineCall::Exec(format!("bo2mpSystemNeedsUpdate {system}")));
            Ok(vec![])
        });
        set_field(&engine, "SystemNeedsUpdate", f);
        for name in [
            "IsMigrating",
            "LockInput",
            "SetUIActive",
            "ProbationCheckIfPenalizedForQuit",
        ] {
            set_field(
                &engine,
                name,
                if name == "IsMigrating" || name == "ProbationCheckIfPenalizedForQuit" {
                    falsy.clone()
                } else {
                    nothing.clone()
                },
            );
        }
        let one = vm.native("one", |_, _| Ok(vec![Value::Num(1.0)]));
        // In a game (the HUD) or not (the front end).
        let v = values.clone();
        let f = vm.native("IsInGame", move |_, _| {
            Ok(vec![Value::Num(if v.borrow().front_end {
                0.0
            } else {
                1.0
            })])
        });
        set_field(&ui, "IsInGame", f);
        // His keyboard and mouse are controller 0 (menus pass on input only
        // from a controller in use).
        set_field(&ui, "IsControllerBeingUsed", one.clone());
        // One local player.
        set_field(&ui, "SplitscreenNum", one.clone());
        set_field(&engine, "GetClientNum", zero.clone());
        set_field(&ui, "IsGuest", zero.clone());
        // The scripts' debug output (a release build prints nothing).
        vm.set_global("DebugPrint", nothing.clone());
        for name in ["IsDemoPlaying", "SessionMode_IsOnlineGame"] {
            set_field(&ui, name, zero.clone());
        }
        // The scoreboard: one team (CoD.TEAM_ALLIES, set by `load_base`'s
        // caller through `players`), its players.
        let v = values.clone();
        let f = vm.native("GetTeamPositions", move |_, _| {
            let t = Table::new_ref();
            let row = Table::new_ref();
            let v = v.borrow();
            // The team's score: its players' first column (score).
            let score: f32 = v
                .players
                .iter()
                .filter_map(|r| r.get(1)?.parse::<f32>().ok())
                .sum();
            row.borrow_mut().set_str("team", Value::Num(v.team as f32));
            row.borrow_mut().set_str("score", Value::Num(score));
            t.borrow_mut().set(Value::Num(1.0), Value::Table(row));
            Ok(vec![Value::Table(t)])
        });
        set_field(&engine, "GetTeamPositions", f);
        let v = values.clone();
        let f = vm.native("GetMatchScoreboardClientCount", move |_, _| {
            // bo2mp: at least his own row in multiplayer; bo2zm: the rows.
            let n = v.borrow().players.len();
            let n = if lui::mp_front_end() { n.max(1) } else { n };
            Ok(vec![Value::Num(n as f32)])
        });
        set_field(&engine, "GetMatchScoreboardClientCount", f);
        // GetMatchScoreboardIndexAndClientNumForTeam(i, team, sort): row i
        // and its client number.
        let v = values.clone();
        let f = vm.native("GetMatchScoreboardIndexAndClientNumForTeam", move |_, a| {
            let i = arg(&a, 0).as_num().map_or(0, |n| n as usize);
            let c = v.borrow().clients.get(i).copied().unwrap_or(i as i32);
            Ok(vec![Value::Num(i as f32), Value::Num(c as f32)])
        });
        set_field(&engine, "GetMatchScoreboardIndexAndClientNumForTeam", f);
        let v = values.clone();
        let f = vm.native("GetFullGamertagForScoreboardIndex", move |_, a| {
            let i = arg(&a, 0).as_num().map_or(0, |n| n as usize);
            let name = v
                .borrow()
                .players
                .get(i)
                .and_then(|r| r.first().cloned())
                .unwrap_or_default();
            Ok(vec![Value::str(&name)])
        });
        set_field(&engine, "GetFullGamertagForScoreboardIndex", f);
        let v = values.clone();
        let f = vm.native("GetScoreboardColumnForScoreboardIndex", move |_, a| {
            let i = arg(&a, 0).as_num().map_or(0, |n| n as usize);
            let col = arg(&a, 1).as_num().map_or(0, |n| n as usize);
            let cell = v
                .borrow()
                .players
                .get(i)
                .and_then(|r| r.get(col + 1).cloned())
                .unwrap_or_default();
            Ok(vec![Value::str(&cell)])
        });
        set_field(&engine, "GetScoreboardColumnForScoreboardIndex", f);
        // A local game: no ping (full bars), nobody muted.
        set_field(&engine, "GetPingForScoreboardIndex", zero.clone());
        set_field(&engine, "IsPlayerMuteToggled", falsy.clone());
        // Nuketown's teams (teamset_cdc): the allies CDC, the axis CIA; the
        // other team numbers (the lobby lists every team) have none.
        let f = vm.native("GetFactionForTeam", |_, a| {
            Ok(match arg(&a, 0).as_num().map(|n| n as i32) {
                Some(1) => vec![Value::str("cdc")],
                Some(2) => vec![Value::str("cia")],
                _ => vec![Value::str("")],
            })
        });
        set_field(&engine, "GetFactionForTeam", f);
        // A faction's colour (r, g, b) from zm/factiontable.csv (name, team,
        // red, green, blue 0-255; the CDC gold 224 178 46).
        let v = values.clone();
        let f = vm.native("GetFactionColor", move |_, a| {
            let want = arg(&a, 0).to_string();
            let v = v.borrow();
            let rgb = v
                .tables
                .get("zm/factiontable.csv")
                .and_then(|rows| rows.iter().find(|r| r.first().is_some_and(|c| *c == want)))
                .map(|r| {
                    [2, 3, 4].map(|i| {
                        r.get(i).and_then(|c| c.parse::<f32>().ok()).unwrap_or(0.0) / 255.0
                    })
                })
                // No faction (the front-end lobby's free team): his
                // lobby row's bar stays dark, as in the real lobby.
                .unwrap_or([0.0, 0.0, 0.0]);
            Ok(rgb.into_iter().map(Value::Num).collect())
        });
        set_field(&engine, "GetFactionColor", f);
        let v = values.clone();
        let f = vm.native("GetScoreBoardColumnName", move |_, a| {
            let i = arg(&a, 1).as_num().map_or(0, |n| n as usize);
            Ok(vec![Value::str(
                v.borrow().columns.get(i).map_or("", String::as_str),
            )])
        });
        set_field(&engine, "GetScoreBoardColumnName", f);
        // Weapons (the HUD passes the weapon the engine named in
        // `hud_update_weapon`: here its name). A weapon the HUD was told of
        // answers by its own `weapType` (Turned's zombie claws are melee: no
        // ammo counter); any other by its name.
        let v = values.clone();
        let f = vm.native("IsWeaponType", move |_, a| {
            let w = arg(&a, 0).to_string();
            let kind = arg(&a, 1).to_string();
            let own = v.borrow().weapon_types.get(&w).copied();
            if let Some(t) = own {
                let want = match kind.as_str() {
                    "grenade" => 1,
                    "gas" => 4,
                    "melee" => 7,
                    _ => -1,
                };
                return Ok(vec![Value::Bool(t == want)]);
            }
            let hit = match kind.as_str() {
                "melee" => ["knife", "bowie", "tazer", "fists", "sickle"]
                    .iter()
                    .any(|k| w.contains(k)),
                "grenade" => ["grenade", "monkey", "claymore", "emp"]
                    .iter()
                    .any(|k| w.contains(k)),
                _ => false,
            };
            Ok(vec![Value::Bool(hit)])
        });
        set_field(&engine, "IsWeaponType", f);
        set_field(&engine, "IsOverheatWeapon", falsy.clone());
        set_field(&engine, "IsShoutcaster", falsy.clone());
        set_field(&engine, "ForceHUDRefresh", nothing.clone());
        set_field(&engine, "GetActiveLocalClientsCount", one.clone());
        // The key a command is bound to (the string argument naming it).
        // (controller, command[, index]): the index-th key (0 first).
        for (table, name) in [
            (&engine, "GetKeyBindingLocalizedString"),
            (&ui, "KeyBinding"),
        ] {
            let v = values.clone();
            let f = vm.native(name, move |_, a| {
                let at = a.iter().position(|x| x.as_str().is_some());
                let cmd = at.and_then(|i| a[i].as_str()).unwrap_or("").to_owned();
                let index = at.and_then(|i| a.get(i + 1)).and_then(Value::as_num).map_or(0, |n| n as usize);
                let keys: Vec<String> = v.borrow().keys_of(&cmd).map(|k| k.iter().skip(index).cloned().collect()).unwrap_or_default();
                let key = keys.first().cloned().unwrap_or_default();
                // The Controls rows ask with every flag (nine arguments) and
                // show what comes back: BO2 says UNBOUND (its KEY_UNBOUND
                // text) for a command with no key (the footer prompts ask
                // with three and want "").
                if key.is_empty() && a.len() >= 9 {
                    let unbound = v.borrow().localize.get("KEY_UNBOUND").cloned();
                    return Ok(vec![Value::str(unbound.as_deref().unwrap_or("UNBOUND"))]);
                }
                if a.len() < 9 {
                    return Ok(vec![Value::str(&key.to_uppercase())]);
                }
                // A row shows every key the command is on, joined by BO2's
                // own KEY_OR; each is its KEY_* name, a mouse button its
                // glyph (^B<FontIcon entry>^).
                let or = v.borrow().localize.get("KEY_OR").cloned().unwrap_or_else(|| "OR".to_owned());
                let shown: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        if let Some((_, glyph)) = MOUSE_GLYPHS.iter().find(|(m, _)| k.eq_ignore_ascii_case(m)) {
                            return format!("^B{glyph}^");
                        }
                        let up = k.to_uppercase();
                        v.borrow().localize.get(&format!("KEY_{up}")).cloned().unwrap_or(up)
                    })
                    .collect();
                // BO2's key table has one SHIFT, one CTRL and one ALT
                // (KEY_SHIFT/KEY_CTRL/KEY_ALT, no left/right names), so a
                // command on both physical keys lists that name once.
                let mut once: Vec<String> = Vec::new();
                for s in shown {
                    if !once.contains(&s) {
                        once.push(s);
                    }
                }
                Ok(vec![Value::str(&once.join(&format!(" {or} ")))])
            });
            set_field(table, name, f);
        }
        // Profile and hardware-profile settings: kept as text by name.
        for (name, arg_at) in [("SetProfileVar", 1usize), ("SetHardwareProfileValue", 0)] {
            let v = values.clone();
            let f = vm.native(name, move |_, a| {
                let key = arg(&a, arg_at).to_string();
                v.borrow_mut()
                    .profile
                    .insert(key, arg(&a, arg_at + 1).to_string());
                Ok(vec![])
            });
            set_field(&engine, name, f);
        }
        let v = values.clone();
        let f = vm.native("GetHardwareProfileValueAsString", move |_, a| {
            let key = arg(&a, 0).to_string();
            let vb = v.borrow();
            Ok(vec![Value::str(
                vb.profile
                    .get(&key)
                    .or_else(|| vb.dvars.get(&key))
                    .map_or("0", String::as_str),
            )])
        });
        set_field(&engine, "GetHardwareProfileValueAsString", f);
        set_field(&engine, "SyncHardwareProfileWithDvars", nothing.clone());
        for (name, number) in [
            ("ProfileValueAsString", false),
            ("ProfileInt", true),
            ("ProfileFloat", true),
            ("ProfileBool", true),
        ] {
            let v = values.clone();
            let f = vm.native(name, move |_, a| {
                let key = arg(&a, 1).to_string();
                // bo2zm: an unset profile value reads 0, as BO2's profile
                // defaults it (the map picker's `unlock_crumbs_zm` bits);
                // bo2mp: empty in the multiplayer front end.
                let s = v
                    .borrow()
                    .profile
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| if lui::mp_front_end() { String::new() } else { "0".to_owned() });
                Ok(vec![if number {
                    Value::Num(s.trim().parse().unwrap_or(0.0))
                } else {
                    Value::str(&s)
                }])
            });
            set_field(&ui, name, f);
        }
        // BindCommand(controller, command, index): the next key he presses.
        let c = calls.clone();
        let f = vm.native("BindCommand", move |_, a| {
            let cmd = arg(&a, 1).to_string();
            c.borrow_mut().push(EngineCall::BindCommand(
                cmd,
                arg(&a, 2).as_num().map_or(0, |n| n as usize),
            ));
            Ok(vec![])
        });
        set_field(&engine, "BindCommand", f);
        // ExecNow(controller, command): bo2mp: config files and dvars here
        // (`run_command`; `mp` binds its own); bo2zm: the same as Exec, done
        // at once (the solo lobby's Start Match: `xpartyready 1`).
        let v = values.clone();
        let c = calls.clone();
        let f = vm.native("ExecNow", move |_, a| {
            let cmd = a.iter().rev().find_map(|v| v.as_str().map(str::to_owned)).unwrap_or_default();
            if lui::mp_front_end() {
                run_command(&v, &cmd);
            } else {
                c.borrow_mut().push(EngineCall::Exec(cmd));
            }
            Ok(vec![])
        });
        set_field(&engine, "ExecNow", f);

        // GetTextDimensions(text, font, height) -> left, top, right, bottom.
        let f = vm.native("GetTextDimensions", move |_, a| {
            let text = arg(&a, 0).to_string();
            let font = match arg(&a, 1) {
                Value::User(u) => font_name(&u).unwrap_or_default(),
                other => other.to_string(),
            };
            let h = arg(&a, 2).as_num().unwrap_or(0.0);
            Ok(vec![
                Value::Num(0.0),
                Value::Num(0.0),
                Value::Num(measure(&text, &font, h)),
                Value::Num(h),
            ])
        });
        vm.set_global("GetTextDimensions", f);

        // require(module): the script named like it under ui/ or ui_mp/.
        let mut by_key: HashMap<String, Rc<[u8]>> = HashMap::new();
        for (name, bytes) in scripts {
            if name.to_ascii_lowercase().ends_with(".lua") {
                by_key.insert(script_key(&name), Rc::from(bytes));
            }
        }
        let loaded = Table::new_ref();
        let miss = missing.clone();
        let require = vm.native("require", move |vm, a| {
            let module = arg(&a, 0).to_string();
            let hit = loaded.borrow().get_str(&module);
            if hit.truthy() {
                return Ok(vec![hit]);
            }
            let stem = script_key(&module);
            let Some(bytes) = ["ui", "uimp"]
                .iter()
                .find_map(|p| by_key.get(&format!("{p}{stem}")))
                .cloned()
            else {
                // The engine's require skips a script the zones lack (the
                // zombies HUD names multiplayer-only ones).
                miss.borrow_mut().push(module);
                return Ok(vec![]);
            };
            loaded.borrow_mut().set_str(&module, Value::Bool(true));
            let mut chunk =
                crate::parse(&bytes).map_err(|e| LuaError::new(format!("{module}: {e}")))?;
            // bo2zm: the Zombies front end leaves out [`DROPPED_BUTTONS`]
            // (bo2mp: the multiplayer menus keep theirs).
            if !lui::mp_front_end()
                && ["mainmenu", "mainlobby", "publicgamelobby", "theaterlobby"]
                    .iter()
                    .any(|m| stem.contains(m))
            {
                drop_buttons(&mut chunk.main);
            }
            let f = vm.load(chunk.main);
            let r = vm.call(f, vec![])?;
            allow_bots_in_every_mode(vm);
            hide_for_bo1_hud(vm);
            hide_for_zm_classic(vm);
            extend_map_categories(vm);
            let v = r.into_iter().next().filter(Value::truthy).unwrap_or(Value::Bool(true));
            loaded.borrow_mut().set_str(&module, v.clone());
            Ok(vec![v])
        });
        vm.set_global("require", require);

        // The fonts' resolution (`fonts/720/...`), read while CoD's base
        // loads.
        values
            .borrow_mut()
            .dvars
            .insert("r_fontResolution".to_owned(), "720".to_owned());
        values.borrow_mut().safe_area = (1280.0, 720.0);
        Host {
            vm,
            values,
            asked,
            missing,
            calls,
            roots: Vec::new(),
            root_size: (1280.0, 720.0),
            errors: Vec::new(),
            step_budget: 20_000_000,
            measure: measure_out,
        }
    }

    /// Call into the scripts with a fresh step budget; an error is kept in
    /// `errors` (the scripts go on, as the engine's do).
    pub fn call(&mut self, f: Value, args: Vec<Value>) -> Option<Vec<Value>> {
        self.vm.steps = 0;
        self.vm.step_limit = self.step_budget;
        match self.vm.call(f, args) {
            Ok(r) => Some(r),
            Err(e) => {
                if self.errors.len() < 200 {
                    self.errors.push(e.to_string());
                }
                None
            }
        }
    }

    pub fn require(&mut self, module: &str) -> Option<Value> {
        let r = self.vm.global("require");
        self.call(r, vec![Value::str(module)])
            .map(|v| v.into_iter().next().unwrap_or(Value::Nil))
    }

    /// LUI's core and CoD's base, as the engine loads them first.
    pub fn load_base(&mut self) {
        let metas: Vec<(&str, Option<TableRef>)> = ["Engine", "UIExpression"]
            .into_iter()
            .map(|n| (n, self.vm.global(n).table().and_then(|t| t.borrow().meta.clone())))
            .collect();
        for base in ["LUI.LUI", "T6.CoDBase"] {
            self.require(base);
        }
        // The engine's element functions on LUI's element class too (the
        // scripts call `LUI.UIElement.removeElement(self, child)`).
        let class = self.field("LUI", "UIElement");
        if let Some(class) = class.table() {
            let natives = lui::natives();
            let natives = natives.borrow();
            let mut key = Value::Nil;
            while let Some((k, v)) = natives.next(&key) {
                if matches!(class.borrow().get(&k), Value::Nil) {
                    class.borrow_mut().set(k.clone(), v);
                }
                key = k;
            }
        }
        // CoD's base answers a missing engine field with CoD.NullFunction;
        // ours does the same and counts it.
        for (n, meta) in metas {
            if let Some(t) = self.vm.global(n).table() {
                t.borrow_mut().meta = meta;
            }
        }
    }

    /// Set a function on an engine table (`Engine`, `UIExpression`, ...).
    pub fn bind(
        &mut self,
        table: &str,
        name: &'static str,
        f: impl Fn(&mut Vm, Vec<Value>) -> Result<Vec<Value>, LuaError> + 'static,
    ) {
        let t = self.vm.global(table);
        let v = self.vm.native(name, f);
        set_field(&t, name, v);
    }

    /// A material, as the scripts' `RegisterMaterial(name)` makes it.
    pub fn material(&mut self, name: &str) -> Value {
        let f = self.vm.global("RegisterMaterial");
        self.call(f, vec![Value::str(name)])
            .and_then(|r| r.into_iter().next())
            .unwrap_or(Value::Nil)
    }

    /// A field of a global table (`CoD.BIT_HUD_VISIBLE`).
    /// Set a field of a global table (`CoD.PlaylistCategoryFilter`).
    pub fn set_global_field(&mut self, global: &str, name: &str, value: Value) {
        let g = self.vm.global(global);
        set_field(&g, name, value);
    }

    /// He plays alone: for Grief and Turned, BO2's lobby swaps Start Match
    /// for the Barracks button while it matchmakes, and zombies has no
    /// Barracks, so the swap stops halfway and Start Match is gone. Those
    /// modes load and play solo here (Mob of the Dead is offered only as
    /// Grief), so a swap with nothing to swap to keeps Start Match. Run
    /// every front-end frame; it takes once (the original is Lua's).
    pub fn solo_lobby_start(&mut self) {
        let el = self.field("LUI", "UIElement");
        let Ok(orig) = self.vm.index(&el, &Value::str("closeAndRefocus")) else {
            return;
        };
        if !matches!(orig, Value::Func(_)) {
            return;
        }
        let f = self.vm.native("closeAndRefocus", move |vm, a| {
            if a.get(1).map_or(true, |t| matches!(t, Value::Nil)) {
                return Ok(vec![]);
            }
            vm.call(orig.clone(), a)
        });
        set_field(&el, "closeAndRefocus", f);
        eprintln!("bo2zm lui: solo lobby keeps Start Match for every mode");
    }

    pub fn field(&mut self, global: &str, name: &str) -> Value {
        let g = self.vm.global(global);
        self.vm.index(&g, &Value::str(name)).unwrap_or(Value::Nil)
    }

    /// The player's root (`LUI.UIRoot.new("UIRoot0")`, with `UIRootFull`
    /// beside it), made on first use and sized for a screen of this aspect
    /// (width / height).
    pub fn root(&mut self, aspect: f32) -> Option<Value> {
        let w = 720.0 * aspect.max(0.5);
        if self.roots.is_empty() {
            let ui_root = self.field("LUI", "UIRoot");
            let new = self.vm.index(&ui_root, &Value::str("new")).ok()?;
            for name in ["UIRoot0", "UIRootFull"] {
                let root = self
                    .call(new.clone(), vec![Value::str(name)])?
                    .into_iter()
                    .next()?;
                lui::add_root(lui::element(&root)?);
                self.roots.push(root);
            }
            self.root_size = (0.0, 0.0);
        }
        if (self.root_size.0 - w).abs() > 0.5 {
            self.root_size = (w, 720.0);
            self.values.borrow_mut().safe_area = (w, 720.0);
            lui::set_root_rect([0.0, 0.0, w, 720.0]);
            for root in self.roots.clone() {
                self.event(
                    &root,
                    "resize",
                    &[
                        ("width", Value::Num(w)),
                        ("height", Value::Num(720.0)),
                        ("unitsToPixels", Value::Num(1.0)),
                    ],
                );
            }
        }
        self.roots.first().cloned()
    }

    /// Send `name` with `fields` to an element (`processEvent`).
    pub fn event(&mut self, target: &Value, name: &str, fields: &[(&str, Value)]) {
        let t = Table::new_ref();
        t.borrow_mut().set_str("name", Value::str(name));
        t.borrow_mut().set_str("controller", Value::Num(0.0));
        for (k, v) in fields {
            t.borrow_mut().set_str(k, v.clone());
        }
        let pe = self
            .vm
            .index(target, &Value::str("processEvent"))
            .unwrap_or(Value::Nil);
        self.call(pe, vec![target.clone(), Value::Table(t)]);
    }

    /// Send an event to the root (and so to every menu on it).
    pub fn root_event(&mut self, name: &str, fields: &[(&str, Value)]) {
        if let Some(root) = self.roots.first().cloned() {
            self.event(&root, name, fields);
        }
    }

    /// An event whose fields are a table already made (a name card the
    /// engine sends as the event itself: `player_obituary_callout`).
    pub fn root_event_table(&mut self, name: &str, t: TableRef) {
        let Some(root) = self.roots.first().cloned() else { return };
        t.borrow_mut().set_str("name", Value::str(name));
        t.borrow_mut().set_str("controller", Value::Num(0.0));
        let pe = self.vm.index(&root, &Value::str("processEvent")).unwrap_or(Value::Nil);
        self.call(pe, vec![root, Value::Table(t)]);
    }

    /// An event whose data is a list (`hud_update_rewards`: the event is
    /// indexed 1, 2, 3 ... as the scripts walk it).
    pub fn root_event_list(&mut self, name: &str, items: Vec<Value>) {
        let Some(root) = self.roots.first().cloned() else { return };
        let t = Table::new_ref();
        t.borrow_mut().set_str("name", Value::str(name));
        t.borrow_mut().set_str("controller", Value::Num(0.0));
        for (i, v) in items.into_iter().enumerate() {
            t.borrow_mut().set(Value::Num((i + 1) as f32), v);
        }
        let pe = self.vm.index(&root, &Value::str("processEvent")).unwrap_or(Value::Nil);
        self.call(pe, vec![root, Value::Table(t)]);
    }

    /// The mouse, as the engine sends it to the root: `name` is
    /// mousemove / mousedown / mouseup, (x, y) in root units, `button`
    /// left / right.
    pub fn mouse(&mut self, name: &str, x: f32, y: f32, button: &str) {
        self.root_event(
            name,
            &[
                ("rootName", Value::str("UIRoot0")),
                ("x", Value::Num(x)),
                ("y", Value::Num(y)),
                ("button", Value::str(button)),
                // His mouse is controller 0's (the menus a controller owns
                // take only its input).
                ("controller", Value::Num(0.0)),
            ],
        );
    }

    /// A pad button (or the key that stands for it): primary, secondary,
    /// start, up, down, left, right, ...
    pub fn button(&mut self, button: &str, down: bool) {
        self.root_event(
            "gamepad_button",
            &[
                ("button", Value::str(button)),
                ("down", Value::Bool(down)),
                ("qualifier", Value::str("keyboard")),
                // His keys and pad are controller 0 (the lobby, opened for
                // a controller, takes only that one's buttons).
                ("controller", Value::Num(0.0)),
            ],
        );
    }

    /// A letter key on PC: BO2's prompts that show a key (F Friends,
    /// S Search Preferences) fire on `key_shortcut` with that key's name.
    pub fn key_shortcut(&mut self, key: &str) {
        self.root_event(
            "gamepad_button",
            &[
                ("button", Value::str("key_shortcut")),
                ("key", Value::str(key)),
                // The key's bound command: a prompt with no key of its own
                // fires when this matches its bind, so it is never nil (nil
                // matched every such prompt: F worked as Back).
                ("bind1", Value::str("")),
                ("down", Value::Bool(true)),
                ("qualifier", Value::str("keyboard")),
                ("controller", Value::Num(0.0)),
            ],
        );
    }

    /// The engine calls the scripts made since the last time.
    /// bo2mp: a call as if the menus had made it (the shell's own menus'
    /// answers, `MenuResponse`).
    pub fn push_call(&self, call: EngineCall) {
        self.calls.borrow_mut().push(call);
    }

    pub fn take_calls(&mut self) -> Vec<EngineCall> {
        std::mem::take(&mut *self.calls.borrow_mut())
    }

    /// bo2zm M4 (debugging aid): an element's parent chain (id, name, rect).
    pub fn chain_of(&self, id: usize) -> Vec<(usize, String, Option<lui::Rect>)> {
        self.roots
            .iter()
            .filter_map(lui::element)
            .map(|r| lui::chain_of(&r, id))
            .find(|c| !c.is_empty())
            .unwrap_or_default()
    }

    /// bo2zm M4 (debugging aid): every open menu and its input state.
    pub fn menu_report(&self) -> Vec<String> {
        self.roots
            .iter()
            .filter_map(lui::element)
            .flat_map(|r| lui::menu_report(&r))
            .collect()
    }

    /// The menus open over the HUD (the root's children other than the HUD
    /// menu: `Menu.class`, popups).
    pub fn open_menus(&self) -> Vec<String> {
        let Some(root) = self.roots.first().and_then(lui::element) else {
            return Vec::new();
        };
        lui::child_ids(&root)
            .into_iter()
            .filter(|id| id.starts_with("Menu.") && id != "Menu.HUD")
            .collect()
    }

    /// bo2mp: close the open menu whose id is `id` (e.g. a confirm
    /// prompt's `Menu.SetDefaultPopup`; the newest if several) as its own NO
    /// does (`CoD.PopupMenus.GoBack(menu, { controller = 0 })`); false if
    /// none is open.
    pub fn close_popup(&mut self, id: &str) -> bool {
        fn find(u: &Rc<UserData>, id: &str, out: &mut Option<Rc<UserData>>) {
            if lui::with(u, |e| e.fields.borrow().get_str("id").as_str().is_some_and(|i| i == id)) {
                *out = Some(u.clone());
            }
            for k in lui::with(u, |e| e.children.clone()) {
                find(&k, id, out);
            }
        }
        let mut found = None;
        for root in self.roots.clone() {
            if let Some(r) = lui::element(&root) {
                find(&r, id, &mut found);
            }
        }
        let Some(menu) = found else {
            return false;
        };
        let popups = self.field("CoD", "PopupMenus");
        let Ok(go_back) = self.vm.index(&popups, &Value::str("GoBack")) else {
            return false;
        };
        let event = crate::value::Table::new_ref();
        event.borrow_mut().set_str("controller", Value::Num(0.0));
        self.call(go_back, vec![Value::User(menu), Value::Table(event)]).is_some()
    }

    /// bo2mp: set the text of an engine-fed label of an open menu (the
    /// Loading screen's `statusLabel`: the engine writes its connect
    /// status there). False if the menu or label is not there.
    pub fn set_menu_label_text(&mut self, menu_id: &str, field: &str, text: &str) -> bool {
        fn find(u: &Rc<UserData>, id: &str, out: &mut Option<Rc<UserData>>) {
            if lui::with(u, |e| e.fields.borrow().get_str("id").as_str().is_some_and(|i| i == id)) {
                *out = Some(u.clone());
            }
            for k in lui::with(u, |e| e.children.clone()) {
                find(&k, id, out);
            }
        }
        let mut found = None;
        for root in self.roots.clone() {
            if let Some(r) = lui::element(&root) {
                find(&r, menu_id, &mut found);
            }
        }
        let Some(menu) = found else { return false };
        let Ok(label) = self.vm.index(&Value::User(menu), &Value::str(field)) else { return false };
        if matches!(label, Value::Nil) {
            return false;
        }
        let Ok(set) = self.vm.index(&label, &Value::str("setText")) else { return false };
        self.call(set, vec![label, Value::str(text)]).is_some()
    }

    /// The engine ids of a menu's elements (its `id`, e.g.
    /// `Menu.Scoreboard`), wherever it was added.
    pub fn menu_element_ids(&self, id: &str) -> Option<std::collections::HashSet<usize>> {
        self.roots
            .iter()
            .filter_map(lui::element)
            .find_map(|r| lui::ids_under_named(&r, id))
    }

    /// Open a menu on the root (the root's `addmenu`), as the engine does.
    pub fn open_menu(&mut self, menu: &str) {
        self.root_event("addmenu", &[("menu", Value::str(menu))]);
    }

    /// One frame at `now_ms`: start the animations the scripts wrote, move
    /// every animation on, and send each one that ended its
    /// `transition_complete_<name>` (the handlers may start more, so this
    /// repeats a few times).
    pub fn frame(&mut self, now_ms: f64) {
        for (el, ev) in lui::stream_events(now_ms) {
            self.event(&Value::User(el), ev, &[]);
        }
        for _ in 0..4 {
            lui::commit_pending();
            let done = lui::tick(now_ms);
            if done.is_empty() {
                break;
            }
            for (el, anim, interrupted, late) in done {
                // BO2ZM_LUI_TRANSLOG=1: every animation's end (debugging aid).
                if std::env::var_os("BO2ZM_LUI_TRANSLOG").is_some() {
                    let id = lui::with_id(&el);
                    eprintln!(
                        "bo2zm lui transition #{id} {anim} interrupted={interrupted} at {now_ms:.0}"
                    );
                }
                let target = Value::User(el);
                // `interrupted` only when a new animation cut it short (the
                // globe's handlers test `event.interrupted == nil`).
                let mut fields = vec![("lateness", Value::Num(late as f32))];
                if interrupted {
                    fields.push(("interrupted", Value::Bool(true)));
                }
                self.event(&target, &format!("transition_complete_{anim}"), &fields);
            }
        }
    }

    /// LUI's clock (the last frame's time).
    pub fn now_ms(&self) -> f64 {
        lui::now_ms()
    }

    /// The element with this id, as a value (debugging aid).
    pub fn element_by_id(&self, id: usize) -> Option<Value> {
        self.roots
            .iter()
            .filter_map(lui::element)
            .find_map(|r| lui::find(&r, id))
            .map(Value::User)
    }

    /// The element trees under the roots (debugging aid).
    pub fn tree(&self) -> Vec<String> {
        let mut out = Vec::new();
        for r in self.roots.iter().filter_map(lui::element) {
            lui::tree(&r, 0, &mut out);
        }
        out
    }

    /// Every element in drawing order, in root units.
    pub fn drawn(&self) -> Vec<Drawn> {
        let (w, h) = self.root_size;
        self.roots
            .iter()
            .filter_map(lui::element)
            .flat_map(|root| lui::layout_clipped(&root, [0.0, 0.0, w, h], &*self.measure))
            .map(|(u, rect, alpha, clip)| drawn_of(&u, rect, alpha, clip))
            .collect()
    }

    /// bo2mp: `drawn`, less what no one sees (alpha at most `min_alpha`):
    /// that is still walked (its places kept) but not copied out. With the
    /// count of every element walked.
    pub fn drawn_shown(&self, min_alpha: f32) -> (Vec<Drawn>, usize) {
        let (w, h) = self.root_size;
        let mut all = 0;
        let shown = self
            .roots
            .iter()
            .filter_map(lui::element)
            .flat_map(|root| lui::layout_clipped(&root, [0.0, 0.0, w, h], &*self.measure))
            .inspect(|_| all += 1)
            .filter(|(_, _, alpha, _)| *alpha > min_alpha)
            .map(|(u, rect, alpha, clip)| drawn_of(&u, rect, alpha, clip))
            .collect();
        (shown, all)
    }

    /// bo2mp: the entity containers on the roots to place this frame (see
    /// `lui::entity_containers`).
    pub fn entity_containers(&self) -> Vec<(Rc<UserData>, i32, [f32; 3], bool)> {
        let roots: Vec<Rc<UserData>> = self.roots.iter().filter_map(lui::element).collect();
        lui::entity_containers()
            .into_iter()
            .filter(|(u, ..)| {
                let t = lui::top(u);
                roots.iter().any(|r| Rc::ptr_eq(r, &t))
            })
            .collect()
    }

    /// bo2mp: place one (see `lui::set_entity_screen`); it hears
    /// `entity_container_clamped` / `_unclamped` when that changes.
    pub fn place_entity_container(&mut self, u: &Rc<UserData>, screen: Option<[f32; 2]>, clamped: bool, toward: f32) {
        if let Some(on) = lui::set_entity_screen(u, screen, clamped, toward) {
            let name = if on { "entity_container_clamped" } else { "entity_container_unclamped" };
            self.event(&Value::User(u.clone()), name, &[]);
        }
    }
}

fn drawn_of(u: &Rc<UserData>, rect: Rect, alpha: f32, clip: Option<Rect>) -> Drawn {
    let d = u.data.borrow();
    let e = d.downcast_ref::<lui::Element>().expect("element");
    let s = &e.state;
    Drawn {
        id: e.id,
        kind: e.kind,
        rect,
        alpha,
        rgb: [s.red, s.green, s.blue],
        material: s.material.clone(),
        text: s.text.clone(),
        font: s.font.clone(),
        alignment: match (s.alignment, s.left_anchor, s.right_anchor) {
            (0, true, false) => 1,
            (0, false, true) => 3,
            // Both anchors pinned: a box with width reads from its middle
            // (the rank-up text); a zero-width one is a place the text
            // starts at, as BO2's Setup Bots values (a selector's value
            // text in a list slot of no width) do.
            (0, true, true) if rect[2] - rect[0] <= 0.5 => 1,
            (0, ..) => 2,
            (a, ..) => a,
        },
        anchors: (s.left_anchor, s.right_anchor),
        z_rot: s.z_rot + e.extra_rot,
        x_rot: s.x_rot,
        y_rot: s.y_rot,
        dashes: e.dashes,
        dash_pitch: e.dash_pitch,
        tiles: e.tiles,
        shader: s.shader,
        blur: e.blur,
        behind: e.behind,
        clip,
    }
}

fn font_name(u: &UserData) -> Option<String> {
    if u.kind != "font" {
        return None;
    }
    u.data.borrow().downcast_ref::<String>().cloned()
}

/// A zombies mode's group, the gametypes table's tenth column: `zclassic`
/// for the story modes, `zsurvival` for zstandard, `zencounter` for Grief
/// and Turned (`ui_zm_gamemodegroup`).
pub fn mode_group(tables: &HashMap<String, Vec<Vec<String>>>, mode: &str) -> Option<String> {
    tables.get("zm/gametypestable.csv").and_then(|rows| {
        rows.iter()
            .find(|c| c.first().is_some_and(|x| x == "0") && c.get(1).is_some_and(|x| x == mode))
            .and_then(|c| c.get(9).filter(|g| !g.is_empty()).cloned())
    })
}

/// The map a start loads. BO2's start-location menu
/// (`selectstartloczombie`) loads Turned at the Diner as its own map,
/// `zm_transit_dr` (the Diner cut out, with Turned's scripts); its other
/// such rule, `zm_transit_tm`, has no zone on PC. Every other start loads
/// the map it was picked on.
pub fn start_map(map: &str, loc: &str, mode: &str) -> String {
    if map.contains("zm_transit") && loc == "diner" && mode == "zcleansed" {
        return "zm_transit_dr".to_owned();
    }
    map.to_owned()
}

/// A dvar by name, any case (CoD's dvar names ignore case: the lobby asks
/// `ui_gameType`, the engine keeps `ui_gametype`).
pub fn dvar_get(dvars: &HashMap<String, String>, name: &str) -> Option<String> {
    dvars
        .get(name)
        .or_else(|| {
            dvars
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v)
        })
        .cloned()
}

/// Set a dvar, keeping the spelling it already has.
fn dvar_set(dvars: &mut HashMap<String, String>, name: &str, value: String) {
    let key = dvars
        .keys()
        .find(|k| k.eq_ignore_ascii_case(name))
        .cloned()
        .unwrap_or_else(|| name.to_owned());
    dvars.insert(key, value);
}

/// A Zombies word the loaded English text lacks (Die Rise's
/// `ZMUI_ROOFTOP_CAPS`, in the map list and the lobby after a game) shows
/// as its own name in capitals (ROOFTOP), never the raw key. Not the
/// per-map words BO2's scripts probe and expect to be missing
/// (`ZMUI_GRIEF_zm_prison_CAPS`: the lobby then says GRIEF, as BO2 does,
/// not GRIEF ZM PRISON).
fn caps_word(up: &str) -> Option<String> {
    let word = up.strip_prefix("ZMUI_")?.strip_suffix("_CAPS")?;
    if word.contains("ZM_") {
        return None;
    }
    Some(word.replace('_', " "))
}

pub(crate) fn set_field(t: &Value, name: &str, v: Value) {
    if let Value::Table(t) = t {
        t.borrow_mut().set_str(name, v);
    }
}

/// bo2zm M4: the lobby's members (`Engine.GetPlayersInLobby`, the lobby
/// updates' `members`): him alone, host of his own party, ready.
/// The engine's party size for the lobby he is in (`party_maxplayers`, the
/// lobby list's "1 Player (N Max)"): a Custom Games lobby
/// (CoD.GAMEMODE_PRIVATE_MATCH) holds `party_maxplayers_privatematch`, the
/// Theater (CoD.GAMEMODE_THEATER) `party_maxplayers_theater` (4), the
/// public lobby `party_maxplayers_partylobby` (8 in the real game); a
/// Public Match's game lobby holds the match's four.
pub fn party_size(v: &mut EngineValues) {
    if v.public_lobby_since.is_some() {
        v.dvars
            .insert("party_maxplayers".into(), ZOMBIES_MAX_PLAYERS.to_string());
        return;
    }
    let key = if v.game_modes.contains(&5.0) {
        "party_maxplayers_theater"
    } else if v.game_modes.contains(&1.0) {
        "party_maxplayers_privatematch"
    } else {
        "party_maxplayers_partylobby"
    };
    if let Some(n) = v.dvars.get(key).cloned() {
        v.dvars.insert("party_maxplayers".into(), n);
    }
}

/// bo2zm H28: a map's shown name (zm/mapstable.csv column 3, localized:
/// `TRANZIT`) from its load name (`zm_transit`, `zm_transit_dr`).
pub fn map_shown_name(v: &EngineValues, load: &str) -> String {
    let maps = v.tables.get("zm/mapstable.csv");
    let cell = |r: &Vec<String>, i: usize| r.get(i).cloned().unwrap_or_default();
    let row = maps.and_then(|m| {
        m.iter()
            .find(|r| cell(r, 0) == load)
            .or_else(|| m.iter().find(|r| !cell(r, 0).is_empty() && load.starts_with(&cell(r, 0))))
    });
    let Some(key) = row.map(|r| cell(r, 3).to_ascii_uppercase()) else {
        return load.to_owned();
    };
    v.localize.get(&key).cloned().or_else(|| caps_word(&key)).unwrap_or(key)
}

pub fn lobby_members(v: &EngineValues) -> Value {
    let m = Table::new_ref();
    {
        let mut t = m.borrow_mut();
        let name = if v.player_name.is_empty() {
            "Player"
        } else {
            v.player_name.as_str()
        };
        t.set_str("name", Value::str(name));
        t.set_str("gamertag", Value::str(name));
        t.set_str("clean_gamertag", Value::str(name));
        t.set_str("clantag", Value::str(""));
        t.set_str("tagprefix", Value::str(""));
        // Rank 1's icon (mp/rankicontable_zm.csv, the menus' column).
        t.set_str("rankIcon", Value::str("menu_zm_rank_1"));
        t.set_str("xuid", Value::str("1"));
        t.set_str("playerXuid", Value::str("1"));
        t.set_str("clientNum", Value::Num(0.0));
        t.set_str("controller", Value::Num(0.0));
        t.set_str("team", Value::Num(v.team as f32));
        // No rank number: the real Zombies lobby row shows only the rank
        // icon (the card adds the rank, below). The score ("0") only in a
        // Custom Games lobby (CoD.GAMEMODE_PRIVATE_MATCH), not the public one.
        t.set_str("prestige", Value::Num(0.0));
        if v.game_modes.contains(&1.0) {
            t.set_str("score", Value::Num(0.0));
        }
        // The player card's rank icon column (rankicontable_zm: 5 + days).
        t.set_str("daysPlayedInLast5Days", Value::Num(0.0));
        // A number, not a flag: the rows test `isLocal == 1` (his own row
        // is gold, others blue) and sort by it.
        t.set_str("isLocal", Value::Num(1.0));
        // Numbers too: the lobby lists sort by isHost and isGuest with `<`
        // (a flag there is a script error and the list never redraws).
        t.set_str("isHost", Value::Num(if v.party_follower { 0.0 } else { 1.0 }));
        t.set_str("isGuest", Value::Num(0.0));
        t.set_str("isInParty", Value::Bool(true));
        t.set_str("isReady", Value::Bool(true));
        t.set_str("subparty", Value::Num(0.0));
        t.set_str("leagueTeamID", Value::Num(0.0));
    }
    let list = Table::new_ref();
    list.borrow_mut().set(Value::Num(1.0), Value::Table(m));
    // bo2zm H28: the others in his Public Match lobby (blue rows; the
    // first is the host when he joined someone else's).
    for (i, name) in v.party.iter().enumerate() {
        let o = Table::new_ref();
        {
            let mut t = o.borrow_mut();
            for key in ["name", "gamertag", "clean_gamertag"] {
                t.set_str(key, Value::str(name));
            }
            t.set_str("clantag", Value::str(""));
            t.set_str("tagprefix", Value::str(""));
            t.set_str("rankIcon", Value::str("menu_zm_rank_1"));
            let xuid = (i + 2).to_string();
            t.set_str("xuid", Value::str(&xuid));
            t.set_str("playerXuid", Value::str(&xuid));
            t.set_str("clientNum", Value::Num((i + 1) as f32));
            t.set_str("controller", Value::Num(0.0));
            t.set_str("team", Value::Num(v.team as f32));
            t.set_str("prestige", Value::Num(0.0));
            t.set_str("daysPlayedInLast5Days", Value::Num(0.0));
            t.set_str("isLocal", Value::Num(0.0));
            let host = v.party_follower && i == 0;
            t.set_str("isHost", Value::Num(if host { 1.0 } else { 0.0 }));
            t.set_str("isGuest", Value::Num(0.0));
            t.set_str("isInParty", Value::Bool(false));
            t.set_str("isReady", Value::Bool(true));
            t.set_str("subparty", Value::Num(0.0));
            t.set_str("leagueTeamID", Value::Num(0.0));
        }
        list.borrow_mut().set(Value::Num((i + 2) as f32), Value::Table(o));
    }
    Value::Table(list)
}

/// bo2zm H28: the lobby row with this xuid ("1" is his, "2".. the others').
pub fn lobby_row(v: &EngineValues, xuid: &str) -> Option<TableRef> {
    let m = lobby_members(v);
    let list = m.table()?.borrow();
    (1..=list.len())
        .filter_map(|i| list.get(&Value::Num(i as f32)).table().cloned())
        .find(|t| t.borrow().get_str("xuid").as_str() == Some(xuid))
}

/// The players the menus count in a playlist, a map or a start location
/// (BO2 counts everyone online; here it is the one local lobby).
const LOBBY_COUNT: f32 = 1.0;

/// bo2zm M4: one playlist: its id, its map's super category (one per map
/// and filter), the map, the filter (`playermatch` / `solomatch`), the
/// start location and game mode (refs and shown names).
struct PlaylistRow {
    id: usize,
    super_index: usize,
    map: String,
    filter: &'static str,
    loc: String,
    loc_name: String,
    /// The playlist's description, shown (`ZMUI_STANDARD_DESC_NUKED`).
    loc_desc: String,
    /// The map's name (zm/mapstable.csv column 3, localized: `NUKETOWN`).
    map_name: String,
    mode: String,
    mode_name: String,
}

/// BO2's loading screen words (the engine sets them as a map loads; the
/// Loading menu shows them): `ls_mapname` NUKETOWN, `ls_maplocation` and
/// `ls_gametype` from the playlist the lobby picked.
pub fn set_loading_dvars(v: &mut EngineValues) {
    let get = |n: &str| dvar_get(&v.dvars, n).unwrap_or_default();
    let (map, loc, mode) = (get("ui_mapname"), get("ui_zm_mapstartlocation"), get("ui_gametype"));
    let rows = playlist_rows(v);
    let Some(row) = rows
        .iter()
        .find(|r| r.map == map && r.loc == loc && r.mode == mode)
        .or_else(|| rows.iter().find(|r| r.map == map))
    else {
        return;
    };
    let words = [
        ("ls_mapname", row.map_name.clone()),
        ("ls_maplocation", row.loc_name.clone()),
        ("ls_gametype", row.mode_name.clone()),
    ];
    for (name, value) in words {
        dvar_set(&mut v.dvars, name, value);
    }
}

/// The start location and mode kept to the map the lobby shows: when the
/// map changes under them (Custom Games opens with `ui_mapname 0`, the
/// first map, and no start set), the map's first start in that mode, else
/// its first start.
pub fn fit_start(v: &mut EngineValues) {
    let get = |n: &str| dvar_get(&v.dvars, n).unwrap_or_default();
    let (map, loc, mode) = (get("ui_mapname"), get("ui_zm_mapstartlocation"), get("ui_gametype"));
    let rows = playlist_rows(v);
    if rows.iter().any(|r| r.map == map && r.loc == loc && r.mode == mode) {
        return;
    }
    let Some(row) = rows
        .iter()
        .find(|r| r.map == map && r.mode == mode)
        .or_else(|| rows.iter().find(|r| r.map == map))
    else {
        return;
    };
    let (loc, mode) = (row.loc.clone(), row.mode.clone());
    dvar_set(&mut v.dvars, "ui_zm_mapstartlocation", loc);
    dvar_set(&mut v.dvars, "ui_gametype", mode);
}

/// The start locations and game modes this rebuild plays, per map (map,
/// location, mode). A map with no entry offers every start its table has
/// (every Zombies map is on the globe).
pub const OFFERED: &[(&str, &str, &str)] = &[];

/// Starts that do not play yet, kept off the menus (map, location, mode;
/// an empty mode shuts the whole location). Checked one by one, hidden.
/// (Turned at the Diner came off 10-09: it loads its own map, `start_map`.)
pub const SHUT: &[(&str, &str, &str)] = &[];

fn offered(map: &str, loc: &str, mode: Option<&str>) -> bool {
    let shut = SHUT.iter().any(|(m, l, g)| {
        *m == map && *l == loc && (g.is_empty() || mode.is_some_and(|x| x == *g))
    });
    // A map with no entry here offers everything its table has.
    !shut && (!OFFERED.iter().any(|(m, ..)| *m == map)
        || OFFERED
            .iter()
            .any(|(m, l, g)| *m == map && *l == loc && mode.is_none_or(|x| x == *g)))
}

/// The playlists the menus offer: every start location and game mode the
/// gametypes table gives the maps this game plays, for Public Match and
/// for Solo (ids from 1 in that order).
fn playlist_rows(v: &EngineValues) -> Vec<PlaylistRow> {
    let rows = v
        .tables
        .get("zm/gametypestable.csv")
        .cloned()
        .unwrap_or_default();
    let cell = |r: &Vec<String>, i: usize| r.get(i).cloned().unwrap_or_default();
    // (A missing ZMUI_X_CAPS word shows as X: `caps_word`.)
    let shown = |k: String| {
        let up = k.to_ascii_uppercase();
        v.localize.get(&up).cloned().or_else(|| caps_word(&up)).unwrap_or(k)
    };
    let maps = v.tables.get("zm/mapstable.csv").cloned().unwrap_or_default();
    // Every start the table gives the globe's maps, once (load checks).
    static LOGGED: std::sync::Once = std::sync::Once::new();
    LOGGED.call_once(|| {
        for r in rows.iter().filter(|r| cell(r, 0) == "6") {
            let (map, loc, mode) = (cell(r, 2), cell(r, 3), cell(r, 4));
            eprintln!("bo2zm lui: table start map={map} loc={loc} mode={mode}");
        }
    });
    let mut out = Vec::new();
    let mut super_index = 0;
    for filter in ["playermatch", "solomatch"] {
        for map in &v.maps {
            super_index += 1;
            let map_name = maps
                .iter()
                .find(|r| cell(r, 0) == *map)
                .map_or_else(|| map.clone(), |r| shown(cell(r, 3)));
            for loc in rows
                .iter()
                .filter(|r| cell(r, 0) == "5" && cell(r, 2) == *map && offered(map, &cell(r, 3), None))
            {
                let loc_ref = cell(loc, 3);
                for m in rows
                    .iter()
                    .filter(|r| {
                        cell(r, 0) == "6"
                            && cell(r, 2) == *map
                            && cell(r, 3) == loc_ref
                            && offered(map, &loc_ref, Some(&cell(r, 4)))
                    })
                {
                    let mode = cell(m, 4);
                    let mode_key = rows
                        .iter()
                        .find(|g| cell(g, 0) == "0" && cell(g, 1) == mode)
                        .map(|g| cell(g, 2))
                        .unwrap_or_default();
                    out.push(PlaylistRow {
                        id: out.len() + 1,
                        super_index,
                        map: map.clone(),
                        filter,
                        loc: loc_ref.clone(),
                        loc_name: shown(cell(loc, 4)),
                        loc_desc: {
                            // BO2's playlists describe a mode at a place:
                            // `ZMUI_STANDARD_DESC_NUKED` ("Mass-energy
                            // equivalence, ..."), the location's own line
                            // when that is missing.
                            let k = format!(
                                "ZMUI_{}_DESC_{}",
                                mode.trim_start_matches('z'),
                                loc_ref
                            )
                            .to_ascii_uppercase();
                            v.localize
                                .get(&k)
                                .cloned()
                                .unwrap_or_else(|| shown(cell(loc, 5)))
                        },
                        map_name: map_name.clone(),
                        // Each map's story mode is named after the map
                        // (BO2's lobby reads DIE RISE, not TRANZIT).
                        mode_name: if mode == "zclassic" {
                            map_name.clone()
                        } else {
                            shown(mode_key)
                        },
                        mode,
                    });
                }
            }
        }
    }
    out
}

/// bo2zm M4: one node of BO2's player stats (`stats.cacLoadouts.
/// resetWarningDisplayed:get()`): any field is another node (made once and
/// kept), `:get()` reads its value (0 until set), `:set(v)` stores it.
fn stat_node(vm: &mut Vm, values: Rc<RefCell<EngineValues>>, path: String) -> Value {
    let t: TableRef = Table::new_ref();
    let meta = Table::new_ref();
    let index = vm.native("stat_index", move |vm, a| {
        let key = arg(&a, 1);
        let name = key.to_string();
        let p = path.clone();
        let out = match name.as_str() {
            "get" => {
                let v = values.clone();
                vm.native("stat_get", move |_, _| {
                    Ok(vec![
                        v.borrow().stats.get(&p).cloned().unwrap_or(Value::Num(0.0)),
                    ])
                })
            }
            "set" => {
                let v = values.clone();
                vm.native("stat_set", move |_, a| {
                    v.borrow_mut().stats.insert(p.clone(), arg(&a, 1));
                    Ok(vec![])
                })
            }
            _ => {
                let child = stat_node(
                    vm,
                    values.clone(),
                    if p.is_empty() {
                        name
                    } else {
                        format!("{p}.{name}")
                    },
                );
                if let Value::Table(me) = arg(&a, 0) {
                    me.borrow_mut().set(key, child.clone());
                }
                child
            }
        };
        Ok(vec![out])
    });
    meta.borrow_mut().set_str("__index", index);
    t.borrow_mut().meta = Some(meta);
    Value::Table(t)
}

/// A table whose missing fields are do-nothing functions, counted.
fn stub_table(
    vm: &mut Vm,
    name: &'static str,
    asked: Rc<RefCell<BTreeMap<String, usize>>>,
) -> Value {
    let t: TableRef = Table::new_ref();
    let meta = Table::new_ref();
    let index = vm.native("stub_index", move |vm, a| {
        let key = arg(&a, 1).to_string();
        *asked
            .borrow_mut()
            .entry(format!("{name}.{key}"))
            .or_insert(0) += 1;
        Ok(vec![vm.native("stub_fn", |_, _| Ok(vec![]))])
    });
    meta.borrow_mut().set_str("__index", index);
    t.borrow_mut().meta = Some(meta);
    Value::Table(t)
}

/// bo2mp: he plays with a controller now (`Engine.LastInput_Gamepad`).
static GAMEPAD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_gamepad_active(on: bool) {
    GAMEPAD.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn gamepad_active() -> bool {
    GAMEPAD.load(std::sync::atomic::Ordering::Relaxed)
}

/// bo2mp: map categories the Change Map popup lists after BO2's own (Base,
/// Revolution, ... Apocalypse): (name, mapPackTypeIndex, icon material).
/// The popup shows `MPUI_MAP_CATEGORY_<name>_CAPS` / `_DESC` and the maps
/// whose mapstable row carries that index.
static EXTRA_MAP_CATEGORIES: std::sync::Mutex<Vec<(String, u32, String)>> = std::sync::Mutex::new(Vec::new());

pub fn set_extra_map_categories(categories: Vec<(String, u32, String)>) {
    if let Ok(mut c) = EXTRA_MAP_CATEGORIES.lock() {
        *c = categories;
    }
}

/// bo2mp: once `CoD.ChangeMap` is defined, its category tables get the
/// extra categories (each game's maps, a page at a time).
fn extend_map_categories(vm: &mut Vm) {
    let extra = match EXTRA_MAP_CATEGORIES.lock() {
        Ok(c) if !c.is_empty() => c.clone(),
        _ => return,
    };
    let Value::Table(cod) = vm.global("CoD") else {
        return;
    };
    let change = cod.borrow().get_str("ChangeMap");
    let Value::Table(change) = change else {
        return;
    };
    let names = change.borrow().get_str("CategoryNames");
    let index = change.borrow().get_str("CategoryIndex");
    let icons = change.borrow().get_str("CategoryIconNames");
    let max = change.borrow().get_str("MaxCategories").as_num();
    let (Value::Table(names), Value::Table(index), Some(max)) = (names, index, max) else {
        return;
    };
    // Once: the first extra category already there means it was done.
    let first = Value::str(&extra[0].0);
    let done = (1..=names.borrow().len()).any(|i| names.borrow().get(&Value::Num(i as f32)).raw_eq(&first));
    if done {
        return;
    }
    let mut n = max as usize;
    for (name, pack, icon) in &extra {
        n += 1;
        let at = Value::Num(n as f32);
        names.borrow_mut().set(at.clone(), Value::str(name));
        index.borrow_mut().set(at.clone(), Value::Num(*pack as f32));
        if let Value::Table(icons) = &icons {
            icons.borrow_mut().set(at, Value::str(icon));
        }
    }
    change.borrow_mut().set_str("MaxCategories", Value::Num(n as f32));
}

/// bo2mp: Black Ops' in-game HUD is drawn in place of these BO2 widgets
/// (ammo, weapon and grenades; the team scores and the clock; the
/// scorestreak column, its slanted boxes and momentum bar: hud.lua builds
/// it as `LUI.createMenu.RewardSelection(controller)`). Set `BO2_HUD=bo2`
/// to keep BO2's own.
const BO1_HUD_REPLACES: [&str; 3] = ["AmmoArea", "ScoreBottomLeft", "RewardSelection"];

/// bo2mp: Black Ops' HUD is on (the default; `BO2_HUD=bo2` turns it off).
pub fn bo1_hud_on() -> bool {
    !std::env::var("BO2_HUD").is_ok_and(|v| v.trim().eq_ignore_ascii_case("bo2"))
}

/// bo2mp: Zombies' classic HUD (World at War's and Black Ops' look: the
/// round in red chalk bottom-left, the points over a blood splash and the
/// ammo bottom-right) is drawn by the game in place of these BO2 widgets:
/// the round (`CoD.RoundStatus`), the players' points
/// (`CoD.CompetitiveScoreboard`) and the ammo and d-pad area
/// (`LUI.createMenu.AmmoAreaZombie`). The perks, power-ups and each map's
/// own widgets (parts, Afterlife, the staffs) stay BO2's. `BO2ZM_HUD=bo2`
/// keeps BO2's whole HUD.
const ZM_CLASSIC_TABLES: [&str; 2] = ["RoundStatus", "CompetitiveScoreboard"];

/// bo2mp: Zombies' classic HUD is on (the default; `BO2ZM_HUD=bo2` off).
pub fn zm_classic_hud() -> bool {
    !std::env::var("BO2ZM_HUD").is_ok_and(|v| v.trim().eq_ignore_ascii_case("bo2"))
}

/// Marks what `f` returns `bo1Hidden` (built and fed, never drawn) in a
/// Zombies game.
fn hiding(vm: &mut Vm, name: &'static str, f: Value) -> Value {
    vm.native(name, move |vm, args| {
        let out = vm.call(f.clone(), args)?;
        let zombies = matches!(vm.global("CoD"), Value::Table(cod) if cod.borrow().get_str("isZombie").truthy());
        if zombies {
            for v in &out {
                if let Some(u) = lui::element(v) {
                    lui::with(&u, |e| e.fields.borrow_mut().set_str("bo1Hidden", Value::Bool(true)));
                }
            }
        }
        Ok(out)
    })
}

/// See `ZM_CLASSIC_TABLES`: each table's functions (its constructors among
/// them) are wrapped once, as soon as its script is in.
fn hide_for_zm_classic(vm: &mut Vm) {
    if !zm_classic_hud() {
        return;
    }
    let Value::Table(cod) = vm.global("CoD") else {
        return;
    };
    for t in ZM_CLASSIC_TABLES {
        let Value::Table(table) = cod.borrow().get_str(t) else { continue };
        if table.borrow().get_str("bo2mpClassic").truthy() {
            continue;
        }
        let mut funcs = Vec::new();
        let mut key = Value::Nil;
        while let Some((k, v)) = table.borrow().next(&key) {
            if matches!(v, Value::Func(_)) {
                funcs.push((k.clone(), v));
            }
            key = k;
        }
        for (k, f) in funcs {
            let wrapped = hiding(vm, t, f);
            table.borrow_mut().set(k, wrapped);
        }
        table.borrow_mut().set_str("bo2mpClassic", Value::Bool(true));
        eprintln!("bo2mp zombies hud: BO2's {t} hidden for the classic HUD");
    }
    let Value::Table(lui) = vm.global("LUI") else {
        return;
    };
    let Value::Table(menus) = lui.borrow().get_str("createMenu") else {
        return;
    };
    let original = menus.borrow().get_str("AmmoAreaZombie");
    if matches!(original, Value::Func(_)) && !menus.borrow().get_str("bo2mpClassicAmmo").truthy() {
        let wrapped = hiding(vm, "AmmoAreaZombie", original);
        menus.borrow_mut().set_str("AmmoAreaZombie", wrapped);
        menus.borrow_mut().set_str("bo2mpClassicAmmo", Value::Bool(true));
    }
}

/// bo2mp: the widgets Black Ops' HUD replaces are still built and still get
/// their events (BO2's HUD script expects them), but each one is marked
/// `bo1Hidden` and the layout never draws it. Zombies keeps its own.
fn hide_for_bo1_hud(vm: &mut Vm) {
    if !bo1_hud_on() {
        return;
    }
    hide_bo2_scorestreaks(vm);
    let Value::Table(lui) = vm.global("LUI") else {
        return;
    };
    let menus = lui.borrow().get_str("createMenu");
    let Value::Table(menus) = menus else {
        return;
    };
    for name in BO1_HUD_REPLACES {
        let original = menus.borrow().get_str(name);
        if !matches!(original, Value::Func(_)) {
            continue;
        }
        let wrapped = vm.native(name, move |vm, args| {
            let out = vm.call(original.clone(), args)?;
            let zombies = match vm.global("CoD") {
                Value::Table(cod) => {
                    let on = cod.borrow().get_str("isZombie").truthy();
                    on
                }
                _ => false,
            };
            if !zombies && let Some(u) = out.first().and_then(lui::element) {
                let id = lui::with(&u, |e| {
                    let was = e.fields.borrow().get_str("bo1Hidden").truthy();
                    e.fields.borrow_mut().set_str("bo1Hidden", Value::Bool(true));
                    (!was).then_some(e.id)
                });
                if let Some(id) = id {
                    eprintln!("bo2mp modern hud: BO2's {name} hidden (element #{id})");
                }
            }
            Ok(out)
        });
        menus.borrow_mut().set_str(name, wrapped);
    }
    // The HUD's square minimap (`CoD.Compass.AddMinimap(hud)` builds it as
    // `hud.miniMapContainer`): Black Ops' round one is drawn instead. The
    // pause menu's full map (`AddInGameMap`) stays BO2's.
    let Value::Table(cod) = vm.global("CoD") else {
        return;
    };
    let compass = cod.borrow().get_str("Compass");
    let Value::Table(compass) = compass else {
        return;
    };
    let original = compass.borrow().get_str("AddMinimap");
    if !matches!(original, Value::Func(_)) {
        return;
    }
    let wrapped = vm.native("AddMinimap", move |vm, args| {
        let out = vm.call(original.clone(), args.clone())?;
        let zombies = match vm.global("CoD") {
            Value::Table(cod) => {
                let on = cod.borrow().get_str("isZombie").truthy();
                on
            }
            _ => false,
        };
        if zombies {
            return Ok(out);
        }
        let hide = |v: &Value| {
            if let Some(u) = lui::element(v) {
                lui::with(&u, |e| e.fields.borrow_mut().set_str("bo1Hidden", Value::Bool(true)));
            }
        };
        if let Some(v) = out.first() {
            hide(v);
        }
        if let Some(hud) = args.first().and_then(lui::element) {
            let container = lui::with(&hud, |e| e.fields.borrow().get_str("miniMapContainer"));
            hide(&container);
        }
        Ok(out)
    });
    compass.borrow_mut().set_str("AddMinimap", wrapped);
}

/// bo2mp: the scorestreak events BO2's scorestreak column (rewardselection.lua)
/// answers: the engine's list of his picks and his momentum.
const SCORESTREAK_EVENTS: [&str; 2] = ["hud_update_rewards", "hud_update_momentum"];

/// bo2mp: the modern HUD draws his scorestreak column itself, so BO2's is
/// built and fed as before but never drawn: whichever element takes the
/// column's events (`registerEventHandler("hud_update_rewards", ...)`) is
/// marked `bo1Hidden`, whatever the script names the widget (its root,
/// `RewardSelection`, is also hidden where it is created: see
/// `BO1_HUD_REPLACES`). The HUD menu itself, should it ever take them, is
/// left alone; Zombies keeps its own. `LUI.UIElement.registerEventHandler` is wrapped once, as soon as
/// LUI's element class is in.
fn hide_bo2_scorestreaks(vm: &mut Vm) {
    let Value::Table(lui) = vm.global("LUI") else {
        return;
    };
    let class = lui.borrow().get_str("UIElement");
    let Value::Table(class) = class else {
        return;
    };
    if class.borrow().get_str("bo1WrappedHandlers").truthy() {
        return;
    }
    let original = class.borrow().get_str("registerEventHandler");
    if !matches!(original, Value::Func(_) | Value::Native(_)) {
        return;
    }
    let wrapped = vm.native("registerEventHandler", move |vm, args| {
        let event = args.get(1).and_then(Value::as_str).map(str::to_owned);
        let out = vm.call(original.clone(), args.clone())?;
        if let Some(event) = event.filter(|e| SCORESTREAK_EVENTS.contains(&e.as_str()))
            && let Some(u) = args.first().and_then(lui::element)
        {
            let zombies = match vm.global("CoD") {
                Value::Table(cod) => {
                    let on = cod.borrow().get_str("isZombie").truthy();
                    on
                }
                _ => false,
            };
            // (RewardSelection is itself a menu, `menuName` set: only the
            // HUD's own root is spared.)
            let is_hud = lui::with(&u, |e| e.fields.borrow().get_str("menuName").as_str() == Some("HUD"));
            if !zombies && !is_hud {
                let id = lui::with(&u, |e| {
                    let was = e.fields.borrow().get_str("bo1Hidden").truthy();
                    e.fields.borrow_mut().set_str("bo1Hidden", Value::Bool(true));
                    (!was).then_some(e.id)
                });
                if let Some(id) = id {
                    eprintln!("bo2mp modern hud: BO2's scorestreak widget hidden (element #{id}, takes `{event}`)");
                }
            }
        }
        Ok(out)
    });
    let mut c = class.borrow_mut();
    c.set_str("registerEventHandler", wrapped);
    c.set_str("bo1WrappedHandlers", Value::Bool(true));
}

/// BO2's custom-games lobby offers Setup Bots only in its own list of game
/// types (FFA, TDM, CTF, Demolition, Domination, Hardpoint, HQ and Kill
/// Confirmed) and greys it out elsewhere. Bots are wanted in every mode, so
/// once the lobby script has defined that check it is replaced by one that
/// always says yes.
fn allow_bots_in_every_mode(vm: &mut Vm) {
    let Value::Table(cod) = vm.global("CoD") else {
        return;
    };
    let lobby = cod.borrow().get_str("PrivateGameLobby");
    let Value::Table(lobby) = lobby else {
        return;
    };
    let check = lobby.borrow().get_str("DoesGametypeSupportBots");
    if !matches!(check, Value::Func(_)) {
        return;
    }
    let always = vm.native("DoesGametypeSupportBots", |_, _| Ok(vec![Value::Bool(true)]));
    lobby.borrow_mut().set_str("DoesGametypeSupportBots", always);
}

#[cfg(test)]
mod tests {
    use super::script_key;

    #[test]
    fn script_names_meet_modules() {
        assert_eq!(
            script_key("ui_mp/t6/hud.lua"),
            script_key("ui_mp_t6_hud.lua")
        );
        assert_eq!(
            script_key("ui_mp__t6__zombie__basezombie.lua"),
            format!("uimp{}", script_key("T6.Zombie.BaseZombie"))
        );
    }
}

#[cfg(test)]
mod host_tests {
    use super::Host;
    use crate::value::Value;

    #[test]
    fn unanswered_engine_fields_are_counted() {
        let mut h = Host::new(Vec::new(), Box::new(|_, _, _| 0.0));
        let e = h.vm.global("Engine");
        let f = h.vm.index(&e, &Value::str("SomeUnboundField")).unwrap();
        assert!(matches!(f, Value::Native(_)));
        assert_eq!(h.asked.borrow().get("Engine.SomeUnboundField"), Some(&1));
    }
}
