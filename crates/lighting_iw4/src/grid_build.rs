//! bo2mp: a light grid's rows made, for a map from a game whose own grid is
//! not read (Infinite Warfare's: `assets::lane::iw7` makes one from the
//! map's lightmaps).
//!
//! The simplest grid the sampler (`row_rle`) reads: every row (along the
//! row axis) covers the same columns and heights, each column one solid run
//! of every height from the bottom. Entries are then laid out row by row,
//! column by column, height fastest: entry `(row * cols + col) * heights +
//! height`.

use crate::row_rle::LIGHT_GRID_ROW_HEADER_SIZE;

/// The bytes `full_box_row` writes for a row of `cols` columns (4-byte
/// aligned, as rows are addressed in 4-byte units).
pub const fn full_box_row_len(cols: u16) -> usize {
    let runs = (cols as usize).div_ceil(0xff);
    (LIGHT_GRID_ROW_HEADER_SIZE + runs * 3).div_ceil(4) * 4
}

/// Row `row` of a full box grid into `out` (at least `full_box_row_len`
/// bytes): its header (`col_start`, `cols`, `height_start`, `heights`, the
/// row's first entry) and runs of at most 255 columns, each `heights`
/// entries a column from the first height (narrow rows: 3 bytes a run),
/// zero padded. The bytes written; 0 when the row does not fit the format
/// (more than 255 heights, entries past `u32`, `out` too short).
pub fn full_box_row(row: usize, col_start: u16, cols: u16, height_start: u16, heights: u16, out: &mut [u8]) -> usize {
    let len = full_box_row_len(cols);
    let Some(base) = row.checked_mul(cols as usize).and_then(|n| n.checked_mul(heights as usize)) else { return 0 };
    let Ok(entry_base) = u32::try_from(base) else { return 0 };
    if heights == 0 || heights > 0xff || cols == 0 || out.len() < len {
        return 0;
    }
    out[..len].fill(0);
    out[0..2].copy_from_slice(&col_start.to_le_bytes());
    out[2..4].copy_from_slice(&cols.to_le_bytes());
    out[4..6].copy_from_slice(&height_start.to_le_bytes());
    out[6..8].copy_from_slice(&heights.to_le_bytes());
    out[8..12].copy_from_slice(&entry_base.to_le_bytes());
    let mut at = LIGHT_GRID_ROW_HEADER_SIZE;
    let mut left = cols;
    while left > 0 {
        let run = if left > 0xff { 0xff } else { left };
        out[at..at + 3].copy_from_slice(&[run as u8, heights as u8, 0]);
        at += 3;
        left -= run;
    }
    len
}

#[cfg(test)]
mod tests {
    use crate::row_rle::*;

    /// Each cell of the box reads back the entry the layout gives it.
    #[test]
    fn every_cell_reads_its_entry() {
        let (cols, heights) = (300u16, 5u16);
        let len = super::full_box_row_len(cols);
        assert_eq!(len % 4, 0);
        for row in 0..3usize {
            let mut data = [0xaau8; 64];
            assert_eq!(super::full_box_row(row, 1000, cols, 2000, heights, &mut data), len);
            let header = LightGridRowHeader::from_bytes(&data).unwrap();
            assert_eq!((header.col_start, header.col_count, header.row_start, header.row_count), (1000, cols, 2000, heights));
            let rle = &data[LIGHT_GRID_ROW_HEADER_SIZE..];
            for col in [0u32, 1, 254, 255, 299] {
                for h in 0..u32::from(heights) {
                    assert!(light_grid_row_contains_cell(&header, col, h));
                    let c = light_grid_rle_advance_to_column(rle, header.row_count, header.entry_base, col).unwrap();
                    let z_base = light_grid_solid_run_z_base(&rle[c.run_offset..], false).unwrap();
                    let entry = light_grid_solid_run_entry_index(c.entry_index, c.col_in_run, h, c.run_increment, z_base);
                    assert!(light_grid_solid_run_has_entry(h - z_base, c.run_increment));
                    assert_eq!(entry as usize, (row * usize::from(cols) + col as usize) * usize::from(heights) + h as usize);
                }
            }
            assert!(!light_grid_row_contains_cell(&header, 300, 0) && !light_grid_row_contains_cell(&header, 0, 5));
        }
        let mut short = [0u8; 8];
        assert_eq!(super::full_box_row(0, 0, cols, 0, heights, &mut short), 0);
    }
}
