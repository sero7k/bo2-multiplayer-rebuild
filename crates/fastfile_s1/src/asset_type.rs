//! `XAssetType` of Advanced Warfare (S1) PC x64, in the game's order (0x00..0x43).
//!
//! The values and names follow x64-zt `zonetool/s1/structs.hpp`;
//! `header_size` is the x64 size of the asset's top-level struct (`layout.rs`),
//! `0` where the tools have no struct for it.

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
    PhysPreset = 0x00, "physpreset", 96;
    PhysCollmap = 0x01, "physcollmap", 88;
    PhysWaterPreset = 0x02, "physwaterpreset", 128;
    PhysWorld = 0x03, "physworld", 56;
    PhysConstraint = 0x04, "physconstraint", 40;
    XAnimParts = 0x05, "xanim", 216;
    XModelSurfs = 0x06, "xmodelsurfs", 56;
    XModel = 0x07, "xmodel", 680;
    Material = 0x08, "material", 480;
    ComputeShader = 0x09, "computeshader", 32;
    VertexShader = 0x0a, "vertexshader", 32;
    HullShader = 0x0b, "hullshader", 32;
    DomainShader = 0x0c, "domainshader", 32;
    PixelShader = 0x0d, "pixelshader", 32;
    VertexDecl = 0x0e, "vertexdecl", 352;
    TechniqueSet = 0x0f, "techset", 1456;
    Image = 0x10, "image", 104;
    Sound = 0x11, "sound", 32;
    SoundSubmix = 0x12, "soundsubmix", 24;
    SoundCurve = 0x13, "sndcurve", 152;
    LpfCurve = 0x14, "lpfcurve", 152;
    ReverbCurve = 0x15, "reverbcurve", 152;
    SoundContext = 0x16, "sndcontext", 16;
    LoadedSound = 0x17, "loaded_sound", 64;
    ClipMap = 0x18, "clipmap", 640;
    ComWorld = 0x19, "comworld", 40;
    GlassWorld = 0x1a, "glassworld", 16;
    PathData = 0x1b, "pathdata", 408;
    VehicleTrack = 0x1c, "vehicletrack", 0;
    MapEnts = 0x1d, "map_ents", 288;
    FxWorld = 0x1e, "fxworld", 176;
    GfxWorld = 0x1f, "gfxworld", 2800;
    LightDef = 0x20, "lightdef", 48;
    UiMap = 0x21, "uimap", 0;
    Font = 0x22, "font", 40;
    MenuList = 0x23, "menulist", 24;
    Menu = 0x24, "menu", 200;
    AnimClass = 0x25, "animclass", 0;
    LocalizeEntry = 0x26, "localize", 16;
    Attachment = 0x27, "attachment", 120;
    Weapon = 0x28, "weapon", 3560;
    SndDriverGlobals = 0x29, "snddriverglobals", 0;
    Fx = 0x2a, "fx", 64;
    ImpactFx = 0x2b, "impactfx", 16;
    SurfaceFx = 0x2c, "surfacefx", 16;
    AiType = 0x2d, "aitype", 0;
    MpType = 0x2e, "mptype", 0;
    Character = 0x2f, "character", 0;
    XModelAlias = 0x30, "xmodelalias", 0;
    RawFile = 0x31, "rawfile", 24;
    ScriptFile = 0x32, "scriptfile", 40;
    StringTable = 0x33, "stringtable", 24;
    Leaderboard = 0x34, "leaderboard", 0;
    StructuredDataDef = 0x35, "structureddatadef", 24;
    Tracer = 0x36, "tracer", 128;
    Vehicle = 0x37, "vehicle", 2256;
    AddonMapEnts = 0x38, "addon_map_ents", 136;
    NetConstStrings = 0x39, "netconststrings", 32;
    ReverbPreset = 0x3a, "reverbpreset", 0;
    LuaFile = 0x3b, "luafile", 24;
    Scriptable = 0x3c, "scriptable", 64;
    EquipmentSndTable = 0x3d, "equipsndtable", 88;
    VectorField = 0x3e, "vectorfield", 48;
    DopplerPreset = 0x3f, "dopplerpreset", 32;
    ParticleSimAnimation = 0x40, "particlesimanimation", 96;
    Laser = 0x41, "laser", 152;
    SkeletonScript = 0x42, "skeletonscript", 64;
    Clut = 0x43, "clut", 32;
}
