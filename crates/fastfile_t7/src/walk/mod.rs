//! The asset walk: the zone read the way the loader reads it (`stream`),
//! the `XAssetList` first, then the assets' data in `XAsset` table order,
//! each asset's top-level struct in TEMP and what its pointers load in
//! VIRTUAL, so that every VIRTUAL offset is counted and the references
//! (`block << 60 | offset + 1`) can be checked against what was read.
//!
//! Types without a walker stop the walk at their first inline asset
//! (`Stop`), so the next walker to add is the one it stopped at.
//! References in the table (assets loaded earlier) take no stream bytes.
//!
//! Walkers: `simple` (localize, rawfile, scriptparsetree, ttf, keyvaluepairs,
//! fonticon, stringtable, texturecombo, bitfield), `sound` (SndBank),
//! `model` (xmodelmesh, xmodel; physpreset/physconstraints as placeholders),
//! `material` (material, techset), `image`, `anim` (xanim), `placeholder`
//! ("," stand-ins of types without a real walker yet). `weapon` has none.
//!
//! Debug hooks (environment): `T7_TRACE=<hex from>,<hex to>` prints every
//! allocation in that file range, `T7_REFLOG=<hex from>,<hex to>` every
//! reference read there, `T7_MISSES=1` the first unresolved references,
//! `T7_VIRT=<hex file offset>,v<hex VIRTUAL offset>,...` maps offsets both
//! ways at the end of the walk.

pub mod anim;
pub mod image;
pub mod material;
pub mod model;
pub mod placeholder;
pub mod simple;
pub mod sound;
pub mod weapon;

use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result, text};
use crate::ptr::Ptr;
use crate::stream::{PHYSICAL, RefStats, STREAMER, Stream, TEMP, VIRTUAL};
use crate::xfile::{XASSET_LEN, XASSET_LIST_LEN};
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
    /// How the references read on the way landed.
    pub refs: RefStats,
    /// The first reference that landed mid-data: (file offset of the
    /// pointer, VIRTUAL offset it names).
    pub first_unresolved: Option<(usize, u64)>,
    /// VIRTUAL's position where the walk ended (at `Done`, the zone
    /// header's VIRTUAL block size).
    pub virtual_end: u64,
    /// PHYSICAL's position where the walk ended (the header's PHYSICAL size at `Done`).
    pub physical_end: u64,
    /// STREAMER's position (no file bytes: streamed images' pixels).
    pub streamer_end: u64,
}

/// Reads one asset whose top-level struct is at file offset `h` (TEMP),
/// the stream in VIRTUAL; its name if it had one.
pub type AssetFn = fn(&mut Stream, usize) -> Result<Option<String>>;

/// The walker of `ty`; `None` when it has none yet.
fn walker(ty: AssetType) -> Option<AssetFn> {
    use AssetType as T;
    Some(match ty {
        T::Localize => simple::localize,
        T::RawFile => simple::rawfile,
        T::ScriptParseTree => simple::script_parse_tree,
        T::Ttf => simple::ttf,
        T::KeyValuePairs => simple::key_value_pairs,
        T::FontIcon => simple::font_icon,
        T::StringTable => simple::string_table,
        T::TextureCombo => simple::texture_combo,
        T::Bitfield => simple::bitfield,
        T::Sound => sound::sound_bank,
        T::XModelMesh => model::xmodel_mesh,
        T::XModel => model::xmodel,
        T::Material => material::material,
        T::TechniqueSet => material::technique_set,
        T::Image => image::image,
        T::XAnimParts => anim::xanim,
        T::PhysPreset => model::phys_preset,
        T::PhysConstraints => model::phys_constraints,
        T::LightDescription => placeholder::light_description,
        T::Fx => placeholder::fx,
        T::VehicleDef => placeholder::vehicle,
        T::TagFx => placeholder::tag_fx,
        _ => return weapon::walker(ty),
    })
}

/// Can the walk read assets of `ty`?
pub fn is_walkable(ty: AssetType) -> bool {
    walker(ty).is_some()
}

/// The error for an inline asset of a type the walk cannot read yet.
fn no_walker(ty: AssetType, at: usize) -> ReadError {
    ReadError::NoWalker(ty.name(), at)
}

impl<'a> Stream<'a> {
    /// The string a pointer field names: read here (current block) when it
    /// follows, the earlier string when it is a reference.
    pub fn string(&mut self, field: usize) -> Result<Option<String>> {
        match self.ptr_at(field)? {
            Ptr::Null => Ok(None),
            p @ (Ptr::Follow | Ptr::Insert) => {
                if p == Ptr::Insert {
                    self.insert_slot();
                }
                Ok(Some(text(self.cstr()?.0)))
            }
            p @ Ptr::Ref { .. } => {
                self.note_ref(field, p);
                Ok(p.virtual_offset().and_then(|v| self.string_at(v)).map(text))
            }
            Ptr::Other(_) => Err(ReadError::Invalid("string pointer", field)),
        }
    }

    /// Data a pointer field loads here: `len` bytes aligned to `mask + 1`
    /// in the current block; its file offset when it followed (`None` when
    /// null or a reference).
    pub fn array(&mut self, field: usize, len: usize, mask: u64) -> Result<Option<usize>> {
        match self.ptr_at(field)? {
            Ptr::Null => Ok(None),
            p @ (Ptr::Follow | Ptr::Insert) => {
                if p == Ptr::Insert {
                    self.insert_slot();
                }
                self.alloc(len, mask)
            }
            p @ Ptr::Ref { .. } => {
                self.note_ref(field, p);
                Ok(None)
            }
            Ptr::Other(_) => Err(ReadError::Invalid("data pointer", field)),
        }
    }

    /// `array` of `count` records of `size` bytes.
    pub fn records(&mut self, field: usize, count: usize, size: usize, mask: u64) -> Result<Option<usize>> {
        let len = count.checked_mul(size).ok_or(ReadError::Invalid("record count", field))?;
        self.array(field, len, mask)
    }

    /// A pointer to an asset of type `ty`: loaded right here when it
    /// follows (its struct in TEMP, its data in VIRTUAL), else a reference
    /// to one loaded earlier. The asset's name when it was loaded here.
    pub fn asset(&mut self, ty: AssetType, field: usize) -> Result<Option<String>> {
        match self.ptr_at(field)? {
            Ptr::Null => Ok(None),
            p @ (Ptr::Follow | Ptr::Insert) => {
                if p == Ptr::Insert {
                    self.insert_slot();
                }
                self.mark_slot(field);
                self.load_asset(ty)
            }
            p @ Ptr::Ref { .. } => {
                self.note_ref(field, p);
                Ok(None)
            }
            Ptr::Other(_) => Err(ReadError::Invalid("asset pointer", field)),
        }
    }

    /// An asset whose data starts here: its struct in TEMP, then its data.
    pub fn load_asset(&mut self, ty: AssetType) -> Result<Option<String>> {
        let f = walker(ty).ok_or_else(|| no_walker(ty, self.pos()))?;
        let size = ty.header_size();
        if size == 0 {
            return Err(no_walker(ty, self.pos()));
        }
        self.push(TEMP);
        let h = self.load(size, 7)?;
        self.push(VIRTUAL);
        let r = f(self, h);
        self.pop();
        self.pop();
        r
    }

    /// Fails when any 8-byte field of `h..h + len` (but `skip`) would load
    /// data: the fields a walker does not know to be pointers.
    pub fn no_loads(&self, h: usize, len: usize, skip: &[usize]) -> Result<()> {
        for off in (0..len / 8 * 8).step_by(8) {
            if !skip.contains(&off) && self.ptr_at(h + off)?.is_inline() {
                return Err(ReadError::Invalid("pointer field the walker does not know", h + off));
            }
        }
        Ok(())
    }
}

/// The `XAssetList` through the stream: the struct in TEMP, the script
/// strings, depends and asset table in VIRTUAL. The asset table's file
/// offset.
fn asset_list(s: &mut Stream, zone: &Zone) -> Result<usize> {
    s.push(TEMP);
    let l = s.load(XASSET_LIST_LEN, 7)?;
    s.push(VIRTUAL);
    for (count, ptr) in [(0, 8), (16, 24)] {
        let n = s.i32_at(l + count)?.max(0) as usize;
        if let Some(t) = s.records(l + ptr, n, 8, 7)? {
            for i in 0..n {
                s.string(t + 8 * i)?;
            }
        }
    }
    let n = zone.assets.assets.len();
    let table = s.records(l + 40, n, XASSET_LEN, 7)?.unwrap_or(zone.assets.table_offset);
    for i in 0..n {
        s.mark_slot(table + XASSET_LEN * i + 8);
    }
    s.pop();
    s.pop();
    if s.pos() != zone.assets.body_offset {
        return Err(ReadError::Invalid("asset list end", s.pos()));
    }
    Ok(table)
}

/// Walks the zone's assets in table order until the end or the first
/// asset it cannot read.
pub fn walk(zone: &Zone) -> Walk {
    let mut s = Stream::new(&zone.image);
    let mut walked = Vec::new();
    let finish = |s: &Stream, walked, stop| {
        // Debug: `T7_VIRT=<hex file offset>,...` prints their VIRTUAL offsets.
        if let Ok(list) = std::env::var("T7_VIRT") {
            for a in list.split(',') {
                if let Some(v) = a.strip_prefix('v').and_then(|v| u64::from_str_radix(v, 16).ok()) {
                    let at = s.vmap.file_pos(v);
                    let text = s.string_at(v).map(text);
                    eprintln!("VIRT {v:#x} -> file {at:x?} {text:?}");
                } else if let Ok(at) = usize::from_str_radix(a.trim_start_matches("0x"), 16) {
                    eprintln!("VIRT file {at:#x} -> {:x?}", s.vmap.virt_of(at));
                }
            }
        }
        Walk {
        walked,
        stop,
        refs: s.refs,
        first_unresolved: s.first_unresolved,
        virtual_end: s.offset(VIRTUAL),
        physical_end: s.offset(PHYSICAL),
        streamer_end: s.offset(STREAMER),
        }
    };
    let table = match asset_list(&mut s, zone) {
        Ok(t) => t,
        Err(error) => {
            let stop = Stop::Failed { index: 0, ty: AssetType::Localize, at: 0, error };
            return finish(&s, walked, stop);
        }
    };
    for (index, a) in zone.assets.assets.iter().enumerate() {
        let field = table + XASSET_LEN * index + 8;
        if !a.is_inline() {
            s.note_ref(field, a.header);
            continue;
        }
        let at = s.pos();
        let Some(ty) = a.asset_type().filter(|t| is_walkable(*t)) else {
            let stop = Stop::Unknown { index, raw_type: a.raw_type, at };
            return finish(&s, walked, stop);
        };
        if a.header == Ptr::Insert {
            s.insert_slot();
        }
        match s.load_asset(ty) {
            Ok(name) => walked.push(Walked { index, ty, start: at, end: s.pos(), name }),
            Err(error) => {
                let stop = Stop::Failed { index, ty, at, error };
                return finish(&s, walked, stop);
            }
        }
    }
    let stop = Stop::Done { end: s.pos() };
    finish(&s, walked, stop)
}
