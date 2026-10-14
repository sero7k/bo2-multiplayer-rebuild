//! Sound banks (`SndBank`, 0x220b0 bytes), read on retail `en_mp_patch`,
//! `en_mp_common` and `core_post_gfx` (x64-zt has no T7 layout for them).
//!
//! ```text
//! SndBank         0x00 char* name ("mp_patch.en"); 0x08 char* zone ("mp_patch");
//!                 0x10 char* gameLanguage ("english"); 0x18 char* soundLanguage ("en")
//!                 0x20 u64 aliasCount;   0x28 SndAliasList* alias;  0x30 SndIndexEntry* aliasIndex
//!                 0x38 u64 radverbCount; 0x40 SndRadverb* radverbs
//!                 0x48 u64 duckCount;    0x50 SndDuck* ducks
//!                 0x58 u64 snapshotCount (?); 0x60 0x184-byte records*
//!                 0x68.. inline (zero in every bank read)
//! SndAliasList    40: 0x00 char* name; 0x08 u32 id; 0x10 SndAlias* head; 0x18 i32 count;
//!                 0x20 f32 (10000.0); 0x24 u32
//! SndAlias        0xd8: 0x00 char* name (a reference to its list's name); 0x08 u32 id;
//!                 0x10 char* subtitle; 0x18 char* secondaryName; 0x28 char* stopAliasName;
//!                 0x38 u32 assetId; 0x40 char* assetFileName ("en\vox\...\x.SN100.pc.snd");
//!                 the rest flags, volumes, distances
//! SndIndexEntry   4: { u16 value; u16 next }
//! SndRadverb      0xa0: char name[64]; u32 id; floats
//! SndDuck         0x88: char name[64]; u32 id; floats; 0x60 f32* attenuation; 0x68 f32* filter
//!                 (64 floats each)
//! 0x58/0x60 array 0x184: char name[64] ("cinematic_room"), ids, char[64], floats; no pointers
//! ```
//!
//! Stream order: the four names, the alias lists (40 each, aligned 8),
//! then per list its name and alias array (aligned 8), then per alias its
//! strings in field order; the index (aligned 4), radverbs, ducks (then
//! each duck's two curves) and the 0x184 records. The sound data is not in
//! the zone: `assetFileName` names it in the `.sabl`/`.sabs` banks.

use crate::cursor::Result;
use crate::stream::Stream;

const ALIAS_LIST: usize = 40;
const ALIAS: usize = 0xd8;
const ALIAS_STRINGS: [usize; 5] = [0x00, 0x10, 0x18, 0x28, 0x40];
const RADVERB: usize = 0xa0;
const DUCK: usize = 0x88;
const DUCK_CURVE: usize = 64 * 4;
const SNAPSHOT: usize = 0x184;

pub fn sound_bank(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::Sound.header_size(), &[0, 8, 0x10, 0x18, 0x28, 0x30, 0x40, 0x50, 0x60])?;
    let name = s.string(h)?;
    for f in [8, 0x10, 0x18] {
        s.string(h + f)?;
    }
    let lists = s.u32_at(h + 0x20)? as usize;
    if let Some(t) = s.records(h + 0x28, lists, ALIAS_LIST, 7)? {
        for i in 0..lists {
            let l = t + ALIAS_LIST * i;
            s.no_loads(l, ALIAS_LIST, &[0, 0x10])?;
            s.string(l)?;
            let n = s.i32_at(l + 0x18)?.max(0) as usize;
            if let Some(a) = s.records(l + 0x10, n, ALIAS, 7)? {
                for j in 0..n {
                    let e = a + ALIAS * j;
                    s.no_loads(e, ALIAS, &ALIAS_STRINGS)?;
                    for f in ALIAS_STRINGS {
                        s.string(e + f)?;
                    }
                }
            }
        }
    }
    s.records(h + 0x30, lists, 4, 3)?;
    let radverbs = s.u32_at(h + 0x38)? as usize;
    s.records(h + 0x40, radverbs, RADVERB, 3)?;
    let ducks = s.u32_at(h + 0x48)? as usize;
    if let Some(t) = s.records(h + 0x50, ducks, DUCK, 7)? {
        for i in 0..ducks {
            let d = t + DUCK * i;
            s.no_loads(d, DUCK, &[0x60, 0x68])?;
            s.array(d + 0x60, DUCK_CURVE, 3)?;
            s.array(d + 0x68, DUCK_CURVE, 3)?;
        }
    }
    let snapshots = s.u32_at(h + 0x58)? as usize;
    s.records(h + 0x60, snapshots, SNAPSHOT, 3)?;
    Ok(name)
}
