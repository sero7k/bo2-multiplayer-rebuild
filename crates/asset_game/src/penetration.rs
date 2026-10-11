use weapon_iw4::{PenetrationDepthTable, SURFACE_TYPE_NAMES, T6_SURFACE_TYPE_NAMES, split_pen_key_in};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PenTableError {
    MissingMagic,
    OddPair,
    BadNumber,
    UnknownKey,
    NotUtf8,
}

pub fn parse_pen_table_info_string(text: &str) -> Result<PenetrationDepthTable, PenTableError> {
    parse_pen_table_with(text, &SURFACE_TYPE_NAMES)
}

/// Black Ops II's `info/bullet_penetration_mp`: the same `BULLET_PEN_TABLE`
/// info string, keyed by its own 32 surface names.
pub fn parse_pen_table_t6(data: &[u8]) -> Result<PenetrationDepthTable, PenTableError> {
    let mut data = data;
    while let [rest @ .., 0] = data {
        data = rest;
    }
    let text = core::str::from_utf8(data).map_err(|_| PenTableError::NotUtf8)?;
    parse_pen_table_with(text, &T6_SURFACE_TYPE_NAMES)
}

fn parse_pen_table_with(
    text: &str,
    surface_names: &[&str],
) -> Result<PenetrationDepthTable, PenTableError> {
    let text = text.trim();
    let rest = text
        .strip_prefix("BULLET_PEN_TABLE")
        .ok_or(PenTableError::MissingMagic)?;
    let mut table = PenetrationDepthTable::empty();
    let mut parts = rest.split('\\').filter(|p| !p.is_empty());
    loop {
        let Some(key) = parts.next() else {
            break;
        };
        let Some(value) = parts.next() else {
            return Err(PenTableError::OddPair);
        };
        let parsed: f32 = value.parse().map_err(|_| PenTableError::BadNumber)?;
        let (ptype, surf) =
            split_pen_key_in(key, surface_names).ok_or(PenTableError::UnknownKey)?;
        table.set_depth(ptype, surf, parsed);
    }
    Ok(table)
}

pub fn parse_pen_table_rawfile(data: &[u8]) -> Result<PenetrationDepthTable, PenTableError> {
    let text = core::str::from_utf8(data).map_err(|_| PenTableError::NotUtf8)?;
    parse_pen_table_info_string(text)
}

pub fn capture_pen_table(
    name: &str,
    data: &[u8],
    zlib_compressed: bool,
) -> Option<PenetrationDepthTable> {
    if name != "info/bullet_penetration_mp" {
        return None;
    }
    let mut bytes = if zlib_compressed {
        asset_transport::inflate_zlib(data).ok()?
    } else {
        data.to_vec()
    };
    if bytes.last() == Some(&0) {
        bytes.pop();
    }
    parse_pen_table_rawfile(&bytes).ok()
}
