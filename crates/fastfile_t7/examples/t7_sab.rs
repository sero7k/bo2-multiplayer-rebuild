//! bo2mp: a Black Ops III sound bank's raw entry table: `t7_sab <bank>
//! [hex id ...]`: the first rows, and where each id sits.
use std::io::{Read, Seek, SeekFrom};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut f = std::fs::File::open(&args[1]).expect("bank");
    let mut head = vec![0u8; 2048];
    f.read_exact(&mut head).unwrap();
    let u32_at = |b: &[u8], a: usize| u32::from_le_bytes([b[a], b[a + 1], b[a + 2], b[a + 3]]);
    let u64_at = |b: &[u8], a: usize| u64::from(u32_at(b, a)) | (u64::from(u32_at(b, a + 4)) << 32);
    println!("header {:02x?}", &head[..0x48]);
    let (count, table, second) = (u32_at(&head, 0x14) as usize, u64_at(&head, 0x28), u64_at(&head, 0x30));
    let len = (second.saturating_sub(table)) as usize;
    println!("count {count} table {table:#x} second {second:#x} table bytes {len} = {:.3} a row", len as f64 / count as f64);
    f.seek(SeekFrom::Start(table)).unwrap();
    let mut b = vec![0u8; len.max(count * 36)];
    f.read_exact(&mut b).unwrap();
    for r in 0..6 {
        println!("row {r}: {}", b[r * 36..r * 36 + 36].iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "));
    }
    for id in &args[2..] {
        let id = u32::from_str_radix(id.trim_start_matches("0x"), 16).unwrap().to_le_bytes();
        for i in 0..b.len() - 4 {
            if b[i..i + 4] == id {
                let s = i.saturating_sub(8);
                println!("{:02x?} at +{i} (mod 36: {}): {}", id, i % 36, b[s..(i + 40).min(b.len())].iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "));
            }
        }
    }
    let mut c = vec![0u8; 96];
    f.seek(SeekFrom::Start(second)).unwrap();
    f.read_exact(&mut c).unwrap();
    println!("second {:02x?}", c);
}
