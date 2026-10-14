//! The asset walk: the assets' data in `XAsset` table order, each type's
//! loader reading its header (TEMP) and then what its pointers say follows.
//!
//! Only the types in `simple` are known so far; the walk stops at the
//! first inline asset of any other type and says where (`Stop`), so the
//! next walker to add is the one it stopped at. References in the table
//! (assets loaded earlier) take no stream bytes and are skipped.

pub mod simple;

use crate::asset_type::AssetType;
use crate::cursor::{Cursor, ReadError};
use crate::zone::Zone;

/// One asset the walk stepped over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walked {
    pub index: usize,
    pub ty: AssetType,
    /// File offsets of the asset's data: `start..end`.
    pub start: usize,
    pub end: usize,
    /// The asset's name, when it followed in the stream.
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// Every asset was walked; the stream ended at `end`.
    Done { end: usize },
    /// The asset at `index` has a type the walk cannot read yet.
    Unknown { index: usize, raw_type: u32, at: usize },
    /// The asset at `index` did not read as its type.
    Failed { index: usize, ty: AssetType, at: usize, error: ReadError },
}

#[derive(Clone, Debug)]
pub struct Walk {
    pub walked: Vec<Walked>,
    pub stop: Stop,
}

/// Reads one asset at the cursor; its name if it had one inline.
type AssetFn = fn(&mut Cursor) -> Result<Option<String>, ReadError>;

#[derive(Clone, Copy)]
enum Walker {
    Full(AssetFn),
    /// Header and name only (`simple::placeholder`), the name at this offset.
    Placeholder(usize),
}

/// The walker of `ty`; `None` when it has none yet.
fn walker(ty: AssetType) -> Option<Walker> {
    Some(match ty {
        AssetType::Localize => Walker::Full(simple::localize),
        AssetType::RawFile => Walker::Full(simple::rawfile),
        AssetType::ScriptParseTree => Walker::Full(simple::script_parse_tree),
        AssetType::Ttf => Walker::Full(simple::ttf),
        AssetType::Image => Walker::Placeholder(simple::IMAGE_NAME_AT),
        AssetType::TechniqueSet
        | AssetType::Material
        | AssetType::XModel
        | AssetType::XModelMesh
        | AssetType::XAnimParts
        | AssetType::PhysPreset
        | AssetType::PhysConstraints => Walker::Placeholder(0),
        _ => return None,
    })
}

/// Can the walk read assets of `ty`?
pub fn is_walkable(ty: AssetType) -> bool {
    walker(ty).is_some()
}

/// Walks the zone's assets in table order until the end or the first
/// asset it cannot read.
pub fn walk(zone: &Zone) -> Walk {
    let mut c = Cursor::new(&zone.image, zone.assets.body_offset);
    let mut walked = Vec::new();
    for (index, a) in zone.assets.assets.iter().enumerate() {
        if !a.is_inline() {
            continue;
        }
        let at = c.pos();
        let Some((ty, f)) = a.asset_type().and_then(|t| walker(t).map(|f| (t, f))) else {
            return Walk { walked, stop: Stop::Unknown { index, raw_type: a.raw_type, at } };
        };
        let read = match f {
            Walker::Full(f) => f(&mut c),
            Walker::Placeholder(name_at) => simple::placeholder(&mut c, ty, name_at),
        };
        match read {
            Ok(name) => walked.push(Walked { index, ty, start: at, end: c.pos(), name }),
            Err(error) => return Walk { walked, stop: Stop::Failed { index, ty, at, error } },
        }
    }
    Walk { walked, stop: Stop::Done { end: c.pos() } }
}
