//! `XAssetType` of Infinite Warfare (IW7) PC x64, in the game's order (0x00..0x4f).
//!
//! The values and names follow x64-zt `zonetool/iw7/structs.hpp`;
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
    PhysicsLibrary = 0x00, "physicslibrary", 24;
    PhysicsSfxEventAsset = 0x01, "physics_sfx_event_asset", 32;
    PhysicsVfxEventAsset = 0x02, "physics_vfx_event_asset", 32;
    PhysicsAsset = 0x03, "physicsasset", 80;
    PhysicsFxPipeline = 0x04, "physics_fx_pipeline", 32;
    PhysicsFxShape = 0x05, "physics_fx_shape", 24;
    XAnimParts = 0x06, "xanimparts", 136;
    XModelSurfs = 0x07, "xmodel_surfs", 56;
    XModel = 0x08, "xmodel", 736;
    Mayhem = 0x09, "mayhem", 80;
    Material = 0x0a, "material", 280;
    ComputeShader = 0x0b, "computeshader", 40;
    VertexShader = 0x0c, "vertexshader", 40;
    HullShader = 0x0d, "hullshader", 40;
    DomainShader = 0x0e, "domainshader", 40;
    PixelShader = 0x0f, "pixelshader", 40;
    VertexDecl = 0x10, "vertexdecl", 2280;
    TechniqueSet = 0x11, "technique_set", 56;
    Image = 0x12, "image", 112;
    SoundGlobals = 0x13, "sound_globals", 208;
    SoundBank = 0x14, "sound_bank", 352;
    SoundBankTransient = 0x15, "sound_bank_transient", 24;
    ClipMap = 0x16, "clipmap", 384;
    ComWorld = 0x17, "comworld", 104;
    GlassWorld = 0x18, "glassworld", 16;
    PathData = 0x19, "pathdata", 144;
    NavMesh = 0x1a, "navmesh", 144;
    MapEnts = 0x1b, "map_ents", 832;
    FxWorld = 0x1c, "fxworld", 176;
    GfxWorld = 0x1d, "gfxworld", 4520;
    GfxWorldTransientZone = 0x1e, "gfxworld_transient_zone", 80;
    IesProfile = 0x1f, "iesprofile", 0;
    LightDef = 0x20, "light_def", 40;
    UiMap = 0x21, "ui_map", 0;
    AnimClass = 0x22, "animclass", 88;
    PlayerAnim = 0x23, "playeranim", 176;
    Gesture = 0x24, "gesture", 168;
    LocalizeEntry = 0x25, "localize_entry", 16;
    Attachment = 0x26, "attachment", 536;
    Weapon = 0x27, "weapon", 312;
    Vfx = 0x28, "vfx", 128;
    Fx = 0x29, "fx", 64;
    ImpactFx = 0x2a, "impact_fx", 16;
    SurfaceFx = 0x2b, "surface_fx", 16;
    AiType = 0x2c, "aitype", 0;
    MpType = 0x2d, "mptype", 0;
    Character = 0x2e, "character", 0;
    XModelAlias = 0x2f, "xmodelalias", 0;
    RawFile = 0x30, "rawfile", 24;
    ScriptFile = 0x31, "scriptfile", 40;
    StringTable = 0x32, "stringtable", 24;
    Leaderboard = 0x33, "leaderboard", 56;
    VirtualLeaderboard = 0x34, "virtual_leaderboard", 48;
    StructuredDataDef = 0x35, "structured_data_def", 24;
    Ddl = 0x36, "ddl", 16;
    Tracer = 0x37, "tracer", 128;
    Vehicle = 0x38, "vehicle", 2704;
    AddonMapEnts = 0x39, "addon_map_ents", 136;
    NetConstStrings = 0x3a, "net_const_strings", 32;
    LuaFile = 0x3b, "lua_file", 24;
    Scriptable = 0x3c, "scriptable", 112;
    EquipmentSndTable = 0x3d, "equipment_snd_table", 112;
    VectorField = 0x3e, "vectorfield", 48;
    ParticleSimAnimation = 0x3f, "particle_sim_animation", 120;
    StreamingInfo = 0x40, "streaming_info", 192;
    Laser = 0x41, "laser", 184;
    Ttf = 0x42, "ttf", 32;
    Suit = 0x43, "suit", 704;
    SuitAnimPackage = 0x44, "suitanimpackage", 3848;
    SpaceshipTarget = 0x45, "spaceshiptarget", 136;
    Rumble = 0x46, "rumble", 40;
    RumbleGraph = 0x47, "rumble_graph", 144;
    AnimPackage = 0x48, "anim_package", 64;
    SfxPackage = 0x49, "sfx_package", 56;
    VfxPackage = 0x4a, "vfx_package", 16;
    BehaviorTree = 0x4b, "behavior_tree", 24;
    XAnimArchetype = 0x4c, "xanim_archetype", 0;
    XAnimProceduralBones = 0x4d, "xanim_proceduralbones", 80;
    Reticle = 0x4e, "reticle", 32;
    GfxLightMap = 0x4f, "gfxlightmap", 32;
}
