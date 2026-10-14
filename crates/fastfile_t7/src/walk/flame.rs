//! Flamethrower tables (`FlameTable`, 0x218; read on retail `core_common`,
//! `default_flame_firstperson` / `default_flame_thirdperson`).
//!
//! ```text
//! 0x000..0x1b0 numbers (flame stream, cone and damage tuning)
//! 0x1b0 char* name; 0x1b8, 0x1c0, 0x1d0, 0x1d8, 0x1e0 references to assets
//!       loaded earlier (effects); 0x1f8, 0x200, 0x208, 0x210 char* sound
//!       aliases (off, start, fire loop, cooldown)
//! ```
//!
//! Stream order: the name, then the four sound aliases.

use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::Stream;

const NAME: usize = 0x1b0;
const SOUNDS: [usize; 4] = [0x1f8, 0x200, 0x208, 0x210];

pub fn flame_table(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let mut known = vec![NAME];
    known.extend(SOUNDS);
    s.no_loads(h, AssetType::FlameTable.header_size(), &known)?;
    let name = s.string(h + NAME)?;
    for f in SOUNDS {
        s.string(h + f)?;
    }
    Ok(name)
}
