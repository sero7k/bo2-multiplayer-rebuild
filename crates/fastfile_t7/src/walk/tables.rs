//! Game-data assets that are one struct of strings and asset pointers:
//! the name first, then each pointer field in offset order (an asset
//! loaded where its pointer is, a string where its pointer is). The fields
//! as HydraX (Scobalula) reads them from the running game; the header
//! sizes measured on retail `mp_common` (the name follows the header).
//! Every other 8-byte field must not load data (`no_loads`), so a field
//! these tables miss stops the walk at that field.

use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::Stream;

#[derive(Clone, Copy, Debug)]
pub enum F {
    Str,
    A(AssetType),
    /// A `-2` pointer to runtime data: no file bytes and no VIRTUAL slot.
    Runtime,
    /// An array of records without pointers: (u32 count field, record size).
    Records(usize, usize),
}
use F::{A, Records, Runtime, Str};
use AssetType as T;

pub struct Spec {
    pub ty: AssetType,
    pub size: usize,
    pub fields: &'static [(usize, F)],
}

pub const SPECS: &[Spec] = &[
    // Read on mp_common: two images load here (grenade icons); its other
    // fields are references to assets loaded earlier.
    Spec { ty: T::CgMedia, size: 0x420, fields: &[(0x228, A(T::Image)), (0x230, A(T::Image))] },
    // Read on mp_common: name and numbers only.
    Spec { ty: T::LocDmgTable, size: 0x60, fields: &[] },
    Spec { ty: T::BulletPenetration, size: 0x1e8, fields: &[] },
    // Read on mp_common: a category string, then four images (`-2`; the
    // later medals name the first one's last two by reference).
    Spec { ty: T::Medal, size: 0x30, fields: &[(0x8, Str), (0x10, A(T::Image)), (0x18, A(T::Image)), (0x20, A(T::Image)), (0x28, A(T::Image))] },
    // Read on mp_common (no HydraX table): an image (`-2`), a localized
    // string, then u32 count
    // at 0x60 records of 24 at 0x68.
    Spec { ty: T::Objective, size: 0x70, fields: &[(0x30, A(T::Image)), (0x38, Str), (0x68, Records(0x60, 24))] },
    // Read on mp_common (no HydraX table): 0x218 is `-2` and loads no file
    // bytes and takes no VIRTUAL slot (runtime data).
    Spec { ty: T::LightDescription, size: 0x240, fields: &[(0x218, Runtime)] },
    Spec { ty: T::SurfaceSoundDef, size: 0xa8, fields: &[] },
    Spec { ty: T::FootstepTable, size: 0x40, fields: &[(0x8, A(T::SurfaceSoundDef)), (0x10, A(T::SurfaceSoundDef)), (0x18, A(T::SurfaceSoundDef)), (0x20, A(T::SurfaceSoundDef)), (0x28, A(T::SurfaceSoundDef)), (0x30, A(T::SurfaceSoundDef)), (0x38, A(T::SurfaceSoundDef))] },
    Spec { ty: T::SurfaceFxTable, size: 0x148, fields: &[(0x8, A(T::Fx)), (0x10, A(T::Fx)), (0x18, A(T::Fx)), (0x20, A(T::Fx)), (0x28, A(T::Fx)), (0x30, A(T::Fx)), (0x38, A(T::Fx)), (0x40, A(T::Fx)), (0x48, A(T::Fx)), (0x50, A(T::Fx)), (0x58, A(T::Fx)), (0x60, A(T::Fx)), (0x68, A(T::Fx)), (0x70, A(T::Fx)), (0x78, A(T::Fx)), (0x80, A(T::Fx)), (0x88, A(T::Fx)), (0x90, A(T::Fx)), (0x98, A(T::Fx)), (0xa0, A(T::Fx)), (0xa8, A(T::Fx)), (0xb0, A(T::Fx)), (0xb8, A(T::Fx)), (0xc0, A(T::Fx)), (0xc8, A(T::Fx)), (0xd0, A(T::Fx)), (0xd8, A(T::Fx)), (0xe0, A(T::Fx)), (0xe8, A(T::Fx)), (0xf0, A(T::Fx)), (0xf8, A(T::Fx)), (0x100, A(T::Fx)), (0x108, A(T::Fx)), (0x110, A(T::Fx)), (0x118, A(T::Fx)), (0x120, A(T::Fx)), (0x128, A(T::Fx)), (0x130, A(T::Fx)), (0x138, A(T::Fx)), (0x140, A(T::Fx))] },
    Spec { ty: T::ImpactFx, size: 0x18, fields: &[(0x8, A(T::SurfaceFxTable)), (0x10, A(T::EntityFxImpacts))] },
    Spec { ty: T::ImpactSound, size: 0x20, fields: &[(0x8, A(T::SurfaceSoundDef)), (0x10, A(T::EntitySoundImpacts)), (0x18, A(T::EntitySoundImpacts))] },
    Spec { ty: T::EntityFxImpacts, size: 0x218, fields: &[(0x38, A(T::Fx)), (0x40, A(T::Fx)), (0x48, A(T::Fx)), (0x50, A(T::Fx)), (0x58, A(T::Fx)), (0x60, A(T::Fx)), (0x68, A(T::Fx)), (0x70, A(T::Fx)), (0x78, A(T::Fx)), (0x80, A(T::Fx)), (0x88, A(T::Fx)), (0x90, A(T::Fx)), (0x98, A(T::Fx)), (0xa0, A(T::Fx)), (0xa8, A(T::Fx)), (0xb0, A(T::Fx)), (0xb8, A(T::Fx)), (0xc0, A(T::Fx)), (0xc8, A(T::Fx)), (0xd0, A(T::Fx)), (0xd8, A(T::Fx)), (0xe0, A(T::Fx)), (0xe8, A(T::Fx)), (0xf0, A(T::Fx)), (0xf8, A(T::Fx)), (0x100, A(T::Fx)), (0x108, A(T::Fx)), (0x110, A(T::Fx)), (0x118, A(T::Fx)), (0x120, A(T::Fx)), (0x128, A(T::Fx)), (0x130, A(T::Fx)), (0x138, A(T::Fx)), (0x140, A(T::Fx)), (0x148, A(T::Fx)), (0x150, A(T::Fx)), (0x158, A(T::Fx)), (0x160, A(T::Fx)), (0x168, A(T::Fx)), (0x170, A(T::Fx)), (0x178, A(T::Fx)), (0x180, A(T::Fx)), (0x188, A(T::Fx)), (0x190, A(T::Fx)), (0x198, A(T::Fx)), (0x1a0, A(T::Fx)), (0x1a8, A(T::Fx)), (0x1b0, A(T::Fx)), (0x1b8, A(T::Fx)), (0x1c0, A(T::Fx)), (0x1c8, A(T::Fx)), (0x1d0, A(T::Fx)), (0x1d8, A(T::Fx)), (0x1e0, A(T::Fx)), (0x1e8, A(T::Fx)), (0x1f0, A(T::Fx)), (0x1f8, A(T::Fx)), (0x200, A(T::Fx)), (0x208, A(T::Fx)), (0x210, A(T::Fx))] },
    Spec { ty: T::EntitySoundImpacts, size: 0x110, fields: &[] },
    Spec { ty: T::Laser, size: 0x70, fields: &[(0x38, A(T::Fx))] },
    Spec { ty: T::Beam, size: 0x2b0, fields: &[(0x270, A(T::Fx)), (0x278, A(T::Fx)), (0x280, A(T::Fx)), (0x288, A(T::Fx)), (0x290, A(T::Fx)), (0x298, Str), (0x2a0, Str)] },
    Spec { ty: T::Tracer, size: 0x98, fields: &[] },
    Spec { ty: T::PhysPreset, size: 0x78, fields: &[(0x28, Str), (0x60, A(T::Fx)), (0x68, A(T::ImpactFx)), (0x70, A(T::ImpactSound))] },
    Spec { ty: T::Shellshock, size: 0xb0, fields: &[(0x40, Str), (0x48, Str), (0x90, Str)] },
    Spec { ty: T::SharedWeaponSounds, size: 0xf0, fields: &[(0x8, A(T::SurfaceSoundDef)), (0x10, A(T::SurfaceSoundDef)), (0x18, A(T::EntitySoundImpacts)), (0x20, A(T::EntitySoundImpacts)), (0x28, A(T::EntitySoundImpacts)), (0x30, A(T::EntitySoundImpacts)), (0x38, A(T::EntitySoundImpacts)), (0x40, A(T::EntitySoundImpacts)), (0x48, A(T::EntitySoundImpacts)), (0x50, A(T::EntitySoundImpacts)), (0x58, A(T::EntitySoundImpacts)), (0x60, A(T::EntitySoundImpacts)), (0x68, A(T::EntitySoundImpacts)), (0x70, A(T::EntitySoundImpacts)), (0x78, A(T::EntitySoundImpacts))] },
    Spec { ty: T::VehicleFxDef, size: 0x3d8, fields: &[(0x8, Str), (0x18, A(T::Fx)), (0x20, A(T::Fx)), (0x28, A(T::Fx)), (0x30, A(T::Fx)), (0x38, A(T::Fx)), (0x40, A(T::Fx)), (0x48, A(T::Fx)), (0x50, A(T::Fx)), (0x58, A(T::Fx)), (0x60, A(T::Fx)), (0x68, A(T::Fx)), (0x70, A(T::Fx)), (0x78, A(T::Fx)), (0x80, A(T::Fx)), (0x88, A(T::Fx)), (0x90, A(T::Fx)), (0x98, A(T::Fx)), (0xa0, A(T::Fx)), (0xa8, A(T::Fx)), (0xb0, A(T::Fx)), (0xb8, A(T::Fx)), (0xc0, A(T::Fx)), (0xc8, A(T::Fx)), (0xd0, A(T::Fx)), (0xd8, A(T::Fx)), (0xe0, A(T::Fx)), (0xe8, A(T::Fx)), (0xf0, A(T::Fx)), (0xf8, A(T::Fx)), (0x118, A(T::Fx)), (0x120, A(T::Fx)), (0x128, A(T::Fx)), (0x130, A(T::Fx)), (0x138, A(T::Fx)), (0x140, A(T::Fx)), (0x148, A(T::Fx)), (0x150, A(T::Fx)), (0x158, A(T::Fx)), (0x160, A(T::Fx)), (0x168, A(T::Fx)), (0x170, A(T::Fx)), (0x178, A(T::Fx)), (0x180, A(T::Fx)), (0x188, A(T::Fx)), (0x190, A(T::Fx)), (0x198, A(T::Fx)), (0x1a0, A(T::Fx)), (0x1a8, A(T::Fx)), (0x1b0, A(T::Fx)), (0x1b8, A(T::Fx)), (0x1c0, A(T::Fx)), (0x1c8, A(T::Fx)), (0x1d0, A(T::Fx)), (0x1d8, A(T::Fx)), (0x1e0, A(T::Fx)), (0x1e8, A(T::Fx)), (0x1f0, A(T::Fx)), (0x1f8, A(T::Fx)), (0x200, A(T::Fx)), (0x208, A(T::Fx)), (0x210, A(T::Fx)), (0x218, A(T::Fx)), (0x220, A(T::Fx)), (0x228, A(T::Fx)), (0x230, A(T::Fx)), (0x238, A(T::Fx)), (0x258, A(T::Fx)), (0x260, A(T::Fx)), (0x268, A(T::Fx)), (0x270, A(T::Fx)), (0x278, A(T::Fx)), (0x280, A(T::Fx)), (0x288, A(T::Fx)), (0x290, A(T::Fx)), (0x298, A(T::Fx)), (0x2a0, A(T::Fx)), (0x2a8, A(T::Fx)), (0x2b0, A(T::Fx)), (0x2b8, A(T::Fx)), (0x2c0, A(T::Fx)), (0x2c8, A(T::Fx)), (0x2d0, A(T::Fx)), (0x2d8, A(T::Fx)), (0x2e0, A(T::Fx)), (0x2e8, A(T::Fx)), (0x2f0, A(T::Fx)), (0x2f8, A(T::Fx)), (0x300, A(T::Fx)), (0x308, A(T::Fx)), (0x310, A(T::Fx)), (0x318, A(T::Fx)), (0x320, A(T::Fx)), (0x328, A(T::Fx)), (0x330, A(T::Fx)), (0x338, A(T::Fx)), (0x340, A(T::Fx)), (0x348, A(T::Fx)), (0x350, A(T::Fx)), (0x358, A(T::Fx)), (0x360, A(T::Fx)), (0x368, A(T::Fx)), (0x370, A(T::Fx)), (0x378, A(T::Fx)), (0x398, A(T::Fx)), (0x3a0, A(T::Fx)), (0x3a8, A(T::Fx)), (0x3b0, A(T::Fx)), (0x3b8, A(T::Fx)), (0x3c0, A(T::Fx)), (0x3c8, A(T::Fx)), (0x3d0, A(T::Fx))] },
    Spec { ty: T::VehicleSoundDef, size: 0x1000, fields: &[(0x58, Str), (0x80, Str), (0xa8, Str), (0xd0, Str), (0xf8, Str), (0x120, Str), (0x148, Str), (0x170, Str), (0x198, Str), (0x1c0, Str), (0x270, Str), (0x290, Str), (0x2b0, Str), (0x2c8, Str), (0x2e8, Str), (0x300, Str), (0x318, Str), (0x330, Str), (0x348, Str), (0x360, Str), (0x378, Str), (0x390, Str), (0x3a8, Str), (0x3c0, Str), (0x3d8, Str), (0x3f0, Str), (0x408, Str), (0x420, Str), (0x438, Str), (0x450, Str), (0x468, Str), (0x480, Str), (0x498, Str), (0x4b0, Str), (0x4c8, Str), (0x4e0, Str), (0x4f8, Str), (0x510, Str), (0x528, Str), (0x540, Str), (0x558, Str), (0x570, Str), (0x588, Str), (0x5a0, Str), (0x5b8, Str), (0x5d0, Str), (0x5e8, Str), (0x600, Str), (0x618, Str), (0x630, Str), (0x648, Str), (0x660, Str), (0x678, Str), (0x690, Str), (0x6a0, Str), (0x6c8, Str), (0x6f0, Str), (0x718, Str), (0x740, Str), (0x768, Str), (0x790, Str), (0x7b8, Str), (0x7e0, Str), (0x808, Str), (0x830, Str), (0x858, Str), (0x880, Str), (0x8a8, Str), (0x8d0, Str), (0x8f8, Str), (0x920, Str), (0x948, Str), (0x970, Str), (0x998, Str), (0x9c0, Str), (0x9d0, Str), (0x9e0, Str), (0x9f0, Str), (0xa00, Str), (0xa10, Str), (0xa20, Str), (0xa30, Str), (0xa40, Str), (0xa50, Str), (0xa60, Str), (0xa70, Str), (0xa80, Str), (0xa90, Str), (0xaa0, Str), (0xab0, Str), (0xac0, Str), (0xad0, Str), (0xae0, Str), (0xaf0, Str), (0xb00, Str), (0xb10, Str), (0xb20, Str), (0xb30, Str), (0xb40, Str), (0xb50, Str), (0xb60, Str), (0xb70, Str), (0xb80, Str), (0xb90, Str), (0xba0, Str), (0xbb0, Str), (0xbc0, Str), (0xbd0, Str), (0xbe0, Str), (0xbf0, Str), (0xc00, Str), (0xc10, Str), (0xc20, Str), (0xc30, Str), (0xc40, Str), (0xc50, Str), (0xc60, Str), (0xc70, Str), (0xc80, Str), (0xc90, Str), (0xca0, Str), (0xcb0, Str), (0xcc0, Str), (0xcd0, Str), (0xce0, Str), (0xcf0, Str), (0xd00, Str), (0xd10, Str), (0xd20, Str), (0xd30, Str), (0xd40, Str), (0xd50, Str), (0xd60, Str), (0xd70, Str), (0xd80, Str), (0xd90, Str), (0xda0, Str), (0xdb0, Str), (0xdc0, Str), (0xdd0, Str), (0xde0, Str), (0xdf0, Str), (0xe00, Str), (0xe10, Str), (0xe20, Str), (0xe30, Str), (0xe40, Str), (0xe50, Str), (0xe60, Str), (0xe70, Str), (0xe80, Str), (0xe90, Str), (0xea0, Str), (0xeb0, Str), (0xec0, Str), (0xed0, Str), (0xee0, Str), (0xef0, Str), (0xf00, Str), (0xf10, Str), (0xf20, Str), (0xf30, Str), (0xf40, Str), (0xf80, Str), (0xf90, Str), (0xfa0, Str), (0xfb0, Str), (0xfc0, Str), (0xfd0, Str), (0xfe0, Str), (0xff0, Str)] },
    Spec { ty: T::XModelAlias, size: 0x198, fields: &[(0x8, A(T::XModel)), (0x10, A(T::XModel)), (0x18, A(T::XModel)), (0x20, A(T::XModel)), (0x28, A(T::XModel)), (0x30, A(T::XModel)), (0x38, A(T::XModel)), (0x40, A(T::XModel)), (0x48, A(T::XModel)), (0x50, A(T::XModel)), (0x58, A(T::XModel)), (0x60, A(T::XModel)), (0x68, A(T::XModel)), (0x70, A(T::XModel)), (0x78, A(T::XModel)), (0x80, A(T::XModel)), (0x88, A(T::XModel)), (0x90, A(T::XModel)), (0x98, A(T::XModel)), (0xa0, A(T::XModel)), (0xa8, A(T::XModel)), (0xb0, A(T::XModel)), (0xb8, A(T::XModel)), (0xc0, A(T::XModel)), (0xc8, A(T::XModel)), (0xd0, A(T::XModel)), (0xd8, A(T::XModel)), (0xe0, A(T::XModel)), (0xe8, A(T::XModel)), (0xf0, A(T::XModel)), (0xf8, A(T::XModel)), (0x100, A(T::XModel)), (0x108, A(T::XModel)), (0x110, A(T::XModel)), (0x118, A(T::XModel)), (0x120, A(T::XModel)), (0x128, A(T::XModel)), (0x130, A(T::XModel)), (0x138, A(T::XModel)), (0x140, A(T::XModel)), (0x148, A(T::XModel)), (0x150, A(T::XModel)), (0x158, A(T::XModel)), (0x160, A(T::XModel)), (0x168, A(T::XModel)), (0x170, A(T::XModel)), (0x178, A(T::XModel)), (0x180, A(T::XModel)), (0x188, A(T::XModel)), (0x190, A(T::XModel))] },
    Spec { ty: T::Character, size: 0x270, fields: &[(0x10, A(T::ScriptBundle)), (0x18, A(T::ScriptBundle)), (0x20, A(T::ScriptBundle)), (0x28, A(T::ScriptBundle)), (0x30, A(T::ScriptBundle)), (0x38, A(T::ScriptBundle)), (0x40, A(T::ScriptBundle)), (0x48, A(T::ScriptBundle)), (0x50, A(T::ScriptBundle)), (0x58, A(T::ScriptBundle)), (0x60, A(T::XModel)), (0x68, A(T::XModelAlias)), (0x70, A(T::XModel)), (0x78, A(T::XModelAlias)), (0x80, A(T::XModelAlias)), (0x88, A(T::XModel)), (0x90, A(T::XModelAlias)), (0x98, A(T::XModel)), (0xa0, A(T::XModelAlias)), (0xa8, A(T::XModel)), (0xb0, A(T::XModel)), (0xb8, A(T::XModel)), (0xc0, A(T::XModelAlias)), (0xc8, A(T::XModel)), (0xd0, A(T::XModelAlias)), (0xd8, A(T::XModel)), (0xe0, A(T::XModelAlias)), (0xe8, A(T::XModel)), (0xf0, A(T::XModelAlias)), (0xf8, A(T::XModel)), (0x100, A(T::XModelAlias)), (0x108, A(T::XModel)), (0x110, A(T::XModelAlias)), (0x118, A(T::XModel)), (0x120, A(T::XModelAlias)), (0x128, A(T::XModel)), (0x130, A(T::XModelAlias)), (0x138, A(T::XModel)), (0x140, A(T::XModelAlias)), (0x148, A(T::XModel)), (0x158, A(T::XModel)), (0x168, A(T::XModel)), (0x178, A(T::XModel)), (0x188, A(T::XModel)), (0x198, A(T::XModel)), (0x1a8, A(T::XModel)), (0x1b8, A(T::XModel)), (0x1c8, A(T::XModel)), (0x1d8, A(T::XModel)), (0x1e8, A(T::XModel)), (0x1f8, A(T::XModel)), (0x208, A(T::XModel)), (0x218, A(T::XModel)), (0x228, A(T::XModel)), (0x238, A(T::XModel)), (0x248, A(T::XModel)), (0x260, Str)] },
    Spec { ty: T::AiType, size: 0x930, fields: &[(0x8, Str), (0x10, Str), (0x360, A(T::FootstepTable)), (0x370, A(T::SurfaceFxTable)), (0x4b8, A(T::Character)), (0x4c0, A(T::Character)), (0x4c8, A(T::Character)), (0x4d0, A(T::Character)), (0x4d8, A(T::Character)), (0x4e0, A(T::Character)), (0x4e8, A(T::Character)), (0x4f0, A(T::Character)), (0x508, A(T::AnimStateMachine)), (0x510, A(T::AnimSelectorTableSet)), (0x520, A(T::AnimMappingTable)), (0x528, A(T::AnimMappingTable)), (0x530, A(T::AnimMappingTable)), (0x538, A(T::AimTable)), (0x540, A(T::ScriptBundle)), (0x548, A(T::ScriptBundle)), (0x550, A(T::ScriptBundle)), (0x558, A(T::ScriptBundle)), (0x560, A(T::ScriptBundle)), (0x568, A(T::ScriptBundle)), (0x570, A(T::ScriptBundle)), (0x580, A(T::ScriptBundle)), (0x818, Str), (0x820, A(T::Image))] },
    Spec { ty: T::VehicleDef, size: 0xaa8, fields: &[(0x220, Str), (0x258, Str), (0x260, Str), (0x268, Str), (0x270, Str), (0x340, Str), (0x348, Str), (0x350, Str), (0x358, Str), (0x360, Str), (0x368, Str), (0x370, Str), (0x398, Str), (0x3a0, Str), (0x3a8, Str), (0x3c0, Str), (0x3d0, A(T::VehicleSoundDef)), (0x3e0, Str), (0x470, A(T::XModel)), (0x478, A(T::XModel)), (0x480, A(T::XModel)), (0x488, A(T::XModel)), (0x4a0, A(T::XModel)), (0x4a8, A(T::XModel)), (0x4b0, A(T::XModel)), (0x4b8, A(T::XModel)), (0x508, A(T::XModel)), (0x510, A(T::XModel)), (0x518, A(T::XModel)), (0x520, A(T::XModel)), (0x530, A(T::Fx)), (0x540, A(T::SurfaceFxTable)), (0x548, A(T::Fx)), (0x558, Str), (0x560, A(T::Fx)), (0x568, A(T::Fx)), (0x570, A(T::Fx)), (0x578, A(T::Fx)), (0x5a0, Str), (0x5b8, Str), (0x600, Str), (0x618, Str), (0x628, Str), (0x638, Str), (0x648, Str), (0x658, Str), (0x668, Str), (0x678, Str), (0x688, Str), (0x698, Str), (0x6a8, Str), (0x6b8, Str), (0x6c8, Str), (0x6d8, Str), (0x708, A(T::Rumble)), (0x718, A(T::Rumble)), (0x720, A(T::Rumble)), (0x9d8, A(T::ScriptBundle)), (0x9e0, A(T::ScriptBundle)), (0x9e8, A(T::ScriptBundle)), (0x9f0, A(T::ScriptBundle)), (0x9f8, A(T::AnimStateMachine)), (0xa00, A(T::AnimSelectorTableSet)), (0xa08, A(T::AnimMappingTable)), (0xa60, Str), (0xa80, A(T::SurfaceSoundDef)), (0xa88, A(T::SurfaceFxTable)), (0xa90, A(T::DestructibleDef)), (0xa98, A(T::Image))] },
];

/// 8-byte words HydraX knows to hold numbers (ints that may be -1, like a
/// vehicle's seat switch order), not pointers: left out of `no_loads`.
pub const DATA_WORDS: &[(AssetType, &[usize])] = &[
    (T::VehicleDef, &[0x8, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80, 0x88, 0x90, 0x98, 0xa0, 0xa8, 0xb0, 0xb8, 0xc0, 0xc8, 0xd0, 0xd8, 0xe0, 0xe8, 0xf0, 0xf8, 0x100, 0x108, 0x110, 0x118, 0x120, 0x128, 0x130, 0x138, 0x140, 0x148, 0x150, 0x158, 0x160, 0x168, 0x170, 0x178, 0x180, 0x188, 0x190, 0x198, 0x1a0, 0x1a8, 0x1b0, 0x1b8, 0x1c0, 0x1c8, 0x1d0, 0x1d8, 0x1e0, 0x1e8, 0x1f0, 0x1f8, 0x200, 0x208, 0x210, 0x218, 0x228, 0x230, 0x238, 0x240, 0x248, 0x250, 0x280, 0x288, 0x2a0, 0x2a8, 0x2c0, 0x2c8, 0x2e0, 0x2e8, 0x300, 0x308, 0x320, 0x328, 0x3b0, 0x3b8, 0x3c8, 0x3d8, 0x3e8, 0x3f0, 0x3f8, 0x400, 0x408, 0x410, 0x418, 0x420, 0x428, 0x430, 0x438, 0x440, 0x448, 0x450, 0x458, 0x460, 0x468, 0x490, 0x498, 0x4c0, 0x4c8, 0x4d0, 0x4d8, 0x4e0, 0x4e8, 0x4f0, 0x4f8, 0x500, 0x528, 0x538, 0x550, 0x580, 0x588, 0x590, 0x598, 0x5a8, 0x5b0, 0x5c0, 0x5c8, 0x5d0, 0x5d8, 0x5e0, 0x5e8, 0x5f0, 0x5f8, 0x610, 0x6c0, 0x6e8, 0x6f0, 0x6f8, 0x700, 0x710, 0x728, 0x730, 0x738, 0x740, 0x748, 0x750, 0x758, 0x760, 0x768, 0x770, 0x778, 0x780, 0x788, 0x790, 0x798, 0x7a0, 0x7a8, 0x7b0, 0x7b8, 0x7c0, 0x7c8, 0x7d0, 0x7d8, 0x7e0, 0x7e8, 0x7f0, 0x818, 0x820, 0x828, 0x830, 0x838, 0x840, 0x848, 0x850, 0x858, 0x860, 0x868, 0x870, 0x878, 0x880, 0x888, 0x890, 0x898, 0x8a0, 0x8a8, 0x8b0, 0x8b8, 0x8c0, 0x8c8, 0x8d0, 0x8d8, 0x8e0, 0x8e8, 0x8f0, 0x8f8, 0x900, 0x910, 0x918, 0x920, 0x928, 0x930, 0x938, 0x940, 0x948, 0x950, 0x958, 0x960, 0x968, 0x970, 0x978, 0x980, 0x988, 0x990, 0x998, 0x9a0, 0x9a8, 0x9b0, 0x9b8, 0x9c0, 0x9c8, 0x9d0, 0xa10, 0xa18, 0xa20, 0xa28, 0xa30, 0xa38, 0xa40, 0xa48, 0xa50, 0xa58, 0xa68, 0xa70, 0xa78, 0xaa0]),
    (T::AiType, &[0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x58, 0x158, 0x2e8, 0x2f0, 0x2f8, 0x300, 0x308, 0x310, 0x318, 0x320, 0x328, 0x330, 0x338, 0x340, 0x348, 0x350, 0x358, 0x368, 0x378, 0x380, 0x388, 0x390, 0x3c0, 0x3c8, 0x3d0, 0x3d8, 0x3e0, 0x3e8, 0x3f0, 0x3f8, 0x400, 0x408, 0x410, 0x418, 0x420, 0x428, 0x430, 0x438, 0x440, 0x448, 0x450, 0x458, 0x460, 0x468, 0x470, 0x478, 0x480, 0x488, 0x490, 0x498, 0x4a0, 0x4a8, 0x4b0, 0x500, 0x518, 0x578, 0x810, 0x828, 0x928]),
    (T::Character, &[0x8, 0x150, 0x160, 0x170, 0x180, 0x190, 0x1a0, 0x1b0, 0x1c0, 0x1d0, 0x1e0, 0x1f0, 0x200, 0x210, 0x220, 0x230, 0x240, 0x250, 0x258, 0x268]),
    (T::DestructibleDef, &[0x0, 0x8, 0x10, 0x18, 0x28, 0x38, 0x78, 0x80, 0x88, 0x90, 0xa0, 0xb0, 0xf0, 0xf8, 0x100, 0x108, 0x118, 0x128, 0x168, 0x170, 0x178, 0x180, 0x190, 0x1a0, 0x1e0, 0x1e8, 0x1f0, 0x1f8, 0x208, 0x218, 0x258, 0x260, 0x268, 0x270, 0x280, 0x290, 0x2d0, 0x2d8, 0x2e0, 0x2e8, 0x2f8, 0x308]),
];

pub fn spec(ty: AssetType) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.ty == ty)
}

/// Walks the asset at `h` by its spec: the name, then the fields.
pub fn flat(s: &mut Stream, h: usize, spec: &Spec) -> Result<Option<String>> {
    let mut known: Vec<usize> = spec.fields.iter().map(|f| f.0).collect();
    known.push(0);
    if let Some((_, words)) = DATA_WORDS.iter().find(|d| d.0 == spec.ty) {
        known.extend(words.iter());
    }
    s.no_loads(h, spec.size, &known)?;
    let name = s.string(h)?;
    for &(off, f) in spec.fields {
        match f {
            Str => {
                s.string(h + off)?;
            }
            A(ty) => {
                s.asset(ty, h + off)?;
            }
            Records(count, size) => {
                let n = s.u32_at(h + count)? as usize;
                if let Some(a) = s.records(h + off, n, size, 7)? {
                    for i in 0..n {
                        s.no_loads(a + size * i, size, &[])?;
                    }
                }
            }
            Runtime => {
                // `-2` there takes no VIRTUAL slot and loads nothing (read
                // on mp_common's light descriptions); other values are data.
            }
        }
    }
    Ok(name)
}

macro_rules! flat_walkers {
    ($($f:ident => $t:ident;)*) => {
        $(
            pub fn $f(s: &mut Stream, h: usize) -> Result<Option<String>> {
                flat(s, h, spec(AssetType::$t).expect("spec"))
            }
        )*
    };
}

flat_walkers! {
    surface_sound_def => SurfaceSoundDef;
    footstep_table => FootstepTable;
    surface_fx_table => SurfaceFxTable;
    impact_fx => ImpactFx;
    impact_sound => ImpactSound;
    entity_fx_impacts => EntityFxImpacts;
    entity_sound_impacts => EntitySoundImpacts;
    laser => Laser;
    beam => Beam;
    tracer => Tracer;
    phys_preset => PhysPreset;
    shellshock => Shellshock;
    shared_weapon_sounds => SharedWeaponSounds;
    vehicle_fx_def => VehicleFxDef;
    vehicle_sound_def => VehicleSoundDef;
    xmodel_alias => XModelAlias;
    character => Character;
    ai_type => AiType;
    vehicle => VehicleDef;
    light_description => LightDescription;
    objective => Objective;
    medal => Medal;
    cg_media => CgMedia;
    loc_dmg_table => LocDmgTable;
    bullet_penetration => BulletPenetration;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_sizes_match_the_type_table() {
        for s in SPECS {
            assert_eq!(s.ty.header_size(), s.size, "{}", s.ty.name());
            assert!(s.fields.iter().all(|f| f.0 + 8 <= s.size), "{}", s.ty.name());
        }
    }
}

const PIECE: usize = 0x3c8;

/// A `DestructibleDef` piece's pointer fields (offsets in the 0x3c8 piece,
/// HydraX's piece table: the base, then six damage stages of effects,
/// sounds, notifies, spawn models and physics presets), and 0x378 (read
/// on mp_common, not in HydraX's table).
const PIECE_FIELDS: &[(usize, F)] = &[(0x20, A(T::Fx)), (0x30, A(T::Fx)), (0x40, Str), (0x48, Str), (0x50, Str), (0x58, A(T::XModel)), (0x60, A(T::XModel)), (0x70, A(T::PhysPreset)), (0xa8, A(T::Fx)), (0xb8, Str), (0xc0, Str), (0xc8, Str), (0xd0, A(T::XModel)), (0xd8, A(T::XModel)), (0xe8, A(T::PhysPreset)), (0x120, A(T::Fx)), (0x130, Str), (0x138, Str), (0x140, Str), (0x148, A(T::XModel)), (0x150, A(T::XModel)), (0x160, A(T::PhysPreset)), (0x198, A(T::Fx)), (0x1a8, Str), (0x1b0, Str), (0x1b8, Str), (0x1c0, A(T::XModel)), (0x1c8, A(T::XModel)), (0x1d8, A(T::PhysPreset)), (0x210, A(T::Fx)), (0x220, Str), (0x228, Str), (0x230, Str), (0x238, A(T::XModel)), (0x240, A(T::XModel)), (0x250, A(T::PhysPreset)), (0x288, A(T::Fx)), (0x298, Str), (0x2a0, Str), (0x2a8, Str), (0x2b0, A(T::XModel)), (0x2b8, A(T::XModel)), (0x2c8, A(T::PhysPreset)), (0x300, A(T::Fx)), (0x310, Str), (0x318, Str), (0x320, Str), (0x328, A(T::XModel)), (0x330, A(T::XModel)), (0x340, A(T::PhysPreset)), (0x378, Str)];

/// `DestructibleDef` (0x30, read on mp_common: `{ char* name; XModel*
/// model; ?; u32 pieceCount; Piece* pieces; ... }`): name, the model, the
/// pieces, then each piece's fields in offset order.
pub fn destructible_def(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::DestructibleDef.header_size(), &[0, 8, 0x20])?;
    let name = s.string(h)?;
    s.asset(AssetType::XModel, h + 8)?;
    let n = s.u32_at(h + 0x18)? as usize;
    if let Some(t) = s.records(h + 0x20, n, PIECE, 7)? {
        let mut known: Vec<usize> = PIECE_FIELDS.iter().map(|f| f.0).collect();
        if let Some((_, words)) = DATA_WORDS.iter().find(|d| d.0 == AssetType::DestructibleDef) {
            known.extend(words.iter());
        }
        for i in 0..n {
            let piece = t + PIECE * i;
            s.no_loads(piece, PIECE, &known)?;
            for &(off, f) in PIECE_FIELDS {
                match f {
                    A(ty) => {
                        s.asset(ty, piece + off)?;
                    }
                    _ => {
                        s.string(piece + off)?;
                    }
                }
            }
        }
    }
    Ok(name)
}
