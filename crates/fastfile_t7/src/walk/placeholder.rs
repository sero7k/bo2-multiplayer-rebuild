//! Asset types read only as "," placeholders so far: the header (its size
//! measured on retail zones, name first) and the name, every other 8-byte
//! field required not to load data. An inline asset of one of these types
//! that loads more fails the walk at that field, naming it, until the type
//! gets a real walker.
//!
//! ```text
//! lightdescription 0x240   (mp_common: ",gfx_dlight_gen_omni_r200599854254")
//! aimtable         0x330   (mp_common: ",soldier_aim")
//! customizationtablecolor 0x30 (mp_common: ",cc_c_t7_mp_warrior_body1_skin1")
//! customizationtable 0x28 (mp_common: ",mp_character_customization")
//! maptable         0x28    (mp_common: ",maptable_mp_base")
//! xcam             0xa8    (mp_common: ",ui_cam_cac_lmg_light")
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
    aim_table => AimTable;
    xcam => XCam;
    customization_table => CustomizationTable;
    map_table => MapTable;
    customization_table_color => CustomizationTableColor;
}
