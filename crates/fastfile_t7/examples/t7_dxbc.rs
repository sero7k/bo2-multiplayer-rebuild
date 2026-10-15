//! Dumps a Black Ops III zone's shader programs: `t7_dxbc <zone.ff> <out dir>
//! [name substring]` writes every `DXBC` container in the inflated zone
//! whose nearest preceding name (the shader's, which the walk reads just
//! before its program) holds the substring, as `<name>_<offset>.dxbc`, for
//! `fxc /dumpbin` to disassemble. Lists them all without a substring.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_t7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let out = std::path::Path::new(&a[2]);
    std::fs::create_dir_all(out).expect("out dir");
    let want = a.get(3).cloned().unwrap_or_default();
    let img = &zone.image;
    let mut i = 0;
    let mut n = 0;
    while i + 32 <= img.len() {
        if &img[i..i + 4] != b"DXBC" {
            i += 1;
            continue;
        }
        let size = u32::from_le_bytes(img[i + 24..i + 28].try_into().unwrap()) as usize;
        if size < 32 || i + size > img.len() {
            i += 1;
            continue;
        }
        // The nearest printable name before the program (within 512 bytes).
        let back = i.saturating_sub(512);
        let mut name = String::new();
        let mut j = i;
        while j > back {
            j -= 1;
            if img[j] == 0 {
                continue;
            }
            let mut k = j;
            while k > back && img[k - 1] != 0 && img[k - 1].is_ascii_graphic() {
                k -= 1;
            }
            let s = &img[k..=j];
            if s.len() >= 6 && s.iter().all(|c| c.is_ascii_graphic()) {
                name = String::from_utf8_lossy(s).into_owned();
                break;
            }
            j = k;
        }
        if want.is_empty() {
            println!("{i:#x} {size} {name}");
        } else if name.contains(&want) {
            let safe: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
            let path = out.join(format!("{safe}_{i:x}.dxbc"));
            std::fs::write(&path, &img[i..i + size]).expect("write");
            println!("{} ({size} bytes)", path.display());
            n += 1;
        }
        i += size;
    }
    if !want.is_empty() {
        println!("{n} written");
    }
}
