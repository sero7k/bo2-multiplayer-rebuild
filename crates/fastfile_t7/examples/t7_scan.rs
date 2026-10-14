//! Looks for pointer runs in a zone's inflated image, to find where a list
//! the walker misreads really starts: `t7_scan <zone.ff> <hex from> <len>
//! <need>` prints each offset (8-aligned to `from`'s alignment) where
//! `need` 8-byte words in a row are all pointer fills (`-1`) or
//! references (block 5 offsets: high byte 0x5n), and where a name follows
//! a pointer fill.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let z = fastfile_t7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let from = usize::from_str_radix(a[2].trim_start_matches("0x"), 16).expect("from");
    let len: usize = a[3].parse().expect("len");
    let need: usize = a[4].parse().expect("need");
    let img = &z.image;
    let end = (from + len).min(img.len());
    let word = |o: usize| u64::from_le_bytes(img[o..o + 8].try_into().unwrap());
    let is_ptr = |w: u64| w == u64::MAX || (w >> 60 == 5 && (w >> 32) & 0x0fff_ffff == 0);
    // `T7_SCAN_ALIGN=<n>`: only offsets with this remainder mod 8 of `from`.
    let align: Option<usize> = std::env::var("T7_SCAN_ALIGN").ok().and_then(|v| v.parse().ok());
    let mut shown = 0;
    let mut o = from;
    while o + 8 * need <= end && shown < 12 {
        if align.is_none_or(|r| (o - from) % 8 == r) && (0..need).all(|k| is_ptr(word(o + 8 * k))) {
            println!("{need} pointers at {o:#x} (+{:#x})", o - from);
            shown += 1;
            o += 8 * need;
            continue;
        }
        o += 1;
    }
}
