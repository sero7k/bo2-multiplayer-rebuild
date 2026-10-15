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
