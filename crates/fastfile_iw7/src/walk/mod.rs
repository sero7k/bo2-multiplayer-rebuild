//! The exact zone walk: every asset read in table order, the way the
//! game's loader reads it, block by block (`stream.rs`), so that every
//! VIRTUAL offset is known and references resolve.
//!
//! The order and alignment of each asset's data follow the x64 IW7 zone
//! tools' writers (x64-zt `zonetool/iw7/assets/*.cpp`), which have to match
//! the game's loader for their zones to load. Where the retail data has
//! something the tools never write (sub-assets stored inline where the tools
//! store a reference; a few fields the tools null), the loader's general
//! rule applies: a pointer that "follows" (or "inserts") loads its data right
//! there, a sub-asset's header in TEMP, its contents in VIRTUAL.
//!
//! What is read is kept in a `Capture`: names of every asset, and the
//! models, materials, images, animations and weapons phase 2 uses.

mod anim;
mod anim_class;
mod behavior_tree;
mod capture;
mod material;
mod misc;
mod model;
mod physics;
mod playeranim;
mod scriptable;
mod sound_bank;
mod spaceship;
mod suit;
mod vehicle;
mod vfx;
mod weapon;
mod world;

use std::collections::HashMap;

pub use capture::*;

use crate::asset_type::AssetType;
use crate::container::StreamFile;
use crate::cursor::{ReadError, Result};
use crate::ptr::Ptr;
use crate::stream::{Stream, TEMP, VIRTUAL};
use crate::xfile::{AssetList, BLOCK_COUNT, XASSET_LEN};

/// One contiguous run of VIRTUAL data read from the file.
#[derive(Clone, Copy, Debug)]
struct Segment {
    virt: u64,
    pos: usize,
    len: u64,
}

/// VIRTUAL offsets of everything read, for resolving references.
#[derive(Default, Debug)]
pub struct VirtualMap {
    segs: Vec<Segment>,
}

impl VirtualMap {
    fn add(&mut self, virt: u64, pos: usize, len: u64) {
        if len == 0 {
            return;
        }
        if let Some(last) = self.segs.last_mut()
            && last.virt + last.len == virt
            && last.pos as u64 + last.len == pos as u64
        {
            last.len += len;
            return;
        }
        self.segs.push(Segment { virt, pos, len });
    }

    /// The file offset of VIRTUAL offset `virt`, if it was read from the file.
    pub fn file_pos(&self, virt: u64) -> Option<usize> {
        let i = self
            .segs
            .partition_point(|s| s.virt <= virt)
            .checked_sub(1)?;
        let s = self.segs[i];
        (virt < s.virt + s.len).then(|| s.pos + (virt - s.virt) as usize)
    }

    /// The VIRTUAL offset of file offset `pos`, if VIRTUAL data was read there.
    pub fn virt_of(&self, pos: usize) -> Option<u64> {
        let i = self.segs.partition_point(|s| s.pos <= pos).checked_sub(1)?;
        let s = self.segs[i];
        ((pos - s.pos) < s.len as usize).then(|| s.virt + (pos - s.pos) as u64)
    }

    pub fn segments(&self) -> usize {
        self.segs.len()
    }
}

/// Where a walk stopped, if it did not finish.
#[derive(Clone, Debug)]
pub struct WalkStop {
    pub asset_index: usize,
    pub asset_type: Option<AssetType>,
    pub file_pos: usize,
    pub error: String,
}

pub struct Walker<'a> {
    pub(crate) s: Stream<'a>,
    pub(crate) list: &'a AssetList,
    pub(crate) vmap: VirtualMap,
    /// Asset pointer slots (the XAsset table's and "insert" slots) by
    /// VIRTUAL offset.
    pub(crate) slots: HashMap<u64, AssetId>,
    pub(crate) streams: &'a [StreamFile],
    pub(crate) next_stream_file: usize,
    pub cap: Capture,
    /// References to strings that land mid-string (a sign of VIRTUAL drift).
    pub suspect: std::cell::Cell<usize>,
    pub first_suspect: std::cell::Cell<Option<(usize, usize)>>,
    /// Asset references that named no slot read so far (VIRTUAL drift).
    pub unresolved: std::cell::Cell<usize>,
    /// References past what has been read so far: pointers are only
    /// offsets, so a zone may name data (even asset headers) it loads later.
    pub forward: Vec<(u64, AssetType)>,
    /// Debug: every VIRTUAL string reference seen (real offsets).
    pub string_refs: Vec<u64>,
    /// How references' (real) VIRTUAL offsets map to the walk's own:
    /// (first real offset, delta) segments, sorted. Retail zones place
    /// data in VIRTUAL that the walk does not see (the shared streams), so
    /// real = ours + delta, the delta fitted from string references.
    pub deltas: Vec<(u64, u64)>,
    pub first_unresolved: std::cell::Cell<Option<(usize, usize, u64)>>,
    pub last_resolved: std::cell::Cell<Option<(usize, u64)>>,
    pub slot_pos: HashMap<u64, usize>,
    /// Debug: the latest registration position a successful lookup used, and
    /// the earliest a failed lookup's near miss came from (with its delta).
    pub good_until: std::cell::Cell<usize>,
    pub bad_from: std::cell::Cell<Option<(usize, i64)>>,
    /// Strings loaded through a pointer field, by the field's VIRTUAL
    /// offset (some references, e.g. effect names, name the field).
    pub(crate) str_slots: HashMap<u64, String>,
    /// Debug: VIRTUAL offsets where allocations started.
    pub alloc_starts: std::collections::HashSet<u64>,
    /// The correction the last reference resolved near its slot needed.
    slot_fix: std::cell::Cell<i64>,
    /// Each asset loaded in place: the VIRTUAL its data took (start, end).
    pub spans: Vec<(u64, u64, AssetId)>,
}

#[cfg(test)]
fn trace_at() -> Option<usize> {
    std::env::var("IW7_TRACE_AT")
        .ok()
        .and_then(|v| usize::from_str_radix(v.trim_start_matches("0x"), 16).ok())
}

pub(crate) fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

impl<'a> Walker<'a> {
    pub fn new(zone: &'a [u8], list: &'a AssetList, streams: &'a [StreamFile]) -> Self {
        let mut off = [0u64; BLOCK_COUNT];
        off[VIRTUAL as usize] = list.body_virtual;
        let mut slots = HashMap::new();
        let _ = &mut slots;
        Self {
            s: Stream::new(zone, list.body_offset, off),
            list,
            vmap: VirtualMap::default(),
            slots,
            streams,
            next_stream_file: 0,
            cap: Capture::default(),
            suspect: Default::default(),
            first_suspect: Default::default(),
            unresolved: Default::default(),
            first_unresolved: Default::default(),
            forward: Vec::new(),
            string_refs: Vec::new(),
            deltas: Vec::new(),
            last_resolved: Default::default(),
            slot_pos: HashMap::new(),
            good_until: Default::default(),
            bad_from: Default::default(),
            alloc_starts: Default::default(),
            str_slots: HashMap::new(),
            slot_fix: Default::default(),
            spans: Vec::new(),
        }
    }

    // ---- stream helpers -------------------------------------------------

    /// `alloc` that records VIRTUAL data.
    pub(crate) fn take(&mut self, len: usize) -> Result<Option<usize>> {
        let b = self.s.block();
        let v = self.s.offset(b);
        let at = self.s.alloc(len)?;
        #[cfg(test)]
        if let Some(t) = trace_at()
            && let Some(a) = at
            && a <= t
            && t < a + len.max(1)
        {
            eprintln!(
                "TRACE {t:#x}: alloc at {a:#x} len {len:#x} block {b} virt {v:#x} asset {}",
                self.cap.walked
            );
        }
        #[cfg(test)]
        if std::env::var("IW7_TRACE_ASSET")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            == Some(self.cap.walked)
        {
            eprintln!("A alloc at {:x?} len {len:#x} block {b} virt {v:#x}", at);
        }
        if b == VIRTUAL
            && let Some(at) = at
        {
            self.vmap.add(v, at, len as u64);
            // (Debug: where allocations start, for the anchor checks.)
            #[cfg(test)]
            self.alloc_starts.insert(v);
        }
        Ok(at)
    }

    /// Align then `take`; the file offset (None for RUNTIME data).
    pub(crate) fn data(&mut self, len: usize, mask: u64) -> Result<Option<usize>> {
        self.s.align(mask);
        self.take(len)
    }

    /// A struct (or array) that must be in the file.
    pub(crate) fn st(&mut self, len: usize, mask: u64) -> Result<usize> {
        let at = self.s.pos();
        self.data(len, mask)?
            .ok_or(ReadError::Invalid("struct in runtime block", at))
    }

    /// The string at the stream (current block).
    pub(crate) fn cstr(&mut self) -> Result<String> {
        let b = self.s.block();
        let (bytes, at, off) = self.s.cstr()?;
        if b == VIRTUAL {
            self.vmap.add(off, at, bytes.len() as u64 + 1);
        }
        Ok(text(bytes))
    }

    /// The string a pointer field names: read here if it follows, else
    /// looked up where it was read.
    pub(crate) fn xstr(&mut self, field: usize) -> Result<Option<String>> {
        match self.s.ptr_at(field)? {
            Ptr::Null => Ok(None),
            p @ (Ptr::Follow | Ptr::Insert) => {
                if p == Ptr::Insert {
                    self.s.insert_slot();
                }
                let text = self.cstr()?;
                if let Some(v) = self.vmap.virt_of(field) {
                    self.str_slots.insert(v, text.clone());
                }
                Ok(Some(text))
            }
            Ptr::Ref {
                block: VIRTUAL,
                offset,
            } => {
                if let Some(t) = self.str_slots.get(&self.local(offset as u64)) {
                    return Ok(Some(t.clone()));
                }
                self.string_refs.push(offset as u64);
                Ok(Some(
                    self.string_at(offset as u64)
                        .unwrap_or_else(|| format!("<ref {offset:#x}>")),
                ))
            }
            Ptr::Ref { .. } | Ptr::SharedOffset | Ptr::Shared => Ok(Some("<ref>".to_owned())),
            Ptr::Other(_) => Ok(None),
        }
    }

    /// The string read earlier at VIRTUAL `virt`.
    /// The string at a reference's (real) VIRTUAL offset.
    pub fn string_at(&self, virt: u64) -> Option<String> {
        let at = self.file_of(virt)?;
        let data = self.s.data();
        if at > 0 && (0x21..0x7f).contains(&data[at - 1]) && (0x21..0x7f).contains(&data[at]) {
            self.suspect.set(self.suspect.get() + 1);
            if self.first_suspect.get().is_none() {
                self.first_suspect.set(Some((self.cap.walked, at)));
            }
        }
        let n = data.get(at..)?.iter().position(|&b| b == 0)?;
        Some(text(&data[at..at + n]))
    }

    /// File offset of data a pointer field names: inline (read here with
    /// `len`/`mask`) or read earlier (a VIRTUAL reference).
    pub(crate) fn blob(&mut self, field: usize, len: usize, mask: u64) -> Result<Option<usize>> {
        match self.s.ptr_at(field)? {
            Ptr::Null => Ok(None),
            p @ (Ptr::Follow | Ptr::Insert) => {
                if p == Ptr::Insert {
                    self.s.insert_slot();
                }
                self.data(len, mask)
            }
            Ptr::Ref {
                block: VIRTUAL,
                offset,
            } => Ok(self.file_of(offset as u64)),
            _ => Ok(None),
        }
    }

    /// Like `blob` but only says whether the data is inline here (the
    /// caller reads its contents itself, e.g. arrays of structs with
    /// pointers): Some(file offset) when inline.
    pub(crate) fn inline_array(
        &mut self,
        field: usize,
        len: usize,
        mask: u64,
    ) -> Result<Option<usize>> {
        match self.s.ptr_at(field)? {
            p @ (Ptr::Follow | Ptr::Insert) => {
                if p == Ptr::Insert {
                    self.s.insert_slot();
                }
                self.data(len, mask)
            }
            _ => Ok(None),
        }
    }

    /// A pointer to an asset of type `ty`: loaded right here when it
    /// follows, else the asset a reference names.
    pub(crate) fn asset(&mut self, ty: AssetType, field: usize) -> Result<Option<AssetId>> {
        let p = self.s.ptr_at(field)?;
        let id = self.asset_ptr(ty, p)?;
        // A pointer that loaded its asset is where later references to that
        // asset point (the loader stores the asset's address there).
        if p.is_inline()
            && let Some(id) = id
            && let Some(v) = self.vmap.virt_of(field)
        {
            self.slots.insert(v, id);
            #[cfg(test)]
            self.slot_pos.insert(v, self.s.pos());
        }
        Ok(id)
    }

    pub(crate) fn asset_ptr(&mut self, ty: AssetType, p: Ptr) -> Result<Option<AssetId>> {
        match p {
            Ptr::Null => Ok(None),
            Ptr::Follow | Ptr::Insert => {
                let slot = (p == Ptr::Insert).then(|| self.s.insert_slot());
                let id = self.load_inline(ty)?;
                if let Some(slot) = slot {
                    self.slots.insert(slot, id);
                    #[cfg(test)]
                    self.slot_pos.insert(slot, self.s.pos());
                }
                Ok(Some(id))
            }
            Ptr::Ref {
                block: VIRTUAL,
                offset,
            } => {
                let r = self.resolve_slot(ty, offset as u64);
                if r.is_none() && self.local(offset as u64) >= self.s.offset(VIRTUAL) {
                    if std::env::var_os("IW7_FWD").is_some() {
                        eprintln!("FWD asset {} pos {:#x} virt {:#x} want {:#x} {:?}", self.cap.walked, self.s.pos(), self.s.offset(VIRTUAL), offset, ty);
                    }
                    self.forward.push((offset as u64, ty));
                    return Ok(None);
                }
                if r.is_some() && std::env::var_os("IW7_RES").is_some() {
                    eprintln!("RES asset {} virt {:#x} want {:#x} {:?}", self.cap.walked, self.s.offset(VIRTUAL), offset, ty);
                }
                if r.is_some() && self.first_unresolved.get().is_none() {
                    self.last_resolved.set(Some((self.s.pos(), offset as u64)));
                }
                let v = offset as u64;
                if r.is_some() {
                    if let Some(&p) = self.slot_pos.get(&v) {
                        self.good_until.set(self.good_until.get().max(p));
                    }
                } else {
                    let near = self
                        .slots
                        .iter()
                        .filter(|(k, id)| id.ty == ty && k.abs_diff(v) <= 0x200)
                        .min_by_key(|(k, _)| k.abs_diff(v))
                        .map(|(k, _)| (*k, self.slot_pos.get(k).copied().unwrap_or(0)));
                    if std::env::var_os("IW7_SPANS").is_some() {
                        let l = self.local(v);
                        let inside: Vec<String> = self.spans.iter().filter(|(a, b, id)| *a <= l && l < *b && id.ty == ty).map(|(a, b, id)| format!("{} [{a:#x}..{b:#x}] +{:#x}", self.cap.name_of(*id), l - a)).collect();
                        let before = self.spans.iter().filter(|(_, b, id)| *b <= l && id.ty == ty).max_by_key(|(_, b, _)| *b).map(|(a, b, id)| format!("{} [{a:#x}..{b:#x}] end+{:#x}", self.cap.name_of(*id), l - b));
                        eprintln!("SPAN want local {l:#x} {ty:?} inside {inside:?} before {before:?}");
                    }
                    if std::env::var_os("IW7_MISSES").is_some() {
                        let ti = self.list.asset_index_of(Ptr::Ref { block: VIRTUAL, offset: v as u32 });
                        eprintln!(
                            "MISS at {:#x} want {v:#x} {:?} near {:x?} table {:?} (table at {:#x}) walked {}",
                            self.s.pos(),
                            ty,
                            near,
                            ti.map(|i| (i, self.list.assets[i].asset_type())),
                            self.list.asset_table_virtual,
                            self.cap.walked
                        );
                    }
                    if let Some((k, p)) = near
                        && self.bad_from.get().is_none_or(|(b, _)| p < b)
                    {
                        self.bad_from.set(Some((p, k as i64 - v as i64)));
                    }
                }
                if r.is_none() {
                    self.unresolved.set(self.unresolved.get() + 1);
                    if self.first_unresolved.get().is_none() {
                        self.first_unresolved.set(Some((
                            self.cap.walked,
                            self.s.pos(),
                            offset as u64,
                        )));
                    }
                }
                Ok(r)
            }
            _ => Ok(None),
        }
    }

    /// The asset a reference to a pointer slot names.
    pub(crate) fn resolve_slot(&self, ty: AssetType, virt: u64) -> Option<AssetId> {
        if let Some(id) = self.slots.get(&self.local(virt)) {
            return Some(*id);
        }
        if let Some(id) = self.slots.get(&virt) {
            return Some(*id);
        }
        // The XAsset table's header slots.
        let table = self
            .list
            .asset_index_of(Ptr::Ref {
                block: VIRTUAL,
                offset: virt as u32,
            })
            .and_then(|i| self.cap.by_table.get(&i).copied().filter(|id| id.ty == ty));
        if table.is_some() || self.deltas.is_empty() {
            return table;
        }
        // A fitted zone's delta can be off where its references switch
        // segments (the fit reads string references; long runs of other
        // data have none): the correction the last such reference needed,
        // else the slot of this type closest within 0x800 (its correction
        // kept for the next). (mp_geneva's placed models name their models
        // 0x4c0 below the fitted delta of their region: 6029 of 9221 missed
        // within 0x100; mp_renaissance2 and mp_hawkwar likewise.)
        let local = self.local(virt);
        let fix = self.slot_fix.get();
        if fix != 0
            && let Some(id) = self.slots.get(&local.wrapping_add_signed(fix)).copied().filter(|id| id.ty == ty)
        {
            return Some(id);
        }
        let found = (1..=512u64).find_map(|k| {
            [local.wrapping_sub(4 * k), local + 4 * k]
                .into_iter()
                .find_map(|v| self.slots.get(&v).copied().filter(|id| id.ty == ty).map(|id| (v, id)))
        });
        if let Some((at, id)) = found {
            self.slot_fix.set(at.wrapping_sub(local) as i64);
            return Some(id);
        }
        // Else an asset of this type loaded in place whose data holds the
        // offset (references into a model loaded inside another's data).
        self.spans
            .iter()
            .filter(|(a, b, id)| *a <= local && local < *b && id.ty == ty)
            .max_by_key(|(a, _, _)| *a)
            .map(|(_, _, id)| *id)
    }

    /// A sub-asset (or table asset) whose data starts here: its header in
    /// TEMP, its contents as the type reads them.
    pub(crate) fn load_inline(&mut self, ty: AssetType) -> Result<AssetId> {
        self.s.push(TEMP);
        let header = ty.header_size();
        if header == 0 {
            return Err(ReadError::Invalid(
                "asset type without a known layout",
                self.s.pos(),
            ));
        }
        let h = self.st(header, 7)?;
        let v0 = self.s.offset(VIRTUAL);
        #[cfg(test)]
        if std::env::var("IW7_TRACE_ASSET")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            == Some(self.cap.walked)
        {
            eprintln!(
                "A load {ty:?} header {h:#x} virt {:#x}",
                self.s.offset(VIRTUAL)
            );
        }
        let id = self.load_body(ty, h);
        if let Ok(id) = id {
            self.spans.push((v0, self.s.offset(VIRTUAL), id));
        }
        #[cfg(test)]
        if std::env::var("IW7_TRACE_ASSET")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            == Some(self.cap.walked)
        {
            eprintln!(
                "A done {ty:?} pos {:#x} virt {:#x}",
                self.s.pos(),
                self.s.offset(VIRTUAL)
            );
        }
        self.s.pop();
        id
    }

    fn load_body(&mut self, ty: AssetType, h: usize) -> Result<AssetId> {
        use AssetType as T;
        match ty {
            T::RawFile => misc::raw_file(self, h),
            T::NetConstStrings => misc::net_const_strings(self, h),
            T::LightDef => misc::light_def(self, h),
            T::Tracer => misc::tracer(self, h),
            T::ScriptFile => misc::script_file(self, h),
            T::StreamingInfo => misc::streaming_info(self, h),
            T::VertexShader => material::shader(self, ty, h, false),
            T::PixelShader | T::HullShader | T::DomainShader | T::ComputeShader => material::shader(self, ty, h, true),
            T::VertexDecl => material::vertex_decl(self, h),
            T::TechniqueSet => material::technique_set(self, h),
            T::Image => material::image(self, h),
            T::Material => material::material(self, h),
            T::XModel => model::xmodel(self, h),
            T::XModelSurfs => model::xmodel_surfs(self, h),
            T::PhysicsAsset => physics::physics_asset(self, h),
            T::PhysicsSfxEventAsset | T::PhysicsVfxEventAsset => physics::event_asset(self, ty, h),
            T::XAnimProceduralBones => physics::procedural_bones(self, h),
            T::Weapon => weapon::weapon(self, h),
            T::AnimPackage => weapon::anim_package(self, h),
            T::XAnimParts => anim::xanim(self, h),
            T::PhysicsFxPipeline => physics::fx_pipeline(self, h),
            T::PhysicsFxShape => physics::fx_shape(self, h),
            T::PhysicsLibrary => physics::library(self, h),
            T::Attachment => weapon::attachment(self, h),
            T::Vfx => vfx::vfx(self, h),
            T::Scriptable => scriptable::scriptable(self, h),
            T::PlayerAnim => playeranim::playeranim(self, h),
            T::Suit => suit::suit(self, h),
            T::SuitAnimPackage => suit::suit_anim_package(self, h),
            T::Vehicle => vehicle::vehicle(self, h),
            T::SpaceshipTarget => spaceship::spaceship(self, h),
            T::Rumble => misc::rumble(self, h),
            T::RumbleGraph => misc::name_only(self, ty, h),
            T::Laser => misc::laser(self, h),
            T::Reticle => misc::reticle(self, h),
            T::Gesture => misc::gesture(self, h),
            T::SfxPackage => weapon::sfx_package(self, h),
            T::VfxPackage => weapon::vfx_package(self, h),
            T::AnimClass => anim_class::anim_class(self, h),
            T::SoundBank => sound_bank::sound_bank(self, h),
            T::BehaviorTree => behavior_tree::behavior_tree(self, h),
            T::ComWorld => world::comworld(self, h),
            T::NavMesh => world::navmesh(self, h),
            T::ImpactFx => world::impact_fx(self, h),
            T::FxWorld => world::fxworld(self, h),
            T::GfxLightMap => world::gfx_light_map(self, h),
            T::IesProfile => world::ies_profile(self, h),
            T::GfxWorld => world::gfxworld(self, h),
            T::GfxWorldTransientZone => world::gfx_world_tr(self, h),
            T::MapEnts => world::mapents(self, h),
            T::GlassWorld => world::glassworld(self, h),
            T::PathData => world::pathdata(self, h),
            T::ClipMap => world::clipmap(self, h),
            T::LocalizeEntry => misc::localize(self, h),
            T::ParticleSimAnimation => vfx::particle_sim(self, h),
            T::StringTable => misc::string_table(self, h),
            _ => Err(ReadError::Invalid(ty.name(), h)),
        }
    }

    pub(crate) fn name_only(&mut self, ty: AssetType, name: String) -> AssetId {
        self.cap.push_other(ty, name)
    }

    /// The next streamed image's four `XStreamFile`s from the zone header.
    pub(crate) fn take_stream_files(&mut self) -> [Option<StreamFile>; 4] {
        let mut out = [None; 4];
        for o in out.iter_mut() {
            *o = self.streams.get(self.next_stream_file).copied();
            self.next_stream_file += 1;
        }
        out
    }

    /// Walks every asset in table order; stops at the first one it cannot
    /// read (the capture keeps what came before).
    pub fn walk(&mut self) -> std::result::Result<(), WalkStop> {
        // Debug: `IW7_START=<index>,<hex file offset>` starts the walk at a
        // table entry whose data begins at that offset (references made
        // before it will not resolve).
        let start = std::env::var("IW7_START").ok().and_then(|v| {
            let (i, p) = v.split_once(',')?;
            Some((i.parse::<usize>().ok()?, usize::from_str_radix(p.trim_start_matches("0x"), 16).ok()?))
        });
        self.walk_from(start)
    }

    /// `walk`, starting at table entry `start.0` whose data begins at file
    /// offset `start.1` (to read past an asset type the walk cannot read
    /// when the next asset's position is known; references to what was
    /// skipped will not resolve).
    pub fn walk_from(&mut self, start: Option<(usize, usize)>) -> std::result::Result<(), WalkStop> {
        let table = self.list.asset_table_virtual;
        if let Some((_, p)) = start {
            self.s.seek(p);
        }
        for (i, entry) in self.list.assets.iter().enumerate() {
            if let Some((first, _)) = start
                && i < first
            {
                self.cap.starts.push(0);
                continue;
            }
            let ty = entry.asset_type();
            let start = self.s.pos();
            self.cap.starts.push(start);
            let result = match (ty, entry.header) {
                (Some(ty), p @ (Ptr::Follow | Ptr::Insert)) => {
                    let depth = self.s.depth();
                    let r = self.load_inline(ty);
                    debug_assert_eq!(depth, self.s.depth());
                    let _ = p;
                    r.map(Some)
                }
                (Some(ty), p @ Ptr::Ref { .. }) => {
                    if std::env::var_os("IW7_TREF").is_some() {
                        let v = p.virtual_offset().unwrap_or(0);
                        let s0: u64 = self.cap.images.iter().map(|im| im.streams[0].2 as u64).sum();
                        eprintln!("TREF {i} {ty:?} want {v:#x} ours {:#x} delta {:#x} images {} s0sum {s0:#x} resolved {:?}", self.s.offset(VIRTUAL), v.wrapping_sub(self.s.offset(VIRTUAL)), self.cap.images.len(), self.resolve_slot(ty, v));
                    }
                    Ok(p.virtual_offset().and_then(|v| self.resolve_slot(ty, v)))
                }
                (Some(_), _) => Ok(None),
                (None, _) => Err(ReadError::Invalid("unknown asset type", start)),
            };
            match result {
                Ok(Some(id)) => {
                    self.cap.by_table.insert(i, id);
                    self.slots.insert(table + (XASSET_LEN * i) as u64 + 8, id);
                }
                Ok(None) => {}
                Err(e) => {
                    return Err(WalkStop {
                        asset_index: i,
                        asset_type: ty,
                        file_pos: self.s.pos(),
                        error: e.to_string(),
                    });
                }
            }
            self.cap.walked = i + 1;
        }
        Ok(())
    }

    /// A reference's offset in the walk's own VIRTUAL numbering.
    pub fn local(&self, real: u64) -> u64 {
        let i = self.deltas.partition_point(|&(r, _)| r <= real);
        match i {
            0 => real,
            _ => real.wrapping_sub(self.deltas[i - 1].1),
        }
    }

    /// The file offset of data a (real) VIRTUAL reference names.
    pub fn file_of(&self, real: u64) -> Option<usize> {
        self.vmap.file_pos(self.local(real))
    }

    /// Whether `ours` is the first byte of a string read in VIRTUAL.
    fn is_string_start(&self, ours: u64) -> bool {
        let data = self.s.data();
        self.vmap.file_pos(ours).is_some_and(|at| at > 0 && data[at - 1] == 0 && data[at] != 0)
    }

    /// Fits `deltas` from the string references of a finished walk: the
    /// delta near `hint` that most references agree on, then a sweep that
    /// switches to a nearby delta where a run of references needs one.
    pub fn fit_deltas(&self, hint: u64) -> Vec<(u64, u64)> {
        self.fit_deltas_below(hint, 0x10000)
    }

    /// `fit_deltas` searching from `below` under `hint` (a zone with many
    /// shared ranges sits further below its header's figure: about 31
    /// bytes a range).
    pub fn fit_deltas_below(&self, hint: u64, below: u64) -> Vec<(u64, u64)> {
        let mut refs = self.string_refs.clone();
        refs.sort_unstable();
        refs.dedup();
        if refs.is_empty() || hint == 0 {
            return Vec::new();
        }
        let sample: Vec<u64> = refs.iter().step_by((refs.len() / 256).max(1)).take(256).copied().collect();
        let lo = hint.saturating_sub(below);
        let hits = |d: u64, rs: &[u64]| rs.iter().filter(|&&r| r >= d && self.is_string_start(r - d)).count();
        let Some(best) = (lo..hint + 0x1000).max_by_key(|&d| hits(d, &sample[..sample.len().min(32)])) else {
            return Vec::new();
        };
        let mut out = vec![(0u64, best)];
        let mut d = best;
        let mut i = 0;
        while i < refs.len() {
            let r = refs[i];
            if r >= d && self.is_string_start(r - d) {
                i += 1;
                continue;
            }
            // A run of misses: try the deltas nearby against the next refs.
            let next = &refs[i..(i + 6).min(refs.len())];
            let cand = (d.saturating_sub(0x800)..d + 0x800)
                .step_by(1)
                .filter(|&c| r >= c && self.is_string_start(r - c))
                .max_by_key(|&c| (hits(c, next), std::cmp::Reverse(c.abs_diff(d))));
            if let Some(c) = cand
                && hits(c, next) * 2 > next.len()
                && hits(c, next) > hits(d, next)
            {
                d = c;
                out.push((r, d));
            }
            i += 1;
        }
        out
    }

    /// Debug: for each delta in `range`, how many string references land
    /// on a string start when shifted back by it.
    pub fn delta_hits(&self, refs: &[u64], range: std::ops::Range<u64>) -> Vec<(u64, usize)> {
        let data = self.s.data();
        range
            .map(|d| {
                let hits = refs
                    .iter()
                    .filter(|&&r| {
                        r.checked_sub(d)
                            .and_then(|v| self.vmap.file_pos(v))
                            .is_some_and(|at| at > 0 && data[at - 1] == 0 && data[at] != 0)
                    })
                    .count();
                (d, hits)
            })
            .collect()
    }

    /// Forward references that never named an asset by the walk's end.
    pub fn unresolved_forward(&self) -> usize {
        self.forward.iter().filter(|(v, ty)| self.resolve_slot(*ty, *v).is_none()).count()
    }

    pub fn slot_list(&self) -> Vec<(u64, AssetId)> {
        self.slots.iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub fn stream_pos(&self) -> usize {
        self.s.pos()
    }
    pub fn virtual_offset(&self) -> u64 {
        self.s.offset(VIRTUAL)
    }
    pub fn virtual_map(&self) -> &VirtualMap {
        &self.vmap
    }
}

/// Walks a whole zone: the capture, and where it stopped if it did.
pub fn walk_zone(zone: &crate::Zone) -> (Capture, Option<WalkStop>) {
    let w = walk_fitted(zone);
    let stop = w.1;
    (w.0.cap, stop)
}

/// Walks a zone; when its references are offset from the walk's VIRTUAL
/// numbering (retail zones with shared streams), fits the offsets from a
/// first walk and walks again with them.
pub fn walk_fitted(zone: &crate::Zone) -> (Walker<'_>, Option<WalkStop>) {
    let mut w = Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
    let stop = w.walk().err();
    let hint = zone.header.shared_virtual;
    if hint == 0 {
        return (w, stop);
    }
    let below = (64 * zone.header.shared_files.len() as u64).max(0x10000);
    let deltas = w.fit_deltas_below(hint, below);
    let mut w2 = Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
    w2.deltas = deltas;
    let stop = w2.walk().err();
    (w2, stop)
}

/// Walks a zone's shared asset streams (`pak::SharedVirtual`): each range
/// is a run of `XAsset` entries (type, header pointer "follows"), each
/// followed by its asset's data, read the way a zone's are. On retail
/// these are the zone's model surfaces (`xmodel_surfs`): the zone itself
/// holds only comma-named placeholders for them, as it does for the
/// materials `techsets_<zone>.ff` defines. Spans in the capture are offsets
/// into `shared.data`. Returns the capture and, per range that did not
/// read to its end, where it stopped.
pub fn walk_shared(shared: &crate::pak::SharedVirtual) -> (Capture, Vec<WalkStop>) {
    let list = AssetList {
        script_strings: Vec::new(),
        globals: None,
        assets: Vec::new(),
        asset_table_virtual: 0,
        body_offset: 0,
        body_virtual: 0,
    };
    let mut w = Walker::new(&shared.data, &list, &[]);
    let mut stops = Vec::new();
    for (i, &(start, len)) in shared.ranges.iter().enumerate() {
        let (start, end) = (start as usize, (start + len) as usize);
        let mut off = [0u64; BLOCK_COUNT];
        off[VIRTUAL as usize] = start as u64;
        w.s = Stream::new(&shared.data, start, off);
        loop {
            let at = w.s.pos();
            if at + XASSET_LEN > end {
                break;
            }
            let raw = w.s.u32_at(at).unwrap_or(u32::MAX);
            let header = w.s.ptr_at(at + 8).unwrap_or(Ptr::Null);
            let ty = AssetType::from_u32(raw);
            let r = match (ty, header.is_inline()) {
                (Some(ty), true) => {
                    w.s.seek(at + XASSET_LEN);
                    w.load_inline(ty).map(|_| ())
                }
                _ => Err(ReadError::Invalid("not an XAsset entry", at)),
            };
            if let Err(e) = r {
                stops.push(WalkStop { asset_index: i, asset_type: ty, file_pos: at, error: e.to_string() });
                break;
            }
            w.cap.walked += 1;
            if w.s.pos() >= end {
                break;
            }
        }
        if w.s.pos() != end && stops.last().is_none_or(|s| s.asset_index != i) {
            stops.push(WalkStop { asset_index: i, asset_type: None, file_pos: w.s.pos(), error: format!("range ends at {end:#x}") });
        }
    }
    (w.cap, stops)
}
