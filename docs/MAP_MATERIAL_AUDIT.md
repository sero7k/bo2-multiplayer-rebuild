# Map material audit: CoD4, World at War, Black Ops 1

Generated 2026-10-13 from the game files at `/mnt/user-data/uploads/Documents`.
Per-map data, including up to 200 missing material/image names per map
sorted by world triangles, is in `docs/map_material_audit.json`.

## Summary

| Game | Map zones in this copy | Worlds built | Colour maps decoded | Status |
|---|---|---|---|---|
| CoD4 (`iw3`) | **0**: `cod4_game_files/main` is empty and there is no `zone/` | n/a | n/a | not auditable |
| World at War (`t4`) | 23 (`waw_game_files/zone/english/mp_*.ff`) | 23 of 23 OK | **0** of 1,401 distinct world colour images; 0 of 1,365 prop pictures | all 23 incomplete |
| Black Ops 1 (`t5`) | 1 (`bo1_game_files/zone/Common/mp_nuked.ff`) | 1 of 1 (484 batches) | **0** of 399 distinct colour images | incomplete |

**The main cause is that this copy has no IWD archives.** Neither
`waw_game_files/main/` nor `bo1_game_files/main/` exists. Both lanes
read every streamed colour, normal and specular map from `main/*.iwd`:
`cod4.rs::link_world_materials` uses `decode_ui_image_from_main`, and
`t5.rs` uses `decode_material_color_maps`. With no `main/`, every world
surface draws flat grey (the `(1,1,[128,128,128,255])` stand-in), apart
from the few BO2-look swaps. The audit checks of the zones and
references themselves are almost all clean:

- 7,689 of the 7,793 WaW material rows (world + prop) are `iwd_absent`.
  The zone resolves the material and its colour image correctly; only
  the IWD file is missing.
- 65 rows are `,name` image references (for example `,global_black_c`,
  `,decal_flag_japan_sun_c`). The lane reads these by bare name from the
  same IWDs, so they are also missing only because the IWDs are absent.
- Only **3** references are actually broken:
  - `mp_nachtfeuer` world material `,*34n_96_5n_2`, a composite-blend
    reference found in neither `localized_common_mp` nor `common_mp`.
  - `mp_downfall` and `mp_roundhouse` prop material `,mc/mtl_fx_dest_atlas`
    (the destructible FX atlas).
- No zone read stopped early: every WaW zone reports `N of N assets`.
  There are no unresolved material ids and no surfaces without a
  material.
- The audit could not test for unsupported image formats, because no
  `.iwi` is present. The decoder (`asset_material::parse_iwi_header` /
  `iwi_pixel_format`) handles IWI v6 (CoD4 and WaW) and v13 (BO1), and
  formats ARGB8, RGB8, LA8, L8, A8, DXT1/3/5 and DXN. That covers every
  format these games ship, so a full copy should decode nearly
  everything. The probe's `not_in_iwds`, `unsupported_format` and
  `decode_error:` statuses will report anything that doesn't.

The gaps that remain with a full copy (design gaps, not missing data):

1. **Blend layers are dropped (WaW and CoD4).** `link_world_materials`
   binds only the first `Color` texture. Composite world materials such
   as `*57n_5n` (technique `l_sm_r0c0n0s0_sco_b1c1n1s1_sco`) blend 2 or 3
   colour layers. These are 4–22% of each map's world triangles (mean
   12%) and 56–81% of the distinct world material entries. Rubble over
   concrete, mud over cobbles and bullet-hole decal layers all lose
   their second and third layers.
2. **Normal and specular maps are unused (WaW and CoD4).** The
   fallback pass draws colour × lightmap only (`world mesh: ... fallback
   draw with colour maps and its lightmaps`), although nearly every
   `wc_l_sm_r0c0n0s0*` material carries normal and specular maps.
   BO2-look swaps are the only legacy surfaces that get normal and
   specular maps. **Fixed on `wip/legacy-lighting`:** `link_world_materials`
   binds each lit material's own `normalMap` (DXT5 xGxR moved to Black Ops
   II's layout, `world_iw3::iw3_normal_to_bo2`) and `specularMap`. The IW3
   branch of the fallback pass bumps the lightmap's dominant light and the
   sun by the normal map, as CoD4's `lm_*n0*` shaders do. It adds the
   specular map's probe reflection and sun highlight from the map's own
   reflection probes (`lm_sm_sun_r0c0n0s0`, in gamma space).
3. **BO1 technique routing.** The T5 catalog reports `unrouted=303` of 585
   materials before decode. 282 of those are stub-routed. Unrouted
   technique families include `mc_l_sm_r#c#n#s#x#` (93),
   `mc_l_sm_r#c#n#s#` (48) and `mc_l_sm_r#c#s#` (13), plus effect and
   particle sets. 66 materials (mostly FX and shadow) have no colour map
   at all.
4. **The BO2-look swap targets are mostly unreadable in this copy.**
   `bo2_game_files/zone/all` has only `mp.ipak`; the per-map packs
   (`mp_la.ipak`, `mp_village.ipak`, `mp_socotra.ipak`,
   `mp_castaway.ipak`, `zm_nuked.ipak`, ...) are absent. 34 `wpc/*` swap
   targets fail `in no pack` (for example `wpc/mud_pak_sludge02_lambert`
   on 20 maps, `wpc/mp_berlin_floor_wood_hardwood_02` on 19,
   `wpc/concrete_wall_broken` and `wpc/iw_ch_metal_rusty_02` on 18).
   Only 2–15 colour images per map are swapped; 193 in total. The model
   maps' colour-grade LUTs are also missing for 20 of 23 maps (for
   example `zm_nuketown_lut_win MISSING (identity)`). Only `mp_village`
   (Seelow, Vodka) and `mp_nightclub` (Makin) read theirs.

**Worst maps (WaW):**

- **By distinct world colour images missing:** Dome (82), Asylum (78),
  Subway (76), Castle (75), Hangar (71).
- **By total material rows (world + props):** Asylum (526), Subway
  (496), Airfield (408), Bgate (398).
- **By BO2-look coverage:** Dome (2 images swapped, 19 targets failed)
  and Docks (3 swapped, 10 failed).
- **With a real broken reference:** Nachtfeuer, Downfall, Roundhouse.
- **Largest dropped-blend share:** Kwai (22%), Shrine (22%), Outskirts
  (20%), Castle (19%).

BO1 Nuketown has 514 of 585 catalog materials missing their colour map.
The catalog is zone-wide (world, props, FX and viewmodel), so the count
includes non-world materials.

## Re-run on the full install (user's PC, 13 Oct)

With World at War's IWDs present, every WaW map's world and prop materials
resolve (`status ok`); the rest are engine images (`$identitynormalmap`,
1-8 per map) and three broken references (Nachtfeuer, Downfall,
Roundhouse; one each). The visible gaps on the full install were the
dropped blend layers: fixed in 54f716cd (colorMap1/2 on the T6 layer slots,
weighted by vertex green/blue). Still open: normal and specular maps on
WaW/CoD4 surfaces, BO1 technique routing, and BO2-look targets whose packs
are missing.

| Map | ok | engine | broken |
|---|---|---|---|
| airfield | 406 | 3 | 0 |
| asylum | 524 | 6 | 0 |
| bgate | 395 | 6 | 0 |
| castle | 343 | 7 | 0 |
| courtyard | 288 | 6 | 0 |
| docks | 299 | 6 | 0 |
| dome | 286 | 4 | 0 |
| downfall | 259 | 2 | 1 |
| drum | 359 | 8 | 0 |
| hangar | 355 | 6 | 0 |
| kneedeep | 345 | 2 | 0 |
| kwai | 282 | 4 | 0 |
| makin | 287 | 7 | 0 |
| makin_day | 294 | 5 | 0 |
| nachtfeuer | 319 | 4 | 1 |
| outskirts | 378 | 1 | 0 |
| roundhouse | 256 | 3 | 1 |
| seelow | 222 | 1 | 0 |
| shrine | 292 | 4 | 0 |
| stalingrad | 298 | 6 | 0 |
| suburban | 357 | 4 | 0 |
| subway | 494 | 3 | 0 |

## Re-run with CoD4 and rendered frames (PC, 15 Oct)

All 21 CoD4 and 23 World at War maps were run three ways: the lane's
census (`map_check_probe`, now ending `textures missing: world N/M
materials (T tris), props P/Q [...]`), the full match load
(`legacy_texture_probe`: lane, BO2 rules, common zones, merge and
finalize, then each drawn world batch's and placed prop's colour binding),
and real frames (`iw4l --cmds "wait world; wait 25; screenshot <name>;
quit" map iw3:<map>`). No IWD image was missing or failed to decode on
either game, the IWD load order was correct, and nothing was lost in the
merge. The missing textures were five other faults:

| Cause | Maps | Fix |
|---|---|---|
| `wc_water` keeps the shader's placeholder `case64blue` (a blue grid of numbers) in its colour slot. The game's water shader draws from its constants and normal map. | 12 CoD4 maps (Backlot, Bog, Broadcast, Cargoship, Convoy, Creek, Farm, Overgrown, Shipment, Showdown, Strike, Vacant) and 21 WaW maps (every one with a pond, stream or puddle) | Drawn as its `waterColor`, less the shader's least sky reflection (`envMapParms.x`), blended by its alpha. |
| `,mc/lambert1`, a common_mp reference that only `code_post_gfx_mp` defines | 7 CoD4 maps (Bloc, Bog, Carentan, Cargoship, Crossfire, Shipment, Vacant), 3–17 prop surfaces each | `code_post_gfx_mp` and `localized_code_post_gfx_mp` are read as common zones too. |
| `,mc/mtl_fx_dest_atlas` (destructibles), defined in `localized_code_post_gfx_mp` | Downfall (15 surfaces) and Roundhouse (853) | As above. |
| Borrowed BO2 sky drawn at brightness 1; the cube is HDR for the model map's exposure | All 23 WaW maps in the BO2 look (white skies on Makin, Airfield, Castle, ...) | The sky row takes the model's `skyColorParm.y × 2^-exposure` (0.08–0.34). |
| Tool and editor surfaces drawn: `wc_tools` (`caulk_shadow`, `aa_default`) and `mc_water_cutaway` (`water_dynamic_editor`, a white slab) | caulk on Bloc, Broadcast, Cargoship (1308 tris), Airfield, Castle, Hangar, Outskirts; aa_default on Showdown; cut-away on Shrine | Shadow-only (not drawn in colour). |

Still open: `mp_nachtfeuer` `,*34n_96_5n_2` (2 triangles) is defined in no
zone of this install. The census and probes match by name and do not
check pixels; a picture that decodes but looks wrong (for example a
greyscale `_c`) is not counted.

## Method

Two offline probes, run sequentially, with
`IW4L_GAMES=/mnt/user-data/uploads/Documents`:

- `target/debug/examples/map_check_probe t4 <zone>`: the existing check
  (`assets::check_map`). It loads the map exactly as a match does
  (lane, `bo2_look`, common zones, props, fog, grade) and prints one
  line. stderr carries the full lane report (BO2-look swaps, failures
  and the grade). Each map takes about 90 s. It refuses `t5`
  (`FAIL no map check for 't5' zones`), so BO1 needed the new probe.
- `target/debug/examples/map_material_audit <iw3|t4|t5> <zone>...`
  (new, `crates/assets/examples/map_material_audit.rs`). It prints one
  JSON object per map.
  - **iw3/t4:** it reads the zone with the `iw3`/`t4` crates, resolves
    `,`-references and technique-less materials against
    `localized_common_mp.ff` and `common_mp.ff` the way the lane does,
    and walks every world-surface material and every placed-prop
    material. For each it reports the technique set, the first colour
    image, the number of colour layers, the number of normal maps, the
    surface type (`surfaceTypeBits`: bit n is CoD4/WaW surface type
    n+1) and a heuristic kind. Its status is one of:
    - `in_zone`: pixels are embedded in the zone.
    - `iwd_absent`: no `main/*.iwd` exists.
    - `not_in_iwds`: the IWDs exist but don't contain
      `images/<name>.iwi`.
    - `unsupported_format` / `decode_error`: the IWI exists but won't
      decode.
    - `reference_unresolved`: a `,name` material missing from both
      common zones.
    - `unresolved_material`: an id with no asset.
    - `no_colour_map`: the material has no colour texture.
    - `engine_reference`: the colour image is a `,name`/`$name`
      reference. The aggregator splits this into `engine_builtin`
      (`$white`, `shadow`, `default`) and `image_reference_iwd` (other
      `,name` images, read by bare name from the IWDs).

    It takes about 45 s per WaW map.
  - **t5:** it loads the map through `assets::lane(ZoneGame::T5)` and
    lists catalog materials whose colour binding has no decoded,
    pending or payload image, plus whether each has a technique route.
- The aggregation script, which sits outside the repo, merged both
  probes into the JSON. To re-run it with full game copies, point
  `IW4L_GAMES` at a tree that has `main/*.iwd`. `iwd_absent` should
  then turn into `ok`, `not_in_iwds` or `unsupported_format`.

Surface kinds come from a name heuristic over the material and colour
image names: `wall`, `floor`, `terrain`, `roof/ceiling`, `trim/detail`,
`decal`, `foliage`, `glass`, `water`, `sky`, `tool/global` and `other`.
World decal surfaces (`GfxWorld.decal_surfs`) are forced to `decal`.
"Top kinds" below are shares of the *missing world triangles*. Name
prefixes seen:

| Prefix | Meaning |
|---|---|
| `wc/` | World material. |
| `mc/` | Model material; BO1 `mc/mtl_*` are props. |
| `*NNn_MMn[_KKn]` | Compiler-generated composite blend materials, 2–3 layers. |
| `,` | Reference to a material or image in a common zone. |
| `~` / `~-g` | Generated specular images or BO1 composite colour images. |
| `decal_*` | Decals. |
| Map-family prefixes | `berlin_`, `okinawa_`, `peleliu_`, `makin_`, `seelow_`, BO1 `us_`, `rus_`, `jun_`, `cub_`, `pent_`. |

## World at War (t4): 23 maps

Columns:

- **World mats / Prop mats:** distinct world-surface materials and distinct
  placed-prop materials.
- **Missing world colour imgs:** the lane's `N not found` (distinct
  colour images) over `linked` world materials.
- **Prop pics missing:** from the lane's props line. Every placed model
  picture is missing, and the lane caps its count at 64.
- **Blend share:** share of world triangles on multi-colour-layer
  materials, whose layers 2–3 are dropped.
- **BO2 look:** the model map, then swapped colour images and
  `in no pack` swap targets.

| Map | World mats | Prop mats | Missing world colour imgs | Prop pics missing | Blend share | Other causes | BO2 look | Top missing kinds (by area) | Status |
|---|---|---|---|---|---|---|---|---|---|
| mp_airfield | 312 | 97 | 63 / 312 | 64 | 16% | 2 `,img` refs | mp_castaway 7 ok / 13 failed | trim/detail 30%, terrain 18%, tool/global 14% | incomplete |
| mp_asylum | 438 | 92 | 78 / 438 | 64 | 17% | 2 `,img` refs | zm_prison 12 ok / 15 failed | wall 25%, roof/ceiling 23%, floor 22% | incomplete |
| mp_bgate | 358 | 43 | 62 / 358 | 41 | 16% | 3 `,img` refs | zm_nuked 9 ok / 11 failed | wall 33%, trim/detail 31%, roof/ceiling 12% | incomplete |
| mp_castle | 263 | 87 | 75 / 263 | 64 | 19% | 6 `,img` refs | mp_magma 15 ok / 12 failed | wall 48%, terrain 17%, tool/global 11% | incomplete |
| mp_courtyard | 209 | 85 | 66 / 209 | 64 | 8% | 5 `,img` refs | mp_magma 13 ok / 11 failed | wall 51%, terrain 13%, roof/ceiling 11% | incomplete |
| mp_docks | 235 | 70 | 65 / 235 | 62 | 4% | 5 `,img` refs | mp_dockside 3 ok / 10 failed | trim/detail 49%, roof/ceiling 17%, wall 15% | incomplete |
| mp_dome | 255 | 35 | 82 / 255 | 34 | 8% | - | zm_nuked 2 ok / 19 failed | wall 45%, trim/detail 39%, terrain 5% | incomplete |
| mp_downfall | 203 | 59 | 45 / 203 | 53 | 14% | **1 unresolved ref** (`,mc/mtl_fx_dest_atlas`), 2 `,img` refs | zm_nuked 5 ok / 10 failed | wall 45%, trim/detail 31%, terrain 10% | incomplete |
| mp_drum | 295 | 72 | 62 / 295 | 64 | 9% | 7 `,img` refs | mp_castaway 4 ok / 11 failed | wall 53%, trim/detail 21%, roof/ceiling 12% | incomplete |
| mp_hangar | 276 | 85 | 71 / 276 | 64 | 13% | 3 `,img` refs | mp_castaway 5 ok / 16 failed | trim/detail 28%, wall 22%, tool/global 20% | incomplete |
| mp_kneedeep | 262 | 85 | 52 / 262 | 64 | 12% | 1 `,img` ref | mp_castaway 10 ok / 10 failed | wall 31%, trim/detail 26%, terrain 12% | incomplete |
| mp_kwai | 179 | 107 | 47 / 179 | 64 | 22% | 1 `,img` ref | mp_castaway 9 ok / 9 failed | decal 31%, foliage 28%, wall 21% | incomplete |
| mp_makin | 200 | 94 | 66 / 200 | 64 | 6% | 4 `,img` refs | mp_nightclub 14 ok / 10 failed | trim/detail 42%, decal 22%, wall 16% | incomplete |
| mp_makin_day | 205 | 94 | 66 / 205 | 64 | 8% | 4 `,img` refs | mp_castaway 14 ok / 11 failed | trim/detail 42%, decal 21%, wall 16% | incomplete |
| mp_nachtfeuer | 265 | 59 | 69 / 265 | 56 | 8% | **1 unresolved ref** (`,*34n_96_5n_2`), 2 `,img` refs | zm_nuked 6 ok / 13 failed | trim/detail 35%, wall 26%, floor 13% | incomplete |
| mp_outskirts | 323 | 56 | 45 / 323 | 54 | 20% | 1 `,img` ref | mp_frostbite 13 ok / 13 failed | wall 59%, terrain 17%, trim/detail 14% | incomplete |
| mp_roundhouse | 205 | 55 | 36 / 205 | 51 | 13% | **1 unresolved ref** (`,mc/mtl_fx_dest_atlas`), 3 `,img` refs | mp_frostbite 5 ok / 14 failed | wall 41%, roof/ceiling 24%, trim/detail 23% | incomplete |
| mp_seelow | 127 | 96 | 41 / 127 | 64 | 16% | 1 `,img` ref | mp_village 6 ok / 10 failed | wall 67%, terrain 15%, trim/detail 7% | incomplete |
| mp_shrine | 198 | 98 | 51 / 198 | 64 | 22% | 4 `,img` refs | mp_magma 6 ok / 10 failed | terrain 35%, wall 18%, tool/global 15% | incomplete |
| mp_stalingrad | 247 | 57 | 52 / 247 | 54 | 10% | 3 `,img` refs | zm_nuked 4 ok / 12 failed | wall 60%, trim/detail 17%, roof/ceiling 15% | incomplete |
| mp_suburban | 265 | 96 | 67 / 265 | 64 | 11% | 2 `,img` refs | mp_hydro 10 ok / 15 failed | wall 47%, trim/detail 16%, roof/ceiling 14% | incomplete |
| mp_subway | 424 | 73 | 76 / 424 | 64 | 9% | 2 `,img` refs | zm_prison 14 ok / 13 failed | trim/detail 33%, wall 25%, floor 14% | incomplete |
| mp_vodka | 240 | 114 | 64 / 240 | 64 | 6% | 2 `,img` refs | mp_village 7 ok / 13 failed | wall 43%, trim/detail 29%, roof/ceiling 15% | incomplete |

Totals: 5,984 world and 1,809 prop material entries. 1,401 distinct
world colour images are missing across maps; some names, such as shared
`berlin_*` images, are counted once per map. All 23 maps build a
world, collision and spawns. Status is `incomplete` everywhere because
the colour maps are absent, not because anything is broken.

Missing world area by kind, over all WaW maps:

| Kind | Share of missing world area |
|---|---|
| wall | 35% |
| trim/detail | 26% |
| terrain | 10% |
| roof/ceiling | 10% |
| decal | 5% |
| floor | 5% |
| tool/global | 4% (mostly `global_black`) |
| foliage | 3% |
| glass, water, other | under 1% each |

## Call of Duty 4 (iw3): no zones

`cod4_game_files/` contains only an empty `main/`, so there are no
`zone/english/mp_*.ff`, no `common_mp.ff` and no `iw_*.iwd`. There is
nothing to audit. The CoD4 lane is the same code path as WaW (`load_iw3_world`
→ `build_legacy_world`), so with a full copy the same caveats apply:
blend layers are dropped, normal and specular maps are unused, and
there is **no BO2 look at all**. `bo2_look::recipe` is only consulted
when `tag == "t4"`. To audit a full copy:
`map_material_audit iw3 <cod4>/zone/english/mp_*.ff`, and
`map_check_probe iw3 ...`.

## Black Ops 1 (t5): mp_nuked

| Map | Catalog materials | Missing colour (distinct imgs) | No colour map | Builtin | Unrouted techniques | World draw | Status |
|---|---|---|---|---|---|---|---|
| mp_nuked | 585 (zone-wide: world, props, FX, viewmodel) | 514 rows / 399 images (498 `iwd_absent` + 16 `,img` refs) | 66 | 5 | 303 pre-decode (282 stub-routed) | 3,494 surfaces, 484 batches, lightmap 1/1, 40 fallback surfaces | incomplete |

- **Lane report:** `IWD material-map gap: no game tree with a 'main/'
  directory above .../bo1_game_files/zone/Common/mp_nuked.ff`.
- **Kinds of missing:** other (mostly `mtl_*` props) 256, decal 64,
  wall 60, terrain 36, foliage 28, trim 24, glass 17, floor 11,
  roof/ceiling 8.
- **Name prefixes:** `mtl_` 270, `decal_` 43, `rus_` 35, `us_` 30,
  `jun_` 23, `t5_` 21, `cub_` 20, `pent_` 18. BO1 colour images are
  `~-g<name>_c` composites, which also live in the IWDs.
- **Other BO1 maps:** `bo1_game_files` has no other MP map zones.
  `en_mp_firingrange.ff` (localized) exists, but `mp_firingrange.ff`
  does not.

## Existing BO2 look (`crates/assets/src/lane/bo2_look.rs`)

- **Scope:** World at War only. `build_legacy_world` calls
  `bo2_look::recipe(&map)` when `tag == "t4"`. CoD4 and BO1 have none.
- **`RECIPES`:** one `Recipe { map, model, swaps }` per WaW map (all 23).
  - `model` is the BO2 map whose grade, fog, reflection probes and sun
    hue the map takes: `zm_nuked`, `zm_prison`, `mp_village`,
    `mp_frostbite`, `mp_hydro`, `mp_nightclub`, `mp_dockside`,
    `mp_castaway` or `mp_magma`.
  - `swaps` is a per-map list of WaW colour image → `zone:material`.
    Only Dome has one (12 entries); the rest are empty.
- **`KEEP`:** substrings that are never swapped (trim, decal, window,
  sign, foliage, fence, flag, and so on). Their UVs are laid out for
  their own atlas.
- **`RULES`:** ordered `(&[substrings], "zone:material")`. The first rule
  whose substrings are all in the WaW colour image name wins. There are
  about 50 rules covering snow, rubble, asphalt, sand, coral, grass,
  mud, dirt, rock, cobble, floors, roofs, brick, plaster, concrete,
  wood, bamboo and metal. Only `*_c` images are considered.
- **`swap_for`:** the map's own list first, then `KEEP`, then `RULES`.
- **`load`:** opens `PackSet::open_dir(<bo2 zone dir>)`, captures the
  model and palette zones, and decodes the colour, normal and specular
  images for each target. Failures are reported as `not: <material>:
  <why>`.
- **`link_world_materials`:** a swapped surface binds the BO2 colour,
  normal and specular images in place of the WaW colour map. Swaps are
  keyed by WaW colour image (so they cover every material using that
  image, including composite `*NNn` layer 0).
- **Off switch:** `BO2MP_BO2_LOOK=0`.

Limitations for the wider remap work:

- The data is compiled into Rust. Adding a map or rule means a rebuild,
  and the tables cannot be reviewed outside code.
- Keys are colour image names only. A material-name key, a
  surface-type key, or a key for a missing image (fallback by kind)
  isn't possible.
- There is no fallback when a target fails: `in no pack` means the
  surface stays on its own (here grey) picture.
- It is wired into one lane only (`cod4.rs`, for `t4`). BO1 materials
  go through the T5 catalog route and never see it.

## Proposed reusable remap design

### 1. One data table

Put the table at `assets/remap/bo2_material_remap.json`, embedded with
`include_str!` so a release build needs no file. A runtime override
comes from `BO2MP_MATERIAL_REMAP=<path>` (layered over the embedded
table). JSON is used because `serde_json` is already a workspace
dependency and TOML is not.

```json
{
  "version": 1,
  "palette_zones": ["mp_la", "mp_village", "mp_socotra", "mp_castaway", "mp_frostbite", "zm_nuked"],
  "keep": ["trim", "decal", "window", "sign", "foliage", "..."],
  "maps": {
    "t4:mp_dome":   { "model": "zm_nuked", "swaps": { "berlin_wall_concrete_grungy_cracks_c": "zm_nuked:wpc/jun_art_concrete_base02_dark" } },
    "t5:mp_nuked":  { "model": "zm_nuked", "swaps": {} },
    "iw3:mp_crash": { "model": "mp_la", "swaps": {} }
  },
  "rules": [
    { "games": ["t4", "iw3", "t5"], "match": { "image_all": ["snow"] }, "to": "mp_frostbite:wpc/dh_snow_powder_flat_01" },
    { "match": { "image_all": ["floor", "wood"] }, "to": "mp_village:wpc/mp_berlin_floor_wood_hardwood_02" },
    { "match": { "material_prefix": "mc/mtl_p_glo_cardboardbox" }, "to": "mp_la:..." }
  ],
  "surface_fallback": {
    "brick": ["mp_village:wpc/brick_wall_redbrick"],
    "concrete": ["mp_la:wpc/concrete_wall_rough_base01", "mp_la:wpc/com_concrete_core_dusty"],
    "wood": ["mp_la:wpc/mp_b_art_floor_wood_dirty_dark", "mp_village:wpc/core_wood_dark"],
    "metal": ["mp_socotra:wpc/jun_art_metal_paintchip_gray"],
    "dirt": ["mp_village:wpc/ch_ground_dirt03"],
    "snow": ["mp_frostbite:wpc/dh_snow_powder_flat_01"],
    "sand": ["mp_castaway:wpc/cas_sand_clean_crushed"],
    "rock": ["mp_castaway:wpc/mtl_p6_cas_rock_large_01"],
    "plaster": ["mp_village:wpc/plaster_wall_plaster01_eggwhite"]
  }
}
```

Keys:

- **Map keys** are `<game>:<map>`.
- **`swaps` keys** may be a colour image (`*_c`) or a material name
  (`wc/...`, `mc/...`).
- **Each target** is `zone:material`; a bare material name means the
  model map's material.
- **`surface_fallback`** is keyed by the material's `surfaceTypeBits`
  name, using the same type list the audit probe prints. It applies
  **only** when the legacy picture is unavailable: missing in IWDs,
  unsupported, or `reference_unresolved`. It never overrides a picture
  that decoded, so authenticity is kept wherever the original exists.
  Each entry is an ordered list; the first target whose pictures decode
  wins. A target that fails `in no pack` falls through to the next, and
  finally to the flat-grey stand-in.

### 2. One resolver, applied in one place

Create `crates/assets/src/lane/material_remap.rs`, replacing the data
half of `bo2_look.rs`:

```rust
pub(crate) struct MaterialKey<'a> { game: &'a str, map: &'a str, material: &'a str,
                                    colour_image: Option<&'a str>, surface: Option<&'a str>,
                                    kind: SurfaceKind /* decal/foliage/sky/... from techset+decal range */,
                                    picture_missing: bool }
pub(crate) enum Remap { Keep, Swap(&'static Target), Fallback(&'static [Target]) }
pub(crate) fn resolve(table: &RemapTable, key: &MaterialKey) -> Remap
```

Resolution order:

1. If the off switch is set, return `Keep`.
2. The per-map `swaps` entry, matched by material name, then by colour
   image.
3. If any `keep` substring matches, or the kind is decal, sky, glass or
   foliage, return `Keep`.
4. The first `rules` entry whose game filter and match predicates pass.
5. If `picture_missing`, the `surface_fallback` entry for the surface
   type.
6. Otherwise `Keep`.

Picture decoding and caching stay in `bo2_look::load` / `swap_images`,
generalised as `RemapLook::prepare(table, game, map, keys) -> Look`.
That function also returns per-target failure reasons so the audit and
`check-maps` can print them.

### 3. Plug-in points

- **`cod4.rs::link_world_materials`** (iw3 and t4): replace
  `look.swaps.get_key_value(image.name)` with `resolve(...)`. Compute
  `picture_missing` from the `decode_ui_image_from_main` result: today
  it is the `missing.push(...)` branch, which would move before the
  swap decision. Also call it for prop materials in
  `legacy_guns::legacy_static_models` (`b.material(mid, PROP_MAX)`), which
  never sees the look today. Drop the `tag == "t4"` gate in
  `build_legacy_world` so CoD4 maps get recipes too.
- **`t5.rs::load_world`**: right after `decode_material_color_maps`,
  walk `materials.materials`. For each one whose colour binding is
  undecoded (the same test the audit uses), or which has a table entry,
  resolve and rebind its colour, normal and specular slots to linked
  BO2 images (`catalog.link_image(AuthoredImage { namespace: T6, name:
  "bo2look:<material>:c", decoded: Some(..) })`), as `cod4.rs` does
  today. The model map's grade, fog and probes come from the same
  `Look` that `cod4.rs` uses (`set_pending_grade`).
- **Reporting**: both lanes push one line in this form:

  ```
  remap: <map> model=<m>: N swapped (by map K, by rule R, by surface fallback F), M kept, X targets failed: <names>
  ```

  `assets::check_map` should accept `t5` (through `lane(ZoneGame::T5)`)
  so `map_check_probe` covers all three games.

### 4. Reversible and safe

- **Off switches:** `BO2MP_BO2_LOOK=0` turns off all remapping (grade,
  fog and probe included), as today. `BO2MP_MATERIAL_REMAP=off` turns
  off material swaps only and keeps the model grade.
  `BO2MP_MATERIAL_REMAP_FALLBACK=0` disables only the surface-type
  fallback. These could become a settings option (`World look: Original
  / Black Ops II`).
- **Non-destructive:** swaps only add catalog images under `bo2look:` names
  and rebind texture slots. Turning a swap off reloads the original
  bindings.
- **Tests:**
  - Keep `rules_swap_tiling_surfaces_and_keep_trims`, moved to run over
    the JSON.
  - Add a test that every target named in the table parses as
    `zone:material` and its zone is in `palette_zones` or is a model.
  - Add a test that every WaW/BO1/CoD4 map key exists in `docs/maps.json`.
- **Audit loop:** `map_material_audit` already emits `colour`,
  `surface`, `kind`, `status` and `tris` per material. Pipe it into a
  small `xtask remap-coverage` that runs `resolve()` over every row and
  lists what would stay grey, ordered by triangles. That gives the
  to-do list for new rules.

### 5. Content follow-ups this audit points to

- Get the IWDs (`main/iw_*.iwd` for CoD4 and WaW, `main/iw_*.iwd` for
  BO1) into the test copy. Until then every legacy map is grey, and
  the audit can't separate `not_in_iwds` from `unsupported_format`.
- Get the BO2 per-map image packs (`mp_la.ipak`, `mp_village.ipak`,
  `mp_socotra.ipak`, `mp_castaway.ipak`, `mp_frostbite.ipak`,
  `zm_nuked.ipak`, `zm_prison.ipak`, ...). Without them, 34 of the
  rule and swap targets fail, and the model colour-grade LUTs are
  missing on 20 of 23 maps.
- Fix the three real reference gaps:
  - `,*34n_96_5n_2` on Nachtfeuer: a composite-blend reference. Either
    synthesize it from its layer images, or fall back to layer 0's
    material `*34n`.
  - `,mc/mtl_fx_dest_atlas` on Downfall and Roundhouse: it lives in
    the FX/destructible shared zone, not in `common_mp` or
    `localized_common_mp`.
- Rendering parity, separate from remapping: draw composite layers 1–2
  (`b1c1`/`t1c1` in the technique name: blend by vertex alpha) and use
  the normal and specular maps in the legacy fallback pass. Blend layers
  alone cover 4–22% of every WaW map's world area.
