//! `NetConstStrings` (type 0x34): the zone's lists of names the network code
//! refers to by index (`ncs_snd_<zone>` the sound aliases, `ncs_snl_<zone>`
//! the looping ones, `ncs_mdl_<zone>` the models, ...). Retail zones put them
//! first, and the linker shares their strings: a sound alias list's name is
//! usually a reference into `ncs_snd_*` rather than its own copy. Reading
//! them exactly gives the VIRTUAL offset of every one of those strings.
//!
//! ```text
//! NetConstStrings { char* name; i32 stringType; i32 sourceType; u32 entryCount; char** stringList; }  32 bytes (TEMP)
//! (VIRTUAL) name; stringList aligned 8: entryCount x char*, then the strings that follow
//! ```

use std::collections::HashMap;

use crate::asset_type::AssetType;
use crate::cursor::{Cursor, Result, text};
use crate::xfile::AssetList;

pub const NET_CONST_STRINGS_LEN: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetConstStrings {
    pub name: String,
    pub string_type: i32,
    pub source_type: i32,
    pub entries: Vec<Option<String>>,
}

/// Every string read, by VIRTUAL offset.
pub type StringPool = HashMap<u64, String>;

/// Reads one `NetConstStrings` at the cursor (its header first).
pub fn read_net_const_strings(c: &mut Cursor, pool: &mut StringPool) -> Result<NetConstStrings> {
    let h = c.other_block(NET_CONST_STRINGS_LEN)?;
    let name_ptr = c.ptr_at(h)?;
    let string_type = c.u32_at(h + 8)? as i32;
    let source_type = c.u32_at(h + 12)? as i32;
    let count = c.u32_at(h + 16)? as usize;
    let list_ptr = c.ptr_at(h + 24)?;
    let mut name = String::new();
    if name_ptr.is_inline() {
        let (s, _, v) = c.cstr()?;
        name = text(s);
        if let Some(v) = v {
            pool.insert(v, name.clone());
        }
    }
    let mut entries = Vec::new();
    if list_ptr.is_inline() {
        let (table, _) = c.virt_struct(8 * count, 8)?;
        let ptrs = (0..count)
            .map(|i| c.ptr_at(table + 8 * i))
            .collect::<Result<Vec<_>>>()?;
        for p in ptrs {
            if p.is_inline() {
                let (s, _, v) = c.cstr()?;
                let s = text(s);
                if let Some(v) = v {
                    pool.insert(v, s.clone());
                }
                entries.push(Some(s));
            } else {
                entries.push(None);
            }
        }
    }
    Ok(NetConstStrings {
        name,
        string_type,
        source_type,
        entries,
    })
}

/// The `NetConstStrings` assets at the head of the zone (as long as the
/// table lists them first), with the cursor left after them.
pub fn read_head<'a>(
    zone: &'a [u8],
    list: &AssetList,
    pool: &mut StringPool,
) -> (Vec<NetConstStrings>, Cursor<'a>) {
    let mut c = Cursor::new(zone, list.body_offset, Some(list.body_virtual));
    let mut out = Vec::new();
    for a in &list.assets {
        if a.asset_type() != Some(AssetType::NetConstStrings) || !a.is_inline() {
            break;
        }
        let mut next = c.clone();
        match read_net_const_strings(&mut next, pool) {
            Ok(ncs) => {
                out.push(ncs);
                c = next;
            }
            Err(_) => break,
        }
    }
    (out, c)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ptr::FOLLOW;

    pub(crate) fn ncs_bytes(name: &str, entries: &[&str]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&FOLLOW.to_le_bytes());
        b.extend_from_slice(&6u32.to_le_bytes());
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(&(entries.len() as u64).to_le_bytes());
        b.extend_from_slice(&(if entries.is_empty() { 0 } else { FOLLOW }).to_le_bytes());
        b.extend_from_slice(name.as_bytes());
        b.push(0);
        for _ in entries {
            b.extend_from_slice(&FOLLOW.to_le_bytes());
        }
        for e in entries {
            b.extend_from_slice(e.as_bytes());
            b.push(0);
        }
        b
    }

    #[test]
    fn reads_and_pools_strings() {
        let mut data = ncs_bytes("ncs_lui_level", &[]);
        data.extend(ncs_bytes("ncs_snd_x", &["music_a", "ui_b"]));
        let mut pool = StringPool::new();
        let mut c = Cursor::new(&data, 0, Some(0x101));
        let a = read_net_const_strings(&mut c, &mut pool).unwrap();
        assert!(a.entries.is_empty());
        // An empty list takes no alignment: the next name follows at once.
        assert_eq!(c.virt(), Some(0x101 + 14));
        let b = read_net_const_strings(&mut c, &mut pool).unwrap();
        assert_eq!(b.entries, vec![Some("music_a".into()), Some("ui_b".into())]);
        assert_eq!(c.pos(), data.len());
        // name at 0x10f (10 bytes) -> 0x119, table aligned to 0x120 (16 bytes).
        assert_eq!(pool.get(&0x10f).map(String::as_str), Some("ncs_snd_x"));
        assert_eq!(pool.get(&0x130).map(String::as_str), Some("music_a"));
        assert_eq!(pool.get(&0x138).map(String::as_str), Some("ui_b"));
    }
}
