# Optional M.O.A.B. (bo2mp)

An optional streak after Modern Warfare 3's M.O.A.B. It is **off by
default**, so native Black Ops II stays untouched. To turn it on, open the
lobby's SETTINGS tab and set **M.O.A.B.** to **25 / 20 / 15 / 30 KILLS**,
or set `BO2MP_MOAB=<kills>`.

Script: `crates/sim/src/t6/scripts/moab.gsc`. It is OUR CODE, compiled at
load like `_globallogic`.

## Where it comes from

| Part | Source |
|---|---|
| 10 s countdown with a ticking timer; incoming sound 3.3 s before the blast; blast sound and flash vision; deaths 1.5 s after the blast; aftermath vision | **Verified:** Modern Warfare 2's `maps/mp/killstreaks/_nuke.gsc` (plain text in MW2's common_mp) |
| Match goes on, enemies die, enemy team is EMP'd | Modern Warfare 3's MOAB. MW3's scripts ship compiled on PC, so these are its published rules, not its code. |
| EMP | Black Ops II's own `_emp::emp_use`. Its aftermath already restores `level.nukevisionset`. |
| Kill count (25), hold [Use] for 1 s to call it in | **Design.** It is earned per kill streak in one life, so it stays outside Black Ops II's three scorestreak slots. |
| Visions `mpnuke` and `mpnuke_aftermath` | **Design:** our values in the IW film form, modelled on MW2's look |
| Sounds | MW2/MW3 aliases when their sounds are installed, plus Black Ops II's own blast, timer and incoming sounds |

## Settings (dvars)

| Dvar | Default | Meaning |
|---|---|---|
| `scr_moab` | 0 | 1 allows the M.O.A.B. |
| `scr_moab_kills` | 25 | Kills in one life to earn it |
| `scr_moab_timer` | 10 | Countdown in seconds |
| `scr_moab_emp` | 1 | 1 EMPs the enemy team after the blast |
| `scr_moab_endsgame` | 0 | 1 ends the match, as MW2's tactical nuke does |

## Not done yet

- **Slow motion:** `setslowmotion` is a no-op in this engine.
- **Flash and aftermath effects:** MW2's are left out (`player_death_nuke_flash`, `nuke_aftermath_mp`).
