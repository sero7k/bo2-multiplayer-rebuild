//! What a walk keeps. Bulk data (vertices, indices, pixels, animation
//! streams) stays in the zone image: records hold its file offsets.

use std::collections::HashMap;

use crate::asset_type::AssetType;
use crate::container::StreamFile;

/// An asset the walk read: its type and its index in that type's list
/// (`Capture::others` for the types kept by name only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssetId {
    pub ty: AssetType,
    pub index: u32,
}

/// A byte range of the zone image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub pos: usize,
    pub len: usize,
}

impl Span {
    pub fn get<'a>(&self, image: &'a [u8]) -> &'a [u8] {
        image.get(self.pos..self.pos + self.len).unwrap_or(&[])
    }
}

#[derive(Clone, Debug, Default)]
pub struct TechniqueSetRec {
    pub name: String,
    pub world_vert_format: u8,
    /// (technique index, technique name, pass count)
    pub techniques: Vec<(usize, String, u16)>,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialTexture {
    pub name_hash: u32,
    pub name_start: u8,
    pub name_end: u8,
    pub sampler_state: u8,
    pub semantic: u8,
    pub image: Option<AssetId>,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialConstant {
    pub name_hash: u32,
    pub name: String,
    pub literal: [f32; 4],
}

#[derive(Clone, Debug, Default)]
pub struct MaterialRec {
    pub name: String,
    pub game_flags: u8,
    pub sort_key: u8,
    pub texture_atlas: (u8, u8),
    pub state_flags: u8,
    pub camera_region: u8,
    pub material_type: u8,
    pub techset: Option<AssetId>,
    pub textures: Vec<MaterialTexture>,
    pub constants: Vec<MaterialConstant>,
    /// GfxStateBits, 28 bytes each.
    pub state_bits: Vec<[u8; crate::layout::GfxStateBits::SIZE]>,
    pub state_bits_entry: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct ImageRec {
    pub name: String,
    /// DXGI_FORMAT.
    pub format: u32,
    pub map_type: u8,
    pub semantic: u8,
    pub category: u8,
    pub flags: u8,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub num_elements: u16,
    pub level_count: u8,
    pub streamed: bool,
    pub data_len1: u32,
    pub data_len2: u32,
    /// Inline pixels (all levels).
    pub pixels: Option<Span>,
    /// Streamed: per stream (biggest last): (width, height, cumulative pixel
    /// bytes, level count), and its `imagefile<N>.pak` range.
    pub streams: [(u16, u16, u32, u8); 4],
    pub stream_files: [Option<StreamFile>; 4],
}

#[derive(Clone, Debug, Default)]
pub struct SurfaceRec {
    pub flags: u16,
    pub vert_count: u16,
    pub tri_count: u16,
    pub blend_vert_counts: [u16; 8],
    /// `GfxPackedVertex` x vert_count (32 bytes each).
    pub verts: Option<Span>,
    /// u16 x 3 x tri_count.
    pub tris: Option<Span>,
    /// (bone offset, vert count, tri offset, tri count)
    pub rigid: Vec<(u16, u16, u16, u16)>,
    /// u16 stream: blend verts (see `XSurface` docs).
    pub blend_verts: Option<Span>,
    pub part_bits: [u32; 8],
}

#[derive(Clone, Debug, Default)]
pub struct ModelSurfsRec {
    pub name: String,
    pub surfs: Vec<SurfaceRec>,
}

#[derive(Clone, Debug, Default)]
pub struct LodRec {
    pub dist: f32,
    pub numsurfs: u16,
    pub surf_index: u16,
    pub surfs: Option<AssetId>,
}

#[derive(Clone, Debug, Default)]
pub struct XModelRec {
    pub name: String,
    pub num_bones: u8,
    pub num_root_bones: u8,
    pub numsurfs: u8,
    pub scale: f32,
    /// Script string indices.
    pub bone_names: Vec<u32>,
    pub parent_list: Vec<u8>,
    /// i16 x 4 quaternions per non-root bone.
    pub tag_angles: Vec<[i16; 4]>,
    pub tag_positions: Vec<[f32; 3]>,
    pub part_classification: Vec<u8>,
    /// DObjAnimMat per bone: quat[4], trans[3], transWeight.
    pub base_mat: Vec<[f32; 8]>,
    pub materials: Vec<Option<AssetId>>,
    pub lods: Vec<LodRec>,
    pub num_lods: u8,
    pub coll_lod: i8,
    pub flags: u8,
    pub contents: i32,
    pub radius: f32,
    pub bounds: ([f32; 3], [f32; 3]),
    pub phys_preset: Option<AssetId>,
    pub phys_collmap: Option<AssetId>,
    /// AW's composite models: the part models drawn together (a gun's
    /// base, sight, stock ...), each with its own surfaces.
    pub composite: Vec<Option<AssetId>>,
}

#[derive(Clone, Debug, Default)]
pub struct XAnimRec {
    pub name: String,
    pub data_byte_count: u16,
    pub data_short_count: u16,
    pub data_int_count: u16,
    pub random_data_byte_count: u16,
    pub random_data_int_count: u16,
    pub numframes: u16,
    pub flags: u8,
    pub bone_count: [u8; 10],
    pub notify_count: u8,
    pub asset_type: u8,
    pub ik_type: u8,
    pub random_data_short_count: u32,
    pub index_count: u32,
    pub framerate: f32,
    pub frequency: f32,
    pub names: Vec<u32>,
    /// (script string, time)
    pub notify: Vec<(u32, f32)>,
    pub delta: Option<DeltaRec>,
    pub data_byte: Option<Span>,
    pub data_short: Option<Span>,
    pub data_int: Option<Span>,
    pub random_data_short: Option<Span>,
    pub random_data_byte: Option<Span>,
    pub random_data_int: Option<Span>,
    pub indices: Option<Span>,
}

/// The root bone's delta motion.
#[derive(Clone, Debug, Default)]
pub struct DeltaRec {
    /// Translation: frame count - 1 (0: a single `frame0`), small (u8) frames.
    pub trans: Option<(u16, bool, Span)>,
    pub quat2: Option<(u16, Span)>,
    pub quat: Option<(u16, Span)>,
}

/// Where a weapon's field is: in its `WeaponDef` or `WeaponCompleteDef`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeaponField {
    Def(usize),
    Complete(usize),
    /// Element `.1` of the array a `WeaponDef` pointer field names (the
    /// complex models).
    DefArray(usize, usize),
}

#[derive(Clone, Debug, Default)]
pub struct WeaponRec {
    pub name: String,
    pub display_name: String,
    /// The `WeaponDef`'s and `WeaponCompleteDef`'s bytes, as stored.
    pub def: Option<Span>,
    pub complete: Span,
    /// Every asset the weapon points at, by field.
    pub assets: Vec<(WeaponField, AssetId)>,
    /// Every string (sound aliases, names, hint strings), by field.
    pub strings: Vec<(WeaponField, String)>,
    /// The six attachment lists.
    pub attachments: [Vec<Option<AssetId>>; 6],
    pub hide_tags: Vec<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct AttachmentRec {
    pub name: String,
    pub display_name: String,
    pub record: Span,
    /// Asset pointers in the record (and its sections), by offset.
    pub assets: Vec<(usize, AssetId)>,
    /// Elements of the arrays the record points at: (field, index, asset).
    pub arrays: Vec<(usize, usize, AssetId)>,
    pub strings: Vec<(usize, String)>,
}

/// `WeaponAnimPackage`: the 260 animation slots.
#[derive(Clone, Debug, Default)]
pub struct AnimPackageRec {
    pub name: String,
    pub anims: Vec<Option<AssetId>>,
}

/// `WeaponSFXPackage`: its sound aliases by offset in the sounds struct.
#[derive(Clone, Debug, Default)]
pub struct SfxPackageRec {
    pub name: String,
    pub sounds: Vec<(usize, String)>,
}

/// `WeaponVFXPackage`: its effects by offset in `WeaponVFXPackageDefs`.
#[derive(Clone, Debug, Default)]
pub struct VfxPackageRec {
    pub name: String,
    pub effects: Vec<(usize, AssetId)>,
}

#[derive(Clone, Debug, Default)]
pub struct FxRec {
    pub name: String,
    pub flags: i32,
    pub elem_counts: (i32, i32, i32),
    /// Per element: type, visual count, visuals (materials, models, ...).
    pub elems: Vec<FxElemRec>,
    pub record: Span,
}

#[derive(Clone, Debug, Default)]
pub struct FxElemRec {
    pub elem_type: u8,
    pub record: Span,
    pub materials: Vec<AssetId>,
    pub models: Vec<AssetId>,
    pub effects: Vec<String>,
    pub sounds: Vec<String>,
}

/// A sound alias list: per alias, its sound file name (in the zone's
/// `.sabl`/`.sabs` sound asset banks), its `SndAliasType` (0 loaded,
/// 1 hybrid PCM, 2 streamed, 3 primed) and asset id.
#[derive(Clone, Debug, Default)]
pub struct SndAliasRec {
    pub name: String,
    pub entries: Vec<(Option<String>, u8, u32)>,
}

#[derive(Clone, Debug, Default)]
pub struct SoundBankRec {
    pub name: String,
    pub zone: String,
    pub game_language: String,
    pub sound_language: String,
    pub aliases: Vec<SndAliasRec>,
}

#[derive(Clone, Debug, Default)]
pub struct OtherRec {
    pub ty: Option<AssetType>,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct RawFileRec {
    pub name: String,
    pub compressed_len: u32,
    pub len: u32,
    pub data: Option<Span>,
}

#[derive(Clone, Debug, Default)]
pub struct ComWorldRec {
    pub name: String,
    pub record: Span,
    pub primary_lights: Option<Span>,
    pub primary_light_count: u32,
}

/// A `GfxWorld`: where its arrays are in the file (`None`: not inline).
#[derive(Clone, Debug, Default)]
pub struct GfxWorldRec {
    pub name: String,
    pub header: usize,
    pub vertices: Option<usize>,
    pub vertex_count: u32,
    pub indices: Option<usize>,
    pub index_count: u32,
    pub surfaces: Option<usize>,
    pub surface_count: u32,
    pub surface_materials: Vec<Option<super::AssetId>>,
    pub sorted_surfs: Option<usize>,
    pub static_surface_count: u32,
    pub smodels: Option<usize>,
    pub smodel_count: u32,
    pub smodel_models: Vec<Option<super::AssetId>>,
    pub models: Option<usize>,
    pub model_count: u32,
    pub lightmaps: Vec<(Option<super::AssetId>, Option<super::AssetId>)>,
    pub sky_images: Vec<Option<super::AssetId>>,
}

/// A `MapEnts`: the entity string and the spawn points' records.
#[derive(Clone, Debug, Default)]
pub struct MapEntsRec {
    pub name: String,
    pub entity_string: String,
    pub spawns: Option<Span>,
}

/// A `clipMap_t`: its terrain triangles (brushes are not captured yet).
#[derive(Clone, Debug, Default)]
pub struct ClipMapRec {
    pub name: String,
    pub header: usize,
    pub verts: Option<usize>,
    pub vert_count: u32,
    pub tri_indices: Option<usize>,
    pub tri_count: u32,
}

#[derive(Default, Debug)]
pub struct Capture {
    pub techsets: Vec<TechniqueSetRec>,
    pub materials: Vec<MaterialRec>,
    pub images: Vec<ImageRec>,
    pub xmodels: Vec<XModelRec>,
    pub model_surfs: Vec<ModelSurfsRec>,
    pub xanims: Vec<XAnimRec>,
    pub weapons: Vec<WeaponRec>,
    pub anim_packages: Vec<AnimPackageRec>,
    pub sfx_packages: Vec<SfxPackageRec>,
    pub vfx_packages: Vec<VfxPackageRec>,
    pub attachments: Vec<AttachmentRec>,
    pub fx: Vec<FxRec>,
    pub raw_files: Vec<RawFileRec>,
    pub com_worlds: Vec<ComWorldRec>,
    pub gfx_worlds: Vec<GfxWorldRec>,
    pub map_ents: Vec<MapEntsRec>,
    pub clip_maps: Vec<ClipMapRec>,
    pub sound_banks: Vec<SoundBankRec>,
    /// Everything else, by name.
    pub others: Vec<OtherRec>,
    /// The table index -> what it loaded.
    pub by_table: HashMap<usize, AssetId>,
    /// Table entries walked.
    pub walked: usize,
    /// Where each walked table entry's header was read (file offset).
    pub starts: Vec<usize>,
}

impl Capture {
    pub(crate) fn push_other(&mut self, ty: AssetType, name: String) -> AssetId {
        self.others.push(OtherRec { ty: Some(ty), name });
        AssetId {
            ty,
            index: (self.others.len() - 1) as u32,
        }
    }

    /// The name of any captured asset.
    pub fn name_of(&self, id: AssetId) -> &str {
        let i = id.index as usize;
        use AssetType as T;
        let s: Option<&str> = match id.ty {
            T::TechniqueSet => self.techsets.get(i).map(|r| r.name.as_str()),
            T::Material => self.materials.get(i).map(|r| r.name.as_str()),
            T::Image => self.images.get(i).map(|r| r.name.as_str()),
            T::XModel => self.xmodels.get(i).map(|r| r.name.as_str()),
            T::XModelSurfs => self.model_surfs.get(i).map(|r| r.name.as_str()),
            T::XAnimParts => self.xanims.get(i).map(|r| r.name.as_str()),
            T::Weapon => self.weapons.get(i).map(|r| r.name.as_str()),
            T::Attachment => self.attachments.get(i).map(|r| r.name.as_str()),
            T::AnimPackage => self.anim_packages.get(i).map(|r| r.name.as_str()),
            T::SfxPackage => self.sfx_packages.get(i).map(|r| r.name.as_str()),
            T::VfxPackage => self.vfx_packages.get(i).map(|r| r.name.as_str()),
            T::Fx => self.fx.get(i).map(|r| r.name.as_str()),
            T::RawFile => self.raw_files.get(i).map(|r| r.name.as_str()),
            T::ComWorld => self.com_worlds.get(i).map(|r| r.name.as_str()),
            T::GfxWorld => self.gfx_worlds.get(i).map(|r| r.name.as_str()),
            T::MapEnts => self.map_ents.get(i).map(|r| r.name.as_str()),
            T::ClipMap => self.clip_maps.get(i).map(|r| r.name.as_str()),
            T::SoundBank => self.sound_banks.get(i).map(|r| r.name.as_str()),
            _ => self.others.get(i).map(|r| r.name.as_str()),
        };
        s.unwrap_or("")
    }

    /// Every captured asset of a type with its name, in capture order.
    pub fn names(&self, ty: AssetType) -> Vec<String> {
        let n = match ty {
            AssetType::TechniqueSet => self.techsets.len(),
            AssetType::Material => self.materials.len(),
            AssetType::Image => self.images.len(),
            AssetType::XModel => self.xmodels.len(),
            AssetType::XModelSurfs => self.model_surfs.len(),
            AssetType::XAnimParts => self.xanims.len(),
            AssetType::Weapon => self.weapons.len(),
            AssetType::Attachment => self.attachments.len(),
            AssetType::AnimPackage => self.anim_packages.len(),
            AssetType::SfxPackage => self.sfx_packages.len(),
            AssetType::VfxPackage => self.vfx_packages.len(),
            AssetType::Fx => self.fx.len(),
            AssetType::RawFile => self.raw_files.len(),
            AssetType::ComWorld => self.com_worlds.len(),
            AssetType::GfxWorld => self.gfx_worlds.len(),
            AssetType::MapEnts => self.map_ents.len(),
            AssetType::ClipMap => self.clip_maps.len(),
            AssetType::SoundBank => self.sound_banks.len(),
            _ => {
                return self
                    .others
                    .iter()
                    .filter(|o| o.ty == Some(ty))
                    .map(|o| o.name.clone())
                    .collect();
            }
        };
        (0..n)
            .map(|i| {
                self.name_of(AssetId {
                    ty,
                    index: i as u32,
                })
                .to_owned()
            })
            .collect()
    }
}
