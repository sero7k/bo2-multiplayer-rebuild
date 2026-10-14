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

pub mod ai;
pub mod anim;
pub mod bundle;
pub mod capture;
pub mod fx;
pub mod image;
pub mod material;
pub mod model;
pub mod placeholder;
pub mod simple;
pub mod sound;
pub mod tables;
pub mod weapon;
pub mod weapon_fields;

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
    /// What the walk kept (weapons).
    pub capture: capture::Capture,
    /// What `Options::resync` skipped.
    pub skipped: Vec<Skip>,
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
        T::PhysPreset => tables::phys_preset,
        T::SurfaceSoundDef => tables::surface_sound_def,
        T::FootstepTable => tables::footstep_table,
        T::SurfaceFxTable => tables::surface_fx_table,
        T::ImpactFx => tables::impact_fx,
        T::ImpactSound => tables::impact_sound,
        T::EntityFxImpacts => tables::entity_fx_impacts,
        T::EntitySoundImpacts => tables::entity_sound_impacts,
        T::Laser => tables::laser,
        T::Beam => tables::beam,
        T::Tracer => tables::tracer,
        T::Shellshock => tables::shellshock,
        T::SharedWeaponSounds => tables::shared_weapon_sounds,
        T::VehicleFxDef => tables::vehicle_fx_def,
        T::VehicleSoundDef => tables::vehicle_sound_def,
        T::XModelAlias => tables::xmodel_alias,
        T::Character => tables::character,
        T::AiType => tables::ai_type,
        T::DestructibleDef => tables::destructible_def,
        T::PhysConstraints => model::phys_constraints,
        T::LightDescription => tables::light_description,
        T::Objective => tables::objective,
        T::Fx => fx::fx,
        T::ScriptBundle => bundle::script_bundle,
        T::BehaviorStateMachine => ai::behavior_state_machine,
        T::BehaviorTree => ai::behavior_tree,
        T::AnimStateMachine => ai::anim_state_machine,
        T::AnimSelectorTableSet => ai::anim_selector_table_set,
        T::AnimMappingTable => ai::anim_mapping_table,
        T::Rumble => bundle::rumble,
        T::WeaponCamo => bundle::weapon_camo,
        T::AttachmentCosmeticVariant => bundle::attachment_cosmetic_variant,
        T::VehicleDef => tables::vehicle,
        T::TagFx => bundle::tag_fx,
        T::NewLensFlareDef => placeholder::lens_flare,
        T::AimTable => placeholder::aim_table,
        T::XCam => placeholder::xcam,
        T::CustomizationTable => placeholder::customization_table,
        T::MapTable => placeholder::map_table,
        T::Medal => tables::medal,
        T::CgMedia => tables::cg_media,
        T::LocDmgTable => tables::loc_dmg_table,
        T::BulletPenetration => tables::bullet_penetration,
        T::MedalTable => bundle::medal_table,
        T::ObjectiveList => bundle::objective_list,
        T::BgCache => bundle::bg_cache,
        T::Leaderboard => bundle::leaderboard,
        T::CustomizationTableColor => placeholder::customization_table_color,
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
                let name = self.load_asset(ty)?;
                if let Some(n) = &name {
                    self.name_slot(field, ty, n);
                }
                Ok(name)
            }
            p @ Ptr::Ref { .. } => {
                self.note_ref(field, p);
                Ok(None)
            }
            Ptr::Other(_) => Err(ReadError::Invalid("asset pointer", field)),
        }
    }

    /// `asset`, and the name of the asset the field names: the one loaded
    /// here, or the one an earlier load left at the slot a reference names.
    pub fn asset_named(&mut self, ty: AssetType, field: usize) -> Result<Option<String>> {
        let p = self.ptr_at(field)?;
        let loaded = self.asset(ty, field)?;
        Ok(match p {
            Ptr::Ref { .. } => {
                let v = p.virtual_offset();
                Some(match v.and_then(|v| self.named_at(v)) {
                    Some((_, n)) => n.clone(),
                    None => format!("<ref {:#x}>", v.unwrap_or(0)),
                })
            }
            _ => loaded,
        })
    }

    /// `string`, but a reference whose text is not found reads `<ref ...>`.
    pub fn string_named(&mut self, field: usize) -> Result<Option<String>> {
        let p = self.ptr_at(field)?;
        let s = self.string(field)?;
        Ok(match (s, p) {
            (None, Ptr::Ref { offset, .. }) => Some(format!("<ref {offset:#x}>")),
            (s, _) => s,
        })
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

/// How `walk_with` goes on past an asset it cannot read.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// Skip assets without a walker (or that fail) by finding the next
    /// asset the walk can read: its header's name pointer (`-1`) with a
    /// printable name right after the header. File positions stay right
    /// once the next asset is found; VIRTUAL offsets after a skip do not
    /// (the skipped assets' allocations are not counted), so references
    /// read after it are counted but no longer checked meaningfully.
    pub resync: bool,
}

/// Assets `walk_with` skipped: (first index, the index it resumed at, the
/// first skipped asset's type, file range skipped, why).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skip {
    pub index: usize,
    pub resumed_at: usize,
    pub raw_type: u32,
    pub start: usize,
    pub end: usize,
    pub why: String,
}

/// Types whose layout is not read yet but whose assets the walk steps over
/// by finding the next asset (always, not only with `Options::resync`):
/// lens flares (`klf`: elements of 0x1f0 with chained sub-structs and
/// curve keys, PHYSICAL constant buffers; see the module docs).
pub fn is_opaque(ty: AssetType) -> bool {
    matches!(ty, AssetType::NewLensFlareDef)
}

/// File offset of the name pointer in an asset's header.
fn name_field(ty: AssetType) -> usize {
    match ty {
        AssetType::Image => image::IMAGE_NAME_AT,
        _ => 0,
    }
}

/// A plausible asset name at `at`: 3..=255 bytes of the characters asset
/// names use (letters, digits, `_/\\.,-#$@:` and spaces) and a NUL.
fn plausible_name(data: &[u8], at: usize) -> bool {
    let w = data.get(at..data.len().min(at + 256)).unwrap_or(&[]);
    let ok = |b: u8| b.is_ascii_alphanumeric() || b"_/\\.,-#$@: ".contains(&b);
    match w.iter().position(|&b| b == 0) {
        Some(n) if n >= 3 => w[..n].iter().all(|&b| ok(b)) && w[..n].iter().any(|b| b.is_ascii_alphabetic()),
        _ => false,
    }
}

/// Do the assets from `index` read from file offset `at` (on a scratch
/// stream)? The first inline one must, the next two until one fails.
fn trial(zone: &Zone, index: usize, at: usize) -> bool {
    let mut s = Stream::new(&zone.image);
    s.seek(at);
    let mut n = 0;
    for a in zone.assets.assets.iter().skip(index) {
        if !a.is_inline() {
            continue;
        }
        let Some(ty) = a.asset_type().filter(|t| is_walkable(*t)) else { return n > 0 };
        if s.load_asset(ty).is_err() {
            // The first must read; a later one may be the next asset the
            // walk cannot read yet.
            return n > 0;
        }
        n += 1;
        if n == 3 {
            return true;
        }
    }
    true
}

/// The next asset after `index` the walk can read, and where its header
/// starts at or after `from`: the first of the next few readable inline
/// assets whose header is found within 64 MiB.
fn resync_point(zone: &Zone, index: usize, from: usize) -> Option<(usize, usize)> {
    const LIMIT: usize = 64 << 20;
    let data = &zone.image;
    let end = data.len().min(from + LIMIT);
    let mut tried = 0;
    for (j, a) in zone.assets.assets.iter().enumerate().skip(index + 1) {
        if !a.is_inline() {
            continue;
        }
        let Some(ty) = a.asset_type().filter(|t| is_walkable(*t) && t.header_size() > 0) else { continue };
        tried += 1;
        if tried > 1 {
            return None;
        }
        let (size, nf) = (ty.header_size(), name_field(ty));
        let mut p = from;
        while p + size < end {
            if data[p + nf..p + nf + 8] == [0xff; 8] && plausible_name(data, p + size) && trial(zone, j, p) {
                return Some((j, p));
            }
            p += 1;
        }
    }
    None
}

/// Walks the zone's assets in table order until the end or the first
/// asset it cannot read.
pub fn walk(zone: &Zone) -> Walk {
    walk_with(zone, Options::default())
}

/// `walk`, with `options`.
pub fn walk_with(zone: &Zone, options: Options) -> Walk {
    let mut s = Stream::new(&zone.image);
    let mut walked = Vec::new();
    let mut skipped: Vec<Skip> = Vec::new();
    let finish = |s: &Stream, walked, skipped, stop| {
        if std::env::var_os("T7_DRIFT").is_some() {
            s.print_drift();
        }
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
            skipped,
            stop,
            refs: s.refs,
            first_unresolved: s.first_unresolved,
            virtual_end: s.offset(VIRTUAL),
            physical_end: s.offset(PHYSICAL),
            streamer_end: s.offset(STREAMER),
            capture: s.capture.clone(),
        }
    };
    let table = match asset_list(&mut s, zone) {
        Ok(t) => t,
        Err(error) => {
            let stop = Stop::Failed { index: 0, ty: AssetType::Localize, at: 0, error };
            return finish(&s, walked, skipped, stop);
        }
    };
    let assets = &zone.assets.assets;
    let mut index = 0;
    while index < assets.len() {
        let a = &assets[index];
        let field = table + XASSET_LEN * index + 8;
        if !a.is_inline() {
            s.note_ref(field, a.header);
            index += 1;
            continue;
        }
        let at = s.pos();
        let result = match a.asset_type().filter(|t| is_walkable(*t)) {
            None => Err(Stop::Unknown { index, raw_type: a.raw_type, at }),
            Some(ty) => {
                if a.header == Ptr::Insert {
                    s.insert_slot();
                }
                match s.load_asset(ty) {
                    Ok(name) => {
                        if let Some(n) = &name {
                            s.name_slot(field, ty, n);
                        }
                        walked.push(Walked { index, ty, start: at, end: s.pos(), name });
                        Ok(())
                    }
                    Err(error) => Err(Stop::Failed { index, ty, at, error }),
                }
            }
        };
        match result {
            Ok(()) => index += 1,
            Err(stop) => {
                let opaque = a.asset_type().is_some_and(is_opaque);
                let resume = if options.resync || opaque { resync_point(zone, index, at + 8) } else { None };
                let Some((j, p)) = resume else { return finish(&s, walked, skipped, stop) };
                let why = match &stop {
                    Stop::Failed { error, .. } => error.to_string(),
                    _ => "no walker".to_string(),
                };
                skipped.push(Skip { index, resumed_at: j, raw_type: a.raw_type, start: at, end: p, why });
                if opaque {
                    // Count what an opaque asset (one, then the resync
                    // point) loads as VIRTUAL: its bytes but the header.
                    let header = a.asset_type().map_or(0, |t| t.header_size());
                    let n = (p - at).saturating_sub(header);
                    s.push(VIRTUAL);
                    s.align(7);
                    s.skip(n as u64);
                    s.pop();
                }
                s.seek(p);
                index = j;
            }
        }
    }
    let stop = Stop::Done { end: s.pos() };
    finish(&s, walked, skipped, stop)
}
