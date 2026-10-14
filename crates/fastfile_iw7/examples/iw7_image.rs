//! Exports images as `.dds` (and `.png` of the top level):
//! `iw7_image <zone.ff>... -- <image name>... ` with
//! `IW7_PAKS=<dir>` (default: the first zone's directory) and
//! `IW7_OUT=<dir>` (default: `.`).
//!
//! A streamed image's best level is read from `<paks>/imagefile<N>.pak`
//! when that exists, else from `<paks>/iw7_imagefile<N>_<offset hex>.bin`
//! (just that range, copied from the `.pak`). `IW7_MIP=<n>`: the `.png` is
//! level n. Missing ranges are listed
//! with a PowerShell snippet that copies them.
use fastfile_iw7::texture::{self, ImageLevels};
use fastfile_iw7::walk::walk_zone;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let split = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let (zones, names) = (&args[..split], args.get(split + 1..).unwrap_or(&[]));
    let paks = std::env::var("IW7_PAKS").map(PathBuf::from).unwrap_or_else(|_| Path::new(&zones[0]).parent().unwrap().to_owned());
    let out = PathBuf::from(std::env::var("IW7_OUT").unwrap_or_else(|_| ".".into()));
    std::fs::create_dir_all(&out).expect("out dir");
    let mut missing = Vec::new();
    let mut done = std::collections::HashSet::new();
    for path in zones {
        let zone = fastfile_iw7::Zone::open(Path::new(path)).expect("zone");
        let (cap, _) = walk_zone(&zone);
        for im in cap.images.iter().filter(|im| !im.name.starts_with(',') && names.contains(&im.name)) {
            if !done.insert(im.name.clone()) {
                continue;
            }
            let levels: Result<ImageLevels, String> = if im.pixels.is_some() {
                texture::inline_levels(im, &zone.image).map_err(|e| e.to_string())
            } else if let Some((k, _)) = texture::best_stream(im) {
                // Every stream from k down (the full chain); the top alone if
                // lower ones are missing.
                let mut parts = vec![Vec::new(); k + 1];
                let mut err = None;
                for (i, part) in parts.iter_mut().enumerate() {
                    let Some(f) = im.stream_files[i].filter(|f| f.offset_end > f.offset) else { continue };
                    let full = paks.join(format!("imagefile{}.pak", f.file_index));
                    let one = paks.join(format!("iw7_imagefile{}_{:x}.bin", f.file_index, f.offset));
                    let raw = if full.exists() { fastfile_iw7::pak::read_range(&full, &f).ok() } else { std::fs::read(&one).ok() };
                    match raw {
                        Some(raw) => match texture::stream_part(im, i, &raw) {
                            Ok(p) => *part = p,
                            Err(e) => err = Some(format!("stream {i}: {e}")),
                        },
                        None => missing.push(f),
                    }
                }
                match err {
                    Some(e) => Err(e),
                    None if parts[k].is_empty() => Err("top stream range not available (see the snippet below)".into()),
                    None => Ok(texture::stream_levels(im, k, &parts)),
                }
            } else {
                Err("no pixels and no stream range".into())
            };
            match levels {
                Ok(l) => {
                    let cube = im.map_type == 5;
                    let dds = texture::to_dds(&l, im.num_elements.max(1) as u32, cube);
                    std::fs::write(out.join(format!("{}.dds", im.name)), &dds).expect("write dds");
                    let mip: u32 = std::env::var("IW7_MIP").ok().and_then(|m| m.parse().ok()).unwrap_or(0);
                    let png = texture::level_rgba(&l, mip).map(|(w, h, rgba)| {
                        let f = std::fs::File::create(out.join(format!("{}.png", im.name))).expect("png");
                        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
                        e.set_color(png::ColorType::Rgba);
                        e.write_header().and_then(|mut w| w.write_image_data(&rgba)).expect("png");
                    });
                    println!(
                        "{} {} {}x{} {} levels ({}), {} bytes -> {}.dds{}",
                        im.name,
                        texture::dxgi_name(l.format),
                        l.width,
                        l.height,
                        l.levels,
                        l.stream.map_or("inline".to_owned(), |k| format!("stream {k}")),
                        l.data.len(),
                        im.name,
                        if png.is_some() { " + .png" } else { "" }
                    );
                }
                Err(e) => println!("{}: {e}", im.name),
            }
        }
    }
    for n in names.iter().filter(|n| !done.contains(*n)) {
        println!("{n}: not defined in the given zones");
    }
    if !missing.is_empty() {
        let list: Vec<String> = missing.iter().map(|f| format!("@({},{},{})", f.file_index, f.offset, f.offset_end)).collect();
        println!("\n# PowerShell: copy the missing ranges into C:\\RE-Tools\\samples");
        println!(
            "foreach($x in @({})){{ $f=[IO.File]::OpenRead(\"E:\\iw_game_files\\imagefile$($x[0]).pak\"); [void]$f.Seek($x[1],'Begin'); $n=$x[2]-$x[1]; $b=New-Object byte[] $n; $g=0; while($g -lt $n){{$k=$f.Read($b,$g,$n-$g); if($k -le 0){{break}}; $g+=$k}}; $f.Close(); [IO.File]::WriteAllBytes(('C:\\RE-Tools\\samples\\iw7_imagefile{{0}}_{{1:x}}.bin' -f $x[0],$x[1]),$b) }}",
            list.join(",")
        );
    }
}
