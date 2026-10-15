# Black Ops III (T7) maps

MP maps play by Black Ops II's rules; Zombies maps as Black Ops II Zombies
through the Infinite Warfare Zombies pipeline (`lane::iw7_zombies`).

| Part | Where |
|---|---|
| Walk of the map assets | `fastfile_t7/src/walk/{world,gfxworld,world_misc}.rs` |
| Map read into plain data | `fastfile_t7/src/map.rs` (`T7Map::load`) |
| Probe | `t7_map <zone.ff>` (`T7_MAP_LOAD=1` reads it whole, `T7_MAP_DUMP=ents` prints entities) |
| Collision | `asset_world/src/world_t7.rs` |
| Match world, Zombies layout | `assets/src/lane/t7_map.rs` (`load_t7_world`, `zm::read`) |
| Discovery | `asset_transport::discover` (`T7_ZONE_VERSION` 0x251, `TAff0000`, `is_t7_map`) |
| Hooks | `match_walk.rs`, `map_check.rs` (`iw4l check-maps`), UI list "Black Ops III", `iw4l map t7:zm_<map>` |

## What walks (retail, 14 Oct 2026)

Every MP map zone walks to its exact end, `mp_veiled_heyday` included.
Zombies: `zm_factory`, `zm_castle`, `zm_island`, `zm_zod`, `zm_genesis`
and `zm_stalingrad` walk to their exact end and read whole
(`T7_MAP_LOAD=1 t7_map`: world, static models, collision, entities).
`zm_common` and `zm_levelcommon` still stop (player sounds, a weapon).
`T7_TRACE` prints the walker line of each allocation; `t7_scan` finds
pointer runs where a list really starts.

What the Zombies maps needed: XCam's list at 0x68 (16 each, count at 0x70)
and its frame-list count in the u32 at 0x18 when bytes 0x14/0x15 are 0;
ComWorld's second tome list (0x68/0x70, a 24-byte record at a tome's 0x20)
and umbra gates (0x78/0x80, 16 each, a name first), a tome's u32 list
(0x40/0x48); GfxWorld's second tome count at 0x56c (0x568 is a flag);
SAnim's first list holds both u16 counts (0x64 + 0x66) when the first is
under three; LightDescription's 0x210..0x230 are runtime words; a decal
effect's second visuals may load a material rather than an image.

Retail vs x64-zt: GfxWorld exposure volumes 108; siege anim count at 0x578;
`unk` (0x1e8) 48 a record; second umbra list and auto-play list (20) load;
static model draws load posed bones (64) and a siege anim; interactive
water 192 (cells 40, outlines 32, wave clamps 48, a list of 72 at 0x90),
grid memory runtime; post-effect lists at 0x1610.. (vukko boxes 144), 0x1b40..
(bloom mods 384, boxes 464), 0x1b80.. (LUT boxes 144), 0x1ff0.. (colour mods
336, boxes 688), 0x2020/0x2030 (underwater boxes 164). ClipMap static models
load posed bones (32). SAnim: a list at 0x20 (28, count at 0x6c).
StreamerHint 0x28; ZBarrier 0x300 (name only); XCam lists at 0x78 and 0x88.

## What converts

World (IW3 builder, BO3 materials, its own light below), placed models (BO2
model builder), materials hidden by their words (sky, tools, clip,
shadow), entities (plain text, BO2 class names: read as a Treyarch map).
Collision is the zone's own: brushes under a k-d tree of their boxes (BO3's
BSP has a handful of nodes), the triangle mesh with each partition's
vertices in its own segment and all AABB roots under one root, plus the
coarsest LOD of each static model the clip map lists.

Zombies: `zm::read` gives `ZmLayout` (player_volume zones with a trigger
model each, spawners, exterior_goal windows, doors/debris, perks, box
spots, Pack-a-Punch, power switch, wall buys, dog spots, traversals) to
`prepare_layout`; `attach_t6_zm_rules` writes Buried-kit Zombies. Not
carried: teleporters, traps, Gobblegum, Dead Shot. `zm_factory` is also
Der Riese's name: registered as a foreign map, `bo1_maps` leaves it alone.

## Light (`assets::lane::t7_light`, `fastfile_t7::light`)

On by default; `BO2MP_T7_LIGHT=0` gives the old flat light. Only Black Ops
III maps take it. Read from the GfxWorld's first sun volume:

- Sky: the sky box model's material (`mc/sky_latlong_hdr`) names a BC6H
  lat-long picture (8192 x 4096) whose pixels are the sun volume's sky
  buffer (an xpak entry); `$Globals.skyScale` scales it. Lookup (sky shader):
  `u = 0.5 - atan2(d.x, d.y) / 2pi`, `v = acos(d.z) / pi`. Drawn as a 512px
  RGBA16F cube in the probes' unit.
- Probes: one BC6H cube array (256px, 7 levels), stored mip-major (all
  probes' faces at level 0, then level 1, ...), faces world +X -X +Y -Y +Z
  -Z. Configs 328 bytes: word 0 the probe's scale (texels times it are the
  game's light; probe 0's is the map's unit), 1..3 mean colour, 11..13
  origin, 14..22 rotation rows, 23..25 half size, 26..31 parallax box. The
  32 most used round the spawns are drawn (probe 0 first), each surface and
  model taking the smallest box holding it.
- Sun: the sun volume's pitch and yaw aim the light; toward the sun is
  `(-cos p cos y, -cos p sin y, sin p)` (matches the probes' brightest
  texel on Redwood, Nuketown, Apartments). Brightness: intensity over
  probe 0's scale.
- Baked at load (~1 s): a 4096px sun height map over the spawn points'
  square from the world and placed models' opaque triangles (cut-outs
  dithered), and per world vertex the ambient (probe ambient cube by
  normal, times each probe's scale over the unit) times a short-ray
  occlusion; placed models take the mean ambient at their origin.
- Shader (`t7_shade` in `geometry_diagnostic.wgsl`): albedo x (ambient +
  sun x N.L x sun map x moving shadows), plus with normal/specular maps the
  ambient bent by the probe's level 6, a GGX sun highlight and the probe's
  reflection at level `5 (1 - gloss)` (BO3's `lit` shader), split-sum
  weighted; then a filmic tone curve (`BO2MP_T7_EXPOSURE`, default 0.6)
  and Black Ops II's roll-off, bloom and grade.

Not read: the probe volume (gProbeX/Y/Z 3D textures), the sun shadow tree,
lightmaps (BO3 has none for MP), the exposure grids, BO3's own colour LUT,
height fog, parallax-corrected probe lookups.

## Not done

Fog, the probe volume; glass and doors static; map pictures; Zombies menu
entries (use `iw4l map t7:zm_factory`).
