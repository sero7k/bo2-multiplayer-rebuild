# Call of Duty: Ghosts (IW6, PC x64) zones

Phase 1: open any Ghosts `.ff`, count its assets by type, read its sound
aliases, find their streams in `soundfile*.pak`, and play Ghosts' MP menu
music in our front end. Code: `crates/fastfile_iw6` (format),
`crates/audio/src/music/ghosts.rs` (music director hook), `iw4l check-ghosts`
(probe). Everything below was checked on the retail files
(`code_post_gfx_mp`, `patch_code_post_gfx_mp`, `code_pre_gfx_mp`, `common_mp`,
`patch_common_mp`, `common_core_mp`, `ui_mp`, `mp_character_room`,
`mp_prisonbreak`, `soundfile1.pak`) unless marked *unverified*.

Reference used for layouts: Joelrau's x64-zt (`src/zonetool/zonetool/iw6/`
`structs.hpp`, `zone.cpp`, `assets/*.cpp`: the IW6 linker/dumper, whose write
order and `align()` calls mirror the game's loader). Struct sizes were
measured by compiling that header with `clang++ --target=x86_64-pc-windows-msvc`
(MSVC x64 layout; `-S -emit-llvm` of a `sizeof` table), then every field the
code reads was confirmed against real bytes.

## Container (`container.rs`)

```
0x00 "IWff0100" (retail, signed) | "IWffu100" (unsigned, tools)
0x08 u32 version = 0x235
0x0c u8 compress=1, u8 compressType=0 (zlib), u8 sizeOfPointer=8, u8 sizeOfLong=4
0x10 u32 fileTimeHigh, u32 fileTimeLow
0x18 u32 imageCount
0x1c imageCount x XStreamFile (24 bytes): +2 u16 imagefile index, +4 u32 (1),
     +8 u64 offset, +16 u64 offsetEnd   -> bytes of zone/imagefile<N>.pak
     u64 baseFileLen (= .ff size), u64 totalFileLen (+ streamed images)
     signed: "IWffs100" auth header (0x2000) + first hash chunk (0x2000), then
             0x2000-byte data chunks with one 0x2000 hash chunk after every 256
     unsigned: the zlib stream directly
```
The data chunks join into ONE zlib stream (`78 DA`) of the whole zone image.
`common_mp`: 1616 streamed images (imagefile59.pak ...), 271,527,715 bytes
inflated; `mp_prisonbreak`: 7592 images; `code_post_gfx_mp`: none.

## Zone image (`xfile.rs`)

```
0x00 XFile { u64 size; u64 externalSize; u64 blockSize[7]; }          72
     blocks: 0 TEMP, 1 PHYSICAL, 2 RUNTIME, 3 VIRTUAL, 4 LARGE, 5 CALLBACK, 6 SCRIPT
0x48 XAssetList { ScriptStringList { i32 count; char** strings; };
                  i32 assetCount; XAsset* assets; XGlobals* globals; }   40
VIRTUAL: count x char*; the strings (string 0 is null)
VIRTUAL: XGlobals { XGfxGlobals* } (8), XGfxGlobals (104):
         u32 depthStencilStateCount, blendStateCount, ...;
         u64 depthStencilStateBits[] (align 4), u32 blendStateBits[] (align 4),
         u32 perPrim/perObj/stable constant-buffer sizes[] (align 4 each);
         the state/buffer tables are RUNTIME (not in the file)
VIRTUAL, align 4: assetCount x XAsset { u32 type; pad; void* header; }   16
asset data, in table order
```
The file has no padding: the loader reads the stream in order and aligns
its *block* positions. To know a VIRTUAL offset (what references point to)
the reader must count every VIRTUAL allocation with the loader's alignment
(`cursor.rs`). Asset headers, inline sub-asset headers and loaded-sound PCM
are TEMP; RUNTIME takes no file bytes.

### Pointers (`ptr.rs`)
8 bytes; top 28 bits `0xFDFDFDF`, bits 32..35 a block, low 32 a value.
* `FDFDFDFF_FFFFFFFF` follow (data inline, here);
* `FDFDFDFF_FFFFFFFE` insert: data inline **and** an 8-byte VIRTUAL slot,
  aligned 8, is taken (seen on `LoadedSound.data`);
* `FDFDFDF3_xxxxxxxx` reference to VIRTUAL offset `xxxxxxxx - 1` (strings,
  shared `SoundFile`s, whole assets via `XAsset[i].header` = table + 16*i + 8);
* a raw x64 pointer into `iw6mp64_ship.exe` (e.g. `0x1411b7cb0`) appears for
  built-in defaults (a retail `OcclusionShape`'s name): no string in the zone.

### Alignments confirmed
string table and strings unaligned; XGlobals/XGfxGlobals unaligned; globals
arrays and the asset table 4; NetConstStrings string table 8 (none when the
list is null/empty); `snd_alias_t[]`, `SoundFile`, `SpeakerMap`,
`OcclusionShape` 4; insert slots 8. With these, 4197 + 235 sound lists in
`common_mp` / `mp_prisonbreak` read with a carried VIRTUAL count land
exactly on their own name anchors (0 conflicts).

## Asset types (`asset_type.rs`)
0x00 physpreset .. 0x3E particlesimanimation (63 types, x64-zt order); 0x0E
sound, 0x12 loaded_sound, 0x22 attachment, 0x23 weapon, 0x34 netconststrings.
Top-level x64 header sizes are in `AssetType::header_size()` (e.g. material
936, techset 3312, xmodel 600, weapon 296, attachment 280, gfxworld 2608,
clipmap 640).

Counts seen:

| zone | assets | notable |
|---|---|---|
| code_post_gfx_mp | 1195 | 452 techset, 415 material, 108 rawfile, 59 sound |
| common_mp | 8777 | 6296 sound, 1418 xanim, 558 fx, 187 loaded_sound, 1 weapon |
| common_core_mp | 3034 | **132 weapon, 63 attachment**, 872 material, 739 sound, 235 xanim, 90 xmodel, 11 vehicle |
| mp_prisonbreak | 1741 | clipmap, comworld, glassworld, map_ents, fxworld, gfxworld, 879 material, 501 techset, 244 sound |
| ui_mp | 2030 | 1897 material, 80 techset, 46 weapon, 3 menulist, 0 sound |

## Sounds (`sound.rs`, `scan.rs`, `pak.rs`)

```
snd_alias_list_t { char* name; snd_alias_t* head; i32 count; }      24 (TEMP)
snd_alias_t 200: 0 name, 8 subtitle, 16 secondary, 24 chain, 32 mixerGroup,
  40 SoundFile*, 48 sequence, 52 volMin, 56 volMax, 60 volModIndex,
  64 pitchMin, 68 pitchMax, 72 distMin, 76 distMax, 80 velocityMin, 84 flags,
  88 masterPriority(u8), 92 masterPct, 96 slavePct, 100 probability,
  104 lfePct, 108 centerPct, 112 startDelay, 120/128/136 SndCurve*
  (volumeFalloff, lpf, reverbSend), 144..152 envelope, 160 SpeakerMap*,
  168 wetMixOverride, 176 OcclusionShape*, 184 allowDoppler, 192 DopplerPreset*
SoundFile 56 { u8 type (1 loaded, 2 streamed, 3 primed); u8 exists; pad; union @8 }
StreamFileName 24 { u16 isLocalized; u16 fileIndex; pad;
                    fileIndex ? {u64 offset; u64 length} : {char* dir; char* name} }
StreamedSound { StreamFileName; u32 totalMsec }   PrimedSound { StreamFileName;
  LoadedSound* loadedPart; i32 dataOffset; i32 totalSize; u32 primedCrc }
LoadedSound 56 (TEMP) { name; data; seekTable; RiffAudioInfo { rate, bytes,
  samples, avgBytesPerSec, u16 channels, bits, blockAlign, u8 format };
  i32 loadedSize; i32 seekTableSize }   name VIRTUAL, data TEMP, seek VIRTUAL
SndCurve 152, DopplerPreset 32 (TEMP + name); SpeakerMap 416, OcclusionShape 32 (VIRTUAL + name)
```
Per alias the stream order is: strings, sound file (+ loaded sound), the
three curves, speaker map, occlusion shape, doppler preset.

**Names are shared.** A list's `name` is usually a reference into the
zone's `ncs_snd_<zone>` `NetConstStrings` (read exactly at the zone's head);
every alias in a list repeats the list's name pointer. `SoundFile`s are
shared too (`music_challenge_factory` and `music_opening` point at the
silent `null` loaded sound of `AB_1mc_boost`).

**Finding them without reading every asset first**: `scan.rs` looks for the
list shape after the asset table, reads each candidate strictly, and places
runs of adjacent lists in VIRTUAL from inline-named lists (anchors) or, with
no anchor, from the zone's shared sound-file references (calibration, >= 2
agreeing votes). Results: every sound list found in every zone above
(59/59, 6296/6296, 739/739, 244/244, 8/8, 1/1), 0 conflicts; unresolved:
1 list name in common_mp (a reference outside NCS and anchored runs), 9
shared sound files in common_core_mp.

**Paks**: `fileIndex N` -> `zone/soundfile<N>.pak` (verified for N = 1; the
offsets of N = 4, 7, 13, 16, 22, 59 in common_mp fit their paks). Paks are
FLAC streams (`fLaC` + STREAMINFO), 0x800-aligned, no table. Localized ->
`zone/<language>/<lang>_soundfile<N>.pak` (*unverified*: no localized
stream in the MP zones read).

### Menu music (`music.rs`)

| alias | zone | soundfile1.pak offset | length | duration | format |
|---|---|---|---|---|---|
| music_mainmenu_mp | patch_code_post_gfx_mp (same stream as base) | 0x0bbad000 | 0x97e0dc | 131.095 s | FLAC 44.1k 2ch 16 |
| music_mainmenu | code_post_gfx_mp | 0x0b149800 | 0xa63044 | 142.845 s | 44.1k |
| music_mainmenu_extinct | code_post_gfx_mp | 0x0a541000 | 0xc08127 | 118.950 s | 48k |
| music_mainmenu_squad | code_post_gfx_mp | 0x0ce2a000 | 0xa21e7f | 121.647 s | 48k |
| music_mainmenu_rorke_files | code_post_gfx_mp | 0x0c52b800 | 0x8fe588 | 116.450 s | 48k |
| music_credits_entire_scroll | code_post_gfx_mp | 0x079ea000 | 0x2b56802 | 407.420 s | 44.1k |

STREAMINFO totals match `totalMsec` within a millisecond; symphonia decodes
all six to the end.

## In the game

* The install is found at `IW4L_GHOSTS_DIR`, a `ghosts=` line in
  game_paths.cfg, `<IW4L_GAMES>/ghosts` (build-and-run.ps1 links
  `games\ghosts` -> `Documents\ghosts_game_files`, and `games\aw` ->
  `aw_game_files`), or a folder under the games root named for Ghosts.
* With an install, `ghosts:music_mainmenu_mp` joins the default menu (and
  lobby) playlist; the others are listed in `audio_manifest.csv` and can be
  added in music.cfg (`menu = ghosts:music_mainmenu_squad`, `iw6:` works too).
  Tracks stream from the pak as FLAC through `StreamedMusic`.
* Ghosts is not an `AssetNamespace` (that enum is matched across the asset
  pipeline; Ghosts feeds none of it yet). Add `Iw6` with phase 2.
* Zone discovery (`asset_transport::discover::files_under`) now drops plain
  non-`.ff` files before stat/canonicalize, so the paks are never touched;
  Ghosts' version 0x235 zones are ignored by every existing lane.
* `iw4l check-ghosts [zone]` prints a zone's counts and sound scan and checks
  every menu track (fLaC, STREAMINFO vs alias, full decode probe); report in
  `<artifacts>/ghosts-check.txt`. Release-speed: common_mp inflates and scans
  in ~2 s (debug ~12 s).

## Next (phase 2: weapons and a map)

1. **Exact asset walk.** The scan is enough for sounds only. Weapons need the
   stream read in table order with every preceding asset parsed: port the
   write() order and `align()` of x64-zt `iw6/assets/*.cpp` type by type
   (xanim, xmodel/xmodelsurfs, material, techset/shaders, image, fx,
   attachment, weapon, vehicle ...) onto `cursor.rs`, checking each against
   the scan's anchors (sound lists give exact VIRTUAL checkpoints mid-zone)
   and the asset-header references (`AssetList::asset_index_of`).
2. **Weapons**: `common_core_mp.ff` (132 weapons, 63 attachments, their
   xanims/models/materials/sounds); IW6 builds guns from a base weapon +
   attachments (`WeaponCompleteDef` 296 bytes -> `WeaponDef`). Images stream
   from `imagefile<N>.pak` via the zone's XStreamFile table; models'
   surfaces may be in separate `xmodelsurfs`; viewhands in `mp_viewhands_*`
   transient zones; bodies/heads in `mp_body_*_tr` / `mp_head_*_tr`.
3. **A map**: `mp_prisonbreak.ff` (+ `_load`, `_path`, `techsets_mp_prisonbreak.ff`,
   `patch_mp_prisonbreak.ff`): clipmap 640, comworld 40, glassworld 16,
   map_ents 256, fxworld 176, gfxworld 2608 bytes of header. Shaders are
   D3D11 DXBC (compute/hull/domain types exist), not the D3D9 SM3 our IW4
   lane translates: the render side needs a DXBC path or re-authored materials.
4. Add `AssetNamespace::Iw6` when Ghosts assets enter the shared pipeline.
