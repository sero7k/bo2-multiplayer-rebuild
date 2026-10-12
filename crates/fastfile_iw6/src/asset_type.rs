//! `XAssetType` of Ghosts (IW6) PC x64, in the game's order (0x00..0x3E).
//!
//! The values and names follow the IW6 tables of the public zone tools
//! (x64-zt `zonetool/iw6/structs.hpp`); every type seen in the retail MP
//! zones read so far (`code_post_gfx_mp`, `common_mp`, `ui_mp`,
//! `mp_character_room`, the `patch_*` zones) is in range. `header_size` is the
//! size of the asset's top-level struct on x64, measured from those headers
//! with an MSVC x64 layout (clang `--target=x86_64-pc-windows-msvc`); `0`
//! where the tools have no struct for it.

macro_rules! asset_types {
    ($($variant:ident = $value:literal, $name:literal, $size:literal;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(u32)]
        pub enum AssetType {
            $($variant = $value,)*
        }

        impl AssetType {
            pub const ALL: &[AssetType] = &[$(AssetType::$variant,)*];

            pub const fn from_u32(value: u32) -> Option<Self> {
                match value {
                    $($value => Some(AssetType::$variant),)*
                    _ => None,
                }
            }

            /// The name the game's tools use (`sound`, `xmodel`, ...).
            pub const fn name(self) -> &'static str {
                match self {
                    $(AssetType::$variant => $name,)*
                }
            }

            /// Size in bytes of the asset's top-level struct (x64), `0` when unknown.
            pub const fn header_size(self) -> usize {
                match self {
                    $(AssetType::$variant => $size,)*
                }
            }
        }
    };
}

asset_types! {
    PhysPreset = 0x00, "physpreset", 88;
    PhysCollmap = 0x01, "physcollmap", 88;
    XAnimParts = 0x02, "xanim", 136;
    XModelSurfs = 0x03, "xmodelsurfs", 56;
    XModel = 0x04, "xmodel", 600;
    Material = 0x05, "material", 936;
    ComputeShader = 0x06, "computeshader", 32;
    VertexShader = 0x07, "vertexshader", 40;
    HullShader = 0x08, "hullshader", 32;
    DomainShader = 0x09, "domainshader", 32;
    PixelShader = 0x0A, "pixelshader", 40;
    VertexDecl = 0x0B, "vertexdecl", 2104;
    TechniqueSet = 0x0C, "techset", 3312;
    Image = 0x0D, "image", 104;
    Sound = 0x0E, "sound", 24;
    SoundCurve = 0x0F, "sndcurve", 152;
    LpfCurve = 0x10, "lpfcurve", 152;
    ReverbCurve = 0x11, "reverbcurve", 152;
    LoadedSound = 0x12, "loaded_sound", 56;
    ClipMap = 0x13, "clipmap", 640;
    ComWorld = 0x14, "comworld", 40;
    GlassWorld = 0x15, "glassworld", 16;
    PathData = 0x16, "pathdata", 136;
    VehicleTrack = 0x17, "vehicletrack", 0;
    MapEnts = 0x18, "map_ents", 256;
    FxWorld = 0x19, "fxworld", 176;
    GfxWorld = 0x1A, "gfxworld", 2608;
    LightDef = 0x1B, "lightdef", 48;
    UiMap = 0x1C, "uimap", 0;
    Font = 0x1D, "font", 40;
    MenuList = 0x1E, "menulist", 24;
    Menu = 0x1F, "menu", 200;
    AnimClass = 0x20, "animclass", 88;
    LocalizeEntry = 0x21, "localize", 16;
    Attachment = 0x22, "attachment", 280;
    Weapon = 0x23, "weapon", 296;
    SndDriverGlobals = 0x24, "snddriverglobals", 0;
    Fx = 0x25, "fx", 64;
    ImpactFx = 0x26, "impactfx", 16;
    SurfaceFx = 0x27, "surfacefx", 16;
    AiType = 0x28, "aitype", 0;
    MpType = 0x29, "mptype", 0;
    Character = 0x2A, "character", 0;
    XModelAlias = 0x2B, "xmodelalias", 0;
    RawFile = 0x2C, "rawfile", 24;
    ScriptFile = 0x2D, "scriptfile", 40;
    StringTable = 0x2E, "stringtable", 24;
    Leaderboard = 0x2F, "leaderboard", 0;
    StructuredDataDef = 0x30, "structureddatadef", 24;
    Tracer = 0x31, "tracer", 120;
    Vehicle = 0x32, "vehicle", 1552;
    AddonMapEnts = 0x33, "addon_map_ents", 104;
    NetConstStrings = 0x34, "netconststrings", 32;
    ReverbPreset = 0x35, "reverbpreset", 0;
    LuaFile = 0x36, "luafile", 24;
    Scriptable = 0x37, "scriptable", 64;
    Colorization = 0x38, "colorization", 48;
    ColorizationSet = 0x39, "colorizationset", 104;
    ToneMapping = 0x3A, "tonemapping", 112;
    EquipmentSndTable = 0x3B, "equipsndtable", 88;
    VectorField = 0x3C, "vectorfield", 80;
    DopplerPreset = 0x3D, "dopplerpreset", 32;
    ParticleSimAnimation = 0x3E, "particlesimanimation", 96;
}

/// `ASSET_TYPE_COUNT`.
pub const ASSET_TYPE_COUNT: u32 = 0x3F;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_dense_and_named() {
        assert_eq!(AssetType::ALL.len(), ASSET_TYPE_COUNT as usize);
        for (i, ty) in AssetType::ALL.iter().enumerate() {
            assert_eq!(*ty as u32, i as u32);
            assert_eq!(AssetType::from_u32(i as u32), Some(*ty));
            assert!(!ty.name().is_empty());
        }
        assert_eq!(AssetType::from_u32(ASSET_TYPE_COUNT), None);
        assert_eq!(AssetType::Sound.name(), "sound");
        assert_eq!(AssetType::Sound.header_size(), 24);
        assert_eq!(AssetType::NetConstStrings as u32, 0x34);
        assert_eq!(AssetType::Weapon as u32, 0x23);
    }
}
