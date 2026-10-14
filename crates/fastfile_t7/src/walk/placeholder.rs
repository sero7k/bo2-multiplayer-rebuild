//! Asset types read only as "," placeholders so far: the header (its size
//! measured on retail zones, name first) and the name, every other 8-byte
//! field required not to load data. An inline asset of one of these types
//! that loads more fails the walk at that field, naming it, until the type
//! gets a real walker.
//!
//! ```text
//! lightdescription 0x240   (mp_common: ",gfx_dlight_gen_omni_r200599854254")
//! fx               0x90    (mp_common: ",weapon/fx_betty_light")
//! tagfx            0x18    (mp_common: ",equip_trophy_laser")
//! vehicledef       0xaa8   (mp_common: ",defaultvehicle_mp")
//! ```

use super::simple::placeholder;
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::Stream;

macro_rules! placeholders {
    ($($f:ident => $t:ident;)*) => {
        $(
            pub fn $f(s: &mut Stream, h: usize) -> Result<Option<String>> {
                placeholder(s, AssetType::$t, h, 0)
            }
        )*
    };
}

placeholders! {
    vehicle => VehicleDef;
    tag_fx => TagFx;
    light_description => LightDescription;
    fx => Fx;
}
