//! What a walk keeps besides the asset list: the weapons' records, and
//! for following a weapon to its art, the models (LOD meshes and their
//! materials), materials (technique set and images) and images (size,
//! format, where the pixels are: in the zone or the `.xpak` entries).

use std::collections::HashMap;

use crate::asset_type::AssetType;
use crate::walk::image::XPakEntry;

/// What a weapon field names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    /// An asset of `ty`: its name when it was loaded here or by an earlier
    /// asset the reference names (`<ref 0x...>` when the walk cannot tell).
    Asset { ty: AssetType, name: String },
    /// A string (sound alias, weapon name, tag, hint, ...).
    Str(String),
}

/// One weapon (`WeaponVariantDef` and its `WeaponDef`).
#[derive(Clone, Debug, Default)]
pub struct WeaponRec {
    /// The variant's name (`ar_standard_mp`) and its `WeaponDef`'s name field
    /// (`ar_standard`).
    pub name: String,
    pub base_name: String,
    pub display_name: String,
    /// File offsets of the variant header and of the `WeaponDef` struct.
    pub header_at: usize,
    pub def_at: Option<usize>,
    /// When the `WeaponDef` is a reference to another variant's: that variant.
    pub def_shared_with: Option<String>,
    /// Every non-null pointer field (but empty strings), in stream order:
    /// (field name, value).
    /// Unnamed fields read as `def+0x...`/`variant+0x...`; arrays as
    /// `field[i]`.
    pub fields: Vec<(String, FieldValue)>,
    /// The animation table (197 slots): (slot, slot name, animation name).
    pub anims: Vec<(usize, String, String)>,
    /// `hideTags`: script string indices (32 slots, zero ones dropped).
    pub hide_tags: Vec<u32>,
    /// bo2mp: the `WeaponDef`'s location damage multipliers (21 f32), when
    /// it loaded them: file offset.
    pub location_damage_at: Option<usize>,
}

impl WeaponRec {
    /// The first field called `name`.
    pub fn field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}

/// One LOD of a model: its mesh and the materials of its surfaces.
#[derive(Clone, Debug, Default)]
pub struct LodRec {
    pub mesh: String,
    pub materials: Vec<String>,
}

/// An `XModel` loaded in the zone.
#[derive(Clone, Debug, Default)]
pub struct ModelRec {
    pub name: String,
    /// File offset of the model's header.
    pub header_at: usize,
    pub bones: usize,
    pub lods: Vec<LodRec>,
    /// The collision LOD mesh and the physics preset, when named.
    pub collision_mesh: Option<String>,
    pub phys_preset: Option<String>,
    /// bo2mp: the skeleton (`walk::model::xmodel`).
    pub skeleton: Skeleton,
}

/// bo2mp: a model's skeleton, as Black Ops II's `XModel` keeps it: bone
/// names (script strings), the root count, per non-root bone its parent
/// (how many bones back) and its rest pose relative to the parent
/// (quaternion xyzw, stored as halves; translation), per bone its part
/// classification and bind pose in model space (`DObjAnimMat`: unit
/// quaternion xyzw, translation, translation weight).
#[derive(Clone, Debug, Default)]
pub struct Skeleton {
    pub bone_names: Vec<u32>,
    pub num_root_bones: u8,
    pub parent_list: Vec<u8>,
    pub quats: Vec<[f32; 4]>,
    pub trans: Vec<[f32; 3]>,
    pub part_classification: Vec<u8>,
    pub base_mat: Vec<[f32; 8]>,
}

/// bo2mp: one sound alias (`SndAlias`, 0xd8): its list's name, the
/// `assetId` its sound has in the `.sabl`/`.sabs` banks, its file name,
/// its secondary alias, and the record (volumes, distances, flags).
#[derive(Clone, Debug, Default)]
pub struct SoundAliasRec {
    pub name: String,
    pub asset_id: u32,
    pub file: String,
    pub secondary: String,
    pub record: Vec<u8>,
}

/// bo2mp: an `XAnimParts` loaded in the zone: its header (0xf8 bytes) and
/// where its arrays are, as (file offset, bytes).
#[derive(Clone, Debug, Default)]
pub struct AnimRec {
    pub name: String,
    pub header: Vec<u8>,
    pub numframes: u16,
    /// The animated bones' names (script strings).
    pub names: Vec<u32>,
    /// Notetracks: (kind script string: `sound`, `rumble`, `self notify`,
    /// time 0..1, the two parameters' script strings: a `sound` note's
    /// alias first).
    pub notifies: Vec<(u32, f32, u32, u32)>,
    pub data_byte: Option<(usize, usize)>,
    pub data_short: Option<(usize, usize)>,
    pub data_int: Option<(usize, usize)>,
    pub random_data_short: Option<(usize, usize)>,
    pub random_data_byte: Option<(usize, usize)>,
    pub random_data_int: Option<(usize, usize)>,
    /// Key frame indices: u8 each when `numframes < 256`, else u16.
    pub indices: Option<(usize, usize)>,
}

/// One surface of a mesh (`XSurface`).
#[derive(Clone, Debug, Default)]
pub struct SurfaceRec {
    pub name: String,
    pub vert_count: u16,
    pub tri_count: u16,
    /// Where its vertices and indices start in the mesh's shared data.
    pub base_vert: u32,
    pub base_index: u32,
    /// `XSurfaceFlags` (0x2: skinned, the vertices have skin weights).
    pub flags: u16,
    /// Rigid vertex lists (unskinned surfaces): (boneOffset, vertCount,
    /// triOffset, triCount), the vertices in order.
    pub rigid: Vec<[u16; 4]>,
}

/// An `XModelMesh` (one LOD's geometry) loaded in the zone.
#[derive(Clone, Debug, Default)]
pub struct MeshRec {
    pub name: String,
    pub surfaces: Vec<SurfaceRec>,
    /// `XSurfaceShared`: flags (bit 0: streamed), counts, and the byte
    /// offsets of each stream in the data.
    pub shared_flags: u32,
    pub vert_count: u32,
    pub skin_vert_count: u32,
    pub index_count: u32,
    pub data_size: u32,
    /// (positions, vertices, indices, skin weights) offsets in the data.
    pub offsets: [u32; 4],
    /// The vertex/index data in the zone (file offset, bytes), when not streamed.
    pub data: Option<(usize, usize)>,
    /// The mesh's `.xpak` entry (streamed data).
    pub xpak: Option<XPakEntry>,
}

/// A `Material` loaded in the zone.
#[derive(Clone, Debug, Default)]
pub struct MaterialRec {
    pub name: String,
    pub techset: Option<String>,
    /// The texture table: (name hash, image).
    pub textures: Vec<(u32, String)>,
    /// bo2mp: floats 40..48 of the material's first `$Globals` constant
    /// buffer of at least 192 bytes: its colour tint (40..43) and its
    /// specular colour (44..47), what a material without a colour map is
    /// drawn with (measured on core_common's gun materials: a bullet's
    /// brass, a tritium dot's red; 1, 1, 1 where a map carries the colour).
    pub colour_consts: Option<[f32; 8]>,
    /// bo2mp: the material's constant table (`MaterialConstantDef`, 32
    /// bytes each), as stored.
    pub constants: Vec<[u8; 32]>,
}

/// A `GfxImage` loaded in the zone.
#[derive(Clone, Debug, Default)]
pub struct ImageRec {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub levels: u8,
    /// `DXGI_FORMAT`.
    pub format: u32,
    /// Pixels in the zone: (file offset, bytes) when not streamed; the
    /// fallback (lowest mips) of a streamed image likewise.
    pub pixels: Option<(usize, usize)>,
    pub fallback: Option<(usize, usize)>,
    /// `totalSize`: the pixel bytes (all mips).
    pub pixels_len: usize,
    /// The streamed parts' `.xpak` entries (empty when the pixels are in the zone).
    pub xpak: Vec<XPakEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct Capture {
    pub weapons: Vec<WeaponRec>,
    pub models: Vec<ModelRec>,
    pub materials: Vec<MaterialRec>,
    pub images: Vec<ImageRec>,
    pub meshes: Vec<MeshRec>,
    /// bo2mp: animations (`walk::anim`), kept only when `keep_anims` is on.
    pub anims: Vec<AnimRec>,
    /// Also the sound aliases (`walk::sound`): kept with the animations.
    pub keep_anims: bool,
    pub sound_aliases: Vec<SoundAliasRec>,
    /// Also the localized strings: (key, text).
    pub localize: Vec<(String, String)>,
    /// The map assets (`walk::world`).
    pub world: crate::walk::world::WorldCapture,
    /// What the fields of the assets whose walkers keep them name
    /// (attachment uniques, cosmetic variants, weapon camos): (type, name)
    /// -> (field, value), in stream order.
    pub links: HashMap<(AssetType, String), Vec<(String, FieldValue)>>,
    by_name: HashMap<(AssetType, String), usize>,
}

impl Capture {
    pub(crate) fn add_model(&mut self, r: ModelRec) {
        self.by_name
            .insert((AssetType::XModel, r.name.clone()), self.models.len());
        self.models.push(r);
    }
    pub(crate) fn add_mesh(&mut self, r: MeshRec) {
        self.by_name
            .insert((AssetType::XModelMesh, r.name.clone()), self.meshes.len());
        self.meshes.push(r);
    }
    pub(crate) fn add_anim(&mut self, r: AnimRec) {
        self.by_name
            .insert((AssetType::XAnimParts, r.name.clone()), self.anims.len());
        self.anims.push(r);
    }
    /// bo2mp: the animation called `name` (when the walk kept animations).
    pub fn anim(&self, name: &str) -> Option<&AnimRec> {
        self.by_name
            .get(&(AssetType::XAnimParts, name.to_string()))
            .map(|&i| &self.anims[i])
    }
    pub fn mesh(&self, name: &str) -> Option<&MeshRec> {
        self.by_name
            .get(&(AssetType::XModelMesh, name.to_string()))
            .map(|&i| &self.meshes[i])
    }
    pub(crate) fn add_material(&mut self, r: MaterialRec) {
        self.by_name
            .insert((AssetType::Material, r.name.clone()), self.materials.len());
        self.materials.push(r);
    }
    pub(crate) fn add_image(&mut self, r: ImageRec) {
        self.by_name
            .insert((AssetType::Image, r.name.clone()), self.images.len());
        self.images.push(r);
    }
    /// The model called `name` (the last one loaded under that name).
    pub fn model(&self, name: &str) -> Option<&ModelRec> {
        self.by_name
            .get(&(AssetType::XModel, name.to_string()))
            .map(|&i| &self.models[i])
    }
    pub fn material(&self, name: &str) -> Option<&MaterialRec> {
        self.by_name
            .get(&(AssetType::Material, name.to_string()))
            .map(|&i| &self.materials[i])
    }
    pub fn image(&self, name: &str) -> Option<&ImageRec> {
        self.by_name
            .get(&(AssetType::Image, name.to_string()))
            .map(|&i| &self.images[i])
    }
    pub(crate) fn add_links(&mut self, ty: AssetType, name: &str, links: Vec<(String, FieldValue)>) {
        self.links.insert((ty, name.to_string()), links);
    }
    /// What the fields of the asset `ty` `name` name (`links`); empty when
    /// the walk did not keep them.
    pub fn links(&self, ty: AssetType, name: &str) -> &[(String, FieldValue)] {
        self.links.get(&(ty, name.to_string())).map_or(&[], Vec::as_slice)
    }
    /// The weapon whose variant or `WeaponDef` is called `name`.
    pub fn weapon(&self, name: &str) -> Option<&WeaponRec> {
        self.weapons
            .iter()
            .find(|r| r.name == name || r.base_name == name)
    }
}
