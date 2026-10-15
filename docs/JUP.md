# Modern Warfare III (2023, "jup", cod23) — containers

Code: `crates/fastfile_jup` (`container.rs`, `xsub.rs`, `oodle.rs`,
`oodle_dll.rs`), probe `cargo run -p fastfile_jup --example jup_inventory --
<install> [--list]`. Read on the retail build 1.21; everything little-endian.
Zone contents (asset list, layouts) are a separate layer, not described here.

## Oodle

Every compressed run is an Oodle LZ stream; the zones' chunk headers say
compressor 6 (Kraken in the open decoders' numbering), first bytes `8C 06`
(compressed) or `CC 06` (stored). No pure-Rust decoder with a licence that
fits this Apache-2.0 tree exists: `oozle` is GPL-3.0, `oozextract` is labelled
MIT but is a structural port of GPL-3.0 `ooz`. So `oodle::Decompressor` is a
trait; on Windows `oodle::load(install)` loads the game's own
`oo2core_8_win64.dll` from the install root (or `$JUP_OODLE_DLL`) at runtime.
The DLL is never shipped or committed. `StoredOnly` (pure Rust) decodes stored
chunks only — enough for the tests. A pure decoder drops in behind the trait.

## `.ff` (`zone/*.ff`)

```
0x000 "IWffa100", u32 0x18, u32 1, u8[4] flags (01 00 04 xx | 01 00 05 00)
0x014 u32 file length - 0x120
0x038 u64 decompressed zone size      (0x040.. more u64s for the zone layer)
0x0e0 "TAFF" ...
0x120 auth region, 0x8000 bytes: u8 2 "IWC" "IWffs100", signature bytes,
      zone name at 0x250 (NUL padded), the first master block
0x8120 data header (12): u32 ?, u32 zone size (low 32 bits), u24 block size
      0x10000, u8 compressor 6
0x812c blocks: u32 compressed, u32 decompressed, u32 0, Oodle stream;
      next header at the next 4-byte boundary
```

Counting the data header as chunk 0, a 0x4000-byte master block sits before
every chunk whose index is a multiple of 512 (before blocks 511, 1023, ...).
The last block ends exactly at EOF; every block but the last decodes to
0x10000 bytes. No searching is needed. Swept on all 16,927 `.ff` of 1.21
(13.6 GB): every one walks strictly to EOF with decoded lengths summing to
0x038; 234 of them (random 200 + the 15 largest, the smallest, Rebirth tiles,
`mp_jup_rust`, `ingame_br`) fully decoded through the DLL, 6.5 GB of zone, all
exact. 563,269 blocks, 2,076 of them stored; 577 master blocks. Not encrypted.

## `KAPI` packages (`zone/*.xsub`, `zone/*.xpak`, `xpak_cache/`)

```
0x000 "KAPI", u16 flavor (1 = .xsub part / cache, 2 = zone .xpak index),
      u16 version 0x17, u64 1, u64 kind (3 in .xsub), u64 file length, u64 hash
0x028 u64 part count (.xpak), 0 in a part
0x788 u64 file count, data offset (0x8000), data size,
      hash count, hash table offset, hash table size (= count * 20)
hash row (20, sorted by key): u64 key; u64 packed: bit 0 compressed,
      bits 1..31 size in the file, bits 32..63 offset / 128; u32 decoded size
```

A stored entry is its bytes. A compressed entry is one or more segments of
at most 1 MiB, each: `u16 0xF01D`, u64 key (= the row's), u32 this segment's
offset in the entry, u32 the next one's (0 = last), u16 segment count, u16
index, u8 block count, then per block 21 bytes (u8 compressor, u32 compressed,
u32 decoded, u32 stream offset from the segment start, u32 offset in the
decoded entry, u32 checksum — not checked), then the streams. Decoded blocks
are 0x3ffe0 bytes, a segment's last shorter. Segments start exactly 1 MiB
apart. Swept on every entry of the 139 parts: 517,923 stored, 2,021,780
compressed (2,013,394 in one segment, the rest in 2 to 9+), all segment
headers and block tables consistent and summing to the row's decoded size;
2.4 M blocks, all compressor 6; a random 4,906 of them decoded through the
DLL, and `jup_inventory` decodes the first/middle/last 3 entries of each
part (1,231) and the first entry of every `.xpak` with parts.

The 139 `.xsub` parts (2,539,703 entries) carry data and their own table at
the end. The 39 zone `.xpak` are tables only, one row per entry of all their
parts (`<name>-00000.xsub` ...): offset = part × stride + offset in the part,
the stride being the smallest power of two that holds the largest part
(256 MiB for `boot`, 1 GiB for `codhq23a`). Checked on every row of the 31
that have parts (2,497,482 rows, all equal to the part's own row); the 8
`*_cdn.xpak` (kind 4) name no installed parts. `xpak_cache/cache_N_2.xpak`
(flavor 1, kind 1) index `cache_N_2.xpakdata` (entries start `1D F0`); not
read yet.

## Resurgence's sounds (`gamemode_iw4::resurgence::SOUNDS`)

Read at runtime from `ingame_br`, `eng_ingame_br`, `eng_mp_jup_escape5`,
`ingame_shared_mp` and `mp_jup_escape5` (`asset_audio::jup_resurgence_sounds`):
1.21 has 1,527 aliases in `ingame_br`, 893 in `mp_jup_escape5`, 1,615 English
voice lines in `eng_ingame_br` and 30 in `eng_mp_jup_escape5`; the battle
royale's pickups, cash, armor hit markers, purchase and end stings are in
`ingame_shared_mp` (16,628 aliases with `ingame_mp` and the base zones). Alias names are FNV-1a
hashes; a route names an alias where the name is known and `#<hash>` where
it is not.

- Names: the music (`mus_mx_mp_br_ri_*`, plain in `ingame_br`), and ~480
  sound names recovered by hashing: identifiers of the zones
  (`jup_alias_names`), and words of the zones joined under prefixes met from
  each hash's end (`jup_alias_brute --tails`: `br_splash_mission_accept`,
  `br_circle_closing`, `uin_iw9_br_victory_slam`, `br_pickup_cash_med_01`,
  `hit_marker_armor_break_plr`, `ui_select_purchase_confirm`, `emt_escape5_*` ...).
- The announcer: the voice lines of the two English zones were decoded from
  the install and run through a local speech-to-text model on the owner's PC
  (transcripts stay there, `C:\RE-Tools\jup\br\vo_all.tsv`; nothing of them
  is committed), then sorted into events by what they say. 304 of the 1,645
  hashed lines are Resurgence's (redeploys, the circle and safe zone,
  squads and players left, the countdown numbers, contracts, UAVs, loadout
  drops, buy station, victory and placings, infil and landing); 879 belong
  to the other modes these zones also carry (DMZ, Plunder, Lockdown, the
  Gulag, events) and 462 are flavour lines with no event here.
- Rebirth Island's ambience: the 37 stereo `wz23` streams of
  `mp_jup_escape5` of 90–97 s (beds), chained one after another.

Not named yet, so these events keep Black Ops II's stand-ins (or the
nearest MW3 sound, said in the route): the plate insert itself (the armor
box's refill plays), plates full, squads remaining, gas tick/enter, the
circle closed, the loadout crate landing, the first spawn.

## Emblems and calling cards (`cards.rs`, probe `jup_cards`)

All of them are defined in `zone/global.ff` (13.7 MB, 44 MB of zone), one
script bundle (type 115) each. A bundle's string cells hold its name in the
clear (`-2`, a 32-bit hash, 4 zero bytes, the name, NUL): `ui_emblem_005`,
`jup_emblem_n5934_1`, `ui_playercard_100`,
`jup_playercard_animated_opc_outbreak_mike4_tier1_atlas`, ... (calling cards
are "player cards"), and its title's localize key (`EMBLEMS/EMBLEM_005`;
the title is encrypted and not read). Its picture is the image asset of the
same name (`hash::name(name)`), stored just before the bundle; on 1.21 every
one has a single level (image header 0x2c) in a single streamed group,
found in `boot_shared` (the older ones), `codhq23b` / `codhq23a` (the
seasons') or `codhq_shared` (87). Image format byte 0x25 is BC1 (cards),
0x30 BC7.

1.21: 5,315 names, 5,272 with a picture in the zone: 2,889 emblems (2,182
flipbooks) and 2,383 calling cards (1,710 flipbooks); 2 pictures are in no
installed package (fetched on demand), 5,270 are there.

Flipbooks are frames side by side in rows, an emblem's 256x256, a card's
512x128: a 768x1024 emblem is 3 by 4 frames, a 1024x1024 one 4 by 4, a
1024x1024 card 2 by 8, a 1536x512 card 3 by 4. Neither the bundle's cells
nor the name tells one from a still picture (`ui_emblem_143` and
`ui_emblem_286`, both 1024x1024, have the same cells; some `_atlas` names
are 256x256 stills). The size decides where it can (a card of whole frames
not itself 4:1, an emblem of whole frames not square: a flipbook); a square
emblem of several frames is a flipbook when its neighbouring frames are
nearly the same picture (`detect_grid`).

The game shows them at runtime through `crates/ui/src/jup_cards.rs`
(`jup:<name>`, a frame `jup:<name>#<n>`), from `$BO2_JUP_INSTALL` or the
game folders file's `jup` line.

## Maps (`fastfile_jup::map`, `assets::lane::jup_map`)

Played: Rebirth Island (`mp_jup_escape5`, Resurgence) and Tokyo
(`mp_jup_elevated`); the rest of the install's `mp_jup_*` maps are left out
(`asset_transport::JUP_MAPS`). Tokyo's zone is the one dressed as Japan
(`jup_appliance_vending_machine_food_japan_01`, `jup_sign_arcade_japan_*`,
`jup_sign_convenience_japan_*`, pachinko and ramen props: 198 `japan`
names, no other map zone more than one). Start one with `map jup:<zone>`.

- World: the `gfx_map`'s 128-byte surface records over `gfx_map_trzone`
  vertex streams (`map/world.rs`). Each record names its world zone: 0 the
  map zone's own, k the k-th `<map>_*_0_tr` zone in name order (Rebirth: 24
  grid tiles `_<x>_<y>_0_0_tr` and 11 swap zones; checked by every zone's
  packed triangle bytes). Swap zones' other states (`_swap_dest_`,
  `_swap_t_`) are left out. Triangle tables run in the records' packed order
  but for a few surfaces placed later: matched by size, read in table order.
  Rebirth: 8297 surfaces, 1.40 M triangles, read in about 9 s.
- Materials (`map/materials.rs`): the `gfx_map`'s material table (back-
  pointers to asset-list entries, all materials) and a 40-byte record per
  surface (+0 surface record, +4 table entry, +0x0e world zone). Material
  headers (0x78 bytes; +0x1b picture count, +0x30 the pictures) are found by
  shape as the run of exactly the zone's material count (Rebirth: 5841);
  pictures are asset-list entries of the image run. Format 0x25 (BC1) is
  the colour maps' form; 0x2a (BC1) holds masks and terrain blend maps, 0x2f
  BC7 (normal maps, x y z in red green blue), 0x30 BC7. Pixels: the largest streamed group within 1024 px from the
  zone `.xpak` indexes (levels placed from the top: BC1 streams to 2x2).
  A material whose picture table is another material's points into a block
  whose offsets drift from the zone image: the shift is voted from the
  materials' own tables (Rebirth: 655 tables, about 62 MB). Terrain (virtual
  textures, 256x256 tiles of formats 0x28/0x29) still draws without a
  picture (Rebirth's largest grey: 0x2a mask only, and tables of none).
- Placed models (`map/models.rs`): the `gfx_map`'s model table (16 bytes,
  back-pointer to the xmodel's entry), collections (16 bytes: first, count,
  model) and instances (24 bytes: u16 quaternion as v/65535*2-1, f16 scale,
  i32 origin in 1/4096 units). XModel headers (0xE8 bytes, name inline,
  `jhash(name)` at +0) stand in asset order; LODs (0x48 bytes) name their
  `xmodelsurfs`, whose meshes stream from the `.xpak` by the key at +0x10.
  Headers pair with surfs entries by aligning surface counts. Positions span
  `(2v-1)` times the half extents. Shadow casters, `tm7/` stand-ins and
  destructibles' other states are left out. Rebirth: 3580 models, 130,785
  placements, 4-6 s. LOD0 is drawn out to the last LOD's distance (at least
  1500); a coarse LOD collides inside the play area, not foliage, decals or
  effects' models.
- Sheets (flat, wider than 16384 units): the sea (Rebirth: 14 at z=64) is
  drawn cut to the play area and 16384 units around, not collided; the sea
  bed and vista sheets are not drawn (they stretched the bounds to ±144k).
- Sky: not found in the zone (the only cube is 128x141 format 0x12); a
  procedural daylight cube stands in.
- Collision: the drawn world and placed models inside the play area; the
  zone's `col_map` is not read. Spawns: made over the map's path
  nodes when the install has its entity text
  (`jupiter/Assets/MapEnts/SpawnVars/<map>.d3dbsp`: Tokyo), else over the
  walkable floor of the play area (Rebirth).
- World surfaces' extra stream (record +0x34, 4 bytes a vertex) holds blend
  weights, not colours.
- Not read yet: lightmaps / light grid, terrain virtual textures, `col_map`,
  the zone's `map_ents`, the real sky.
- A hidden test run spawns with `BO2MP_AUTOCLASS=class_smg` (the console's
  `spawn 0` waits for a class menu nobody answers):
  `iw4l map jup:mp_jup_escape5 --cmds "wait world; wait 50s; force_match_start &; …"`.

## Guns (`fastfile_jup::gun`, `assets::lane::jup_gun`)

Played: the Renetti (`jup_jp12_pi_mike93`, `bo2_profile::jup_names::JUP_PLAYED`);
the other named guns are not listed in Create-a-Class, for bots or in the box.

- Parts: a gun is a receiver and the attachments it carries, each its own
  model in a `_tr` zone: `mp_av_*` the view model's, `mp_aw_*` the world
  model's (the Renetti: `jp19_pi_mike93_rec_000` receiver, `bar_000` slide
  and barrel, `mag_15_9mm_000`, `pgrip_000`). A zone is the mesh (world
  zones: one per LOD, then a detail collision), the model, its skin
  variants and a streaming info.
- Mesh positions are 21-bit fractions spanning the largest extent either
  side of the centre, in the model's space; the parts fit the receiver
  only so. Bones: per bone a 32-byte pose in the model (quaternion,
  position, 2.0), then the FNV-1a 32 hashes of their names (`j_gun`,
  `tag_barrel_attach`, `tag_mag_attach`, `tag_pistolgrip_attach`,
  `tag_flash`, `tag_brass`, ...). A part's root is at its origin and goes
  on the receiver's tag of its kind.
- Materials: one pointer per surface, -1/-2 for a 0x78-byte reference stub
  after the pointers (the material's hash), else a pointer back to a
  material loaded before (not read: drawn as dark gun metal). Gun
  materials are defined in `global_stream_mp` (one pass for the header
  word `c0 b6 73 2c`); their pictures are asset-list entries there, the
  list's place found from the pointers themselves. A gun material's
  colour is a tint in its constants (row 6, sRGB bytes), the game laying
  camo and wear over it; its pictures are a normal map (format 0x2f, BC7,
  x y z), an occlusion picture (0x30, usage 0x36, occlusion in alpha) and
  masks. The colour map built is the tint darkened by the occlusion.
- In the match the gun is a copy of a Black Ops II gun (the B23R) with its
  own models, name and handling (semi-automatic, 15 rounds): Black Ops
  II's view animations, arms, effects, sounds and HUD pictures
  (`WeaponRegistry::borrowed_namespace_of`). The models are one bone,
  `tag_weapon` at the receiver's grip tag, with `tag_flash` and
  `tag_brass`.
- Not read: Modern Warfare III's view animations (animpkg, xanims), its
  weapon numbers, and its weapon sounds: the gun's sound package
  (`jup_pi_mike93_sfxpackage` in `ingame_shared_mp`) names a handful of
  `ingame_shared_mp` aliases (shells, a 0.46 s stereo sound twice) and
  otherwise hashes no alias or sndasset of the zones read names.
- `iw4l check-guns` (`BO2MP_CHECK_GUNS=jup`) reads and builds the models.
