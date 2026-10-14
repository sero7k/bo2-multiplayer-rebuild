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
mod capture;
mod material;
mod misc;
mod model;
mod physics;
mod scriptable;
mod vfx;
mod weapon;

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
            last_resolved: Default::default(),
            slot_pos: HashMap::new(),
            good_until: Default::default(),
            bad_from: Default::default(),
            alloc_starts: Default::default(),
            str_slots: HashMap::new(),
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
                if let Some(t) = self.str_slots.get(&(offset as u64)) {
                    return Ok(Some(t.clone()));
                }
                if let Ok(d) = std::env::var("IW7_D") {
                    let d = u64::from_str_radix(&d, 16).unwrap_or(0);
                    let v = (offset as u64).wrapping_sub(d);
                    let ok = self.vmap.file_pos(v).map(|at| at > 0 && self.s.data()[at - 1] == 0 && self.s.data()[at] != 0);
                    let mut near = None;
                    if ok != Some(true) {
                        for k in 1..64u64 {
                            for c in [v.wrapping_sub(k), v + k] {
                                if near.is_none() && self.vmap.file_pos(c).is_some_and(|at| at > 0 && self.s.data()[at - 1] == 0 && self.s.data()[at] != 0) {
                                    near = Some(c as i64 - v as i64);
                                }
                            }
                        }
                    }
                    eprintln!("DSTR asset {} virt {:#x} ok {:?} near {:?} {:?}", self.cap.walked, self.s.offset(VIRTUAL), ok, near, self.string_at(v));
                }
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
    pub fn string_at(&self, virt: u64) -> Option<String> {
        let at = self.vmap.file_pos(virt)?;
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
            } => Ok(self.vmap.file_pos(offset as u64)),
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
                if r.is_none() && offset as u64 >= self.s.offset(VIRTUAL) {
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
        if let Some(id) = self.slots.get(&virt) {
            return Some(*id);
        }
        // The XAsset table's header slots.
        let i = self.list.asset_index_of(Ptr::Ref {
            block: VIRTUAL,
            offset: virt as u32,
        })?;
        self.cap.by_table.get(&i).copied().filter(|id| id.ty == ty)
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
            T::Attachment => weapon::attachment(self, h),
            T::Vfx => vfx::vfx(self, h),
            T::Scriptable => scriptable::scriptable(self, h),
            T::Rumble => misc::rumble(self, h),
            T::RumbleGraph => misc::name_only(self, ty, h),
            T::Laser => misc::laser(self, h),
            T::Reticle => misc::reticle(self, h),
            T::Gesture => misc::gesture(self, h),
            T::SfxPackage => weapon::sfx_package(self, h),
            T::VfxPackage => weapon::vfx_package(self, h),
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
        let table = self.list.asset_table_virtual;
        for (i, entry) in self.list.assets.iter().enumerate() {
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
    let mut w = Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
    let stop = w.walk().err();
    (w.cap, stop)
}
