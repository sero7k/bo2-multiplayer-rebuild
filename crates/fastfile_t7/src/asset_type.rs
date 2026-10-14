//! `XAssetType` of Black Ops III (T7) PC x64, in the game's order (0x00..0x66).
//!
//! The values follow x64-zt `zonetool/t7/structs.hpp` (and match what the
//! retail zones hold: localize entries in the `en_` zones, rawfiles in
//! `core_post_gfx`, one `sound` and one `keyvaluepairs` per zone, ...).
//! `header_size` is the x64 size of the asset's top-level struct (the
//! bytes the `XAsset` table's `-1` header pointer reads in TEMP), `0` where
//! it is not known yet. Sources:
//! * x64-zt's T7 structs (with their own size asserts): physpreset,
//!   physconstraints, xanim, xmodel, xmodelmesh, material,
//!   computeshaderset, techset, image, clipmap, comworld, gameworld,
//!   map_ents, gfxworld;
//! * read on retail zones (name string right after the header): localize
//!   (16), rawfile (24), scriptparsetree (24), sound (0x220b0: a large
//!   inline part, the four names follow it), ttf (0x20020), fonticon
//!   (0x20), stringtable (0x20), keyvaluepairs (0x18), texturecombo (0x80),
//!   bitfield (0x20), and from "," placeholders lightdescription (0x240),
//!   fx (0x90), tagfx (0x18), vehicle (0xaa8).

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

            /// The name the game's tools use (`rawfile`, `xmodel`, ...).
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
    PhysPreset = 0x00, "physpreset", 0x78;
    PhysConstraints = 0x01, "physconstraints", 0x690;
    DestructibleDef = 0x02, "destructibledef", 0x30;
    XAnimParts = 0x03, "xanim", 0xf8;
    XModel = 0x04, "xmodel", 0x188;
    XModelMesh = 0x05, "xmodelmesh", 0x78;
    Material = 0x06, "material", 0x2a0;
    ComputeShaderSet = 0x07, "computeshaderset", 0x18;
    TechniqueSet = 0x08, "techset", 0x70;
    Image = 0x09, "image", 0x108;
    Sound = 0x0a, "sound", 0x220b0;
    SoundPatch = 0x0b, "sound_patch", 0x0;
    ClipMap = 0x0c, "clipmap", 0x2d0;
    ComWorld = 0x0d, "comworld", 0x68;
    GameWorld = 0x0e, "gameworld", 0x50;
    MapEnts = 0x0f, "map_ents", 0x48;
    GfxWorld = 0x10, "gfxworld", 0x2040;
    LightDef = 0x11, "lightdef", 0x0;
    LensFlareDef = 0x12, "lensflaredef", 0x0;
    UiMap = 0x13, "ui_map", 0x0;
    Font = 0x14, "font", 0x0;
    FontIcon = 0x15, "fonticon", 0x20;
    Localize = 0x16, "localize", 0x10;
    Weapon = 0x17, "weapon", 0x300;
    WeaponDef = 0x18, "weapondef", 0x0;
    WeaponVariant = 0x19, "weaponvariant", 0x0;
    WeaponFull = 0x1a, "weaponfull", 0x0;
    CgMedia = 0x1b, "cgmedia", 0x420;
    PlayerSounds = 0x1c, "playersounds", 0x0;
    PlayerFx = 0x1d, "playerfx", 0x0;
    SharedWeaponSounds = 0x1e, "sharedweaponsounds", 0xf0;
    Attachment = 0x1f, "attachment", 0x0;
    AttachmentUnique = 0x20, "attachmentunique", 0x0;
    WeaponCamo = 0x21, "weaponcamo", 0x18;
    CustomizationTable = 0x22, "customizationtable", 0x28;
    CustomizationTableFeImages = 0x23, "customizationtable_feimages", 0x0;
    CustomizationTableColor = 0x24, "customizationtablecolor", 0x30;
    SndDriverGlobals = 0x25, "snddriverglobals", 0x0;
    Fx = 0x26, "fx", 0x90;
    TagFx = 0x27, "tagfx", 0x18;
    NewLensFlareDef = 0x28, "klf", 0xa8;
    ImpactFx = 0x29, "impactsfxtable", 0x18;
    ImpactSound = 0x2a, "impactsoundstable", 0x20;
    PlayerCharacter = 0x2b, "player_character", 0x0;
    AiType = 0x2c, "aitype", 0x930;
    Character = 0x2d, "character", 0x270;
    XModelAlias = 0x2e, "xmodelalias", 0x198;
    RawFile = 0x2f, "rawfile", 0x18;
    StringTable = 0x30, "stringtable", 0x20;
    StructuredTable = 0x31, "structuredtable", 0x0;
    Leaderboard = 0x32, "leaderboard", 0x30;
    Ddl = 0x33, "ddl", 0x0;
    Glasses = 0x34, "glasses", 0x0;
    TextureList = 0x35, "texturelist", 0x0;
    ScriptParseTree = 0x36, "scriptparsetree", 0x18;
    KeyValuePairs = 0x37, "keyvaluepairs", 0x18;
    VehicleDef = 0x38, "vehicle", 0xaa8;
    AddonMapEnts = 0x39, "addon_map_ents", 0x0;
    Tracer = 0x3a, "tracer", 0x98;
    Slug = 0x3b, "slug", 0x0;
    SurfaceFxTable = 0x3c, "surfacefxtable", 0x148;
    SurfaceSoundDef = 0x3d, "surfacesounddef", 0xa8;
    FootstepTable = 0x3e, "footsteptable", 0x40;
    EntityFxImpacts = 0x3f, "entityfximpacts", 0x218;
    EntitySoundImpacts = 0x40, "entitysoundimpacts", 0x110;
    ZBarrier = 0x41, "zbarrier", 0x0;
    VehicleFxDef = 0x42, "vehiclefxdef", 0x3d8;
    VehicleSoundDef = 0x43, "vehiclesounddef", 0x1000;
    TypeInfo = 0x44, "typeinfo", 0x0;
    ScriptBundle = 0x45, "scriptbundle", 0x30;
    ScriptBundleList = 0x46, "scriptbundlelist", 0x0;
    Rumble = 0x47, "rumble", 0x40;
    BulletPenetration = 0x48, "bulletpenetration", 0x1e8;
    LocDmgTable = 0x49, "locdmgtable", 0x60;
    AimTable = 0x4a, "aimtable", 0x330;
    AnimSelectorTableSet = 0x4b, "animselectortableset", 0x18;
    AnimMappingTable = 0x4c, "animmappingtable", 0x18;
    AnimStateMachine = 0x4d, "animstatemachine", 0x38;
    BehaviorTree = 0x4e, "behaviortree", 0x28;
    BehaviorStateMachine = 0x4f, "behaviorstatemachine", 0x20;
    Ttf = 0x50, "ttf", 0x20020;
    SAnim = 0x51, "sanim", 0x0;
    LightDescription = 0x52, "lightdescription", 0x240;
    Shellshock = 0x53, "shellshock", 0xb0;
    XCam = 0x54, "xcam", 0xa8;
    BgCache = 0x55, "bgcache", 0x18;
    TextureCombo = 0x56, "texturecombo", 0x80;
    FlameTable = 0x57, "flametable", 0x0;
    Bitfield = 0x58, "bitfield", 0x20;
    AttachmentCosmeticVariant = 0x59, "attachmentcosmeticvariant", 0x18;
    MapTable = 0x5a, "maptable", 0x28;
    MapTableLoadingImages = 0x5b, "maptable_loading_images", 0x0;
    Medal = 0x5c, "medal", 0x30;
    MedalTable = 0x5d, "medaltable", 0x18;
    Objective = 0x5e, "objective", 0x70;
    ObjectiveList = 0x5f, "objectivelist", 0x18;
    UmbraTome = 0x60, "umbratome", 0x0;
    NavMesh = 0x61, "navmesh", 0x0;
    NavVolume = 0x62, "navvolume", 0x0;
    BinaryHtml = 0x63, "binaryhtml", 0x0;
    Laser = 0x64, "laser", 0x70;
    Beam = 0x65, "beam", 0x2b0;
    StreamerHint = 0x66, "streamerhint", 0x0;
}

/// One past the last asset type (`ASSET_TYPE_COUNT`).
pub const ASSET_TYPE_COUNT: u32 = 0x67;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_are_dense() {
        assert_eq!(AssetType::ALL.len(), ASSET_TYPE_COUNT as usize);
        for (i, t) in AssetType::ALL.iter().enumerate() {
            assert_eq!(*t as u32, i as u32);
            assert_eq!(AssetType::from_u32(i as u32), Some(*t));
        }
        assert_eq!(AssetType::from_u32(0x17), Some(AssetType::Weapon));
        assert_eq!(AssetType::from_u32(ASSET_TYPE_COUNT), None);
    }
}
