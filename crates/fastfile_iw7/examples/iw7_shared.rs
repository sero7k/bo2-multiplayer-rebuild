//! A zone's shared-asset streams (`assetfile<N>.pak` ranges):
//! `iw7_shared <zone.ff> [<ranges.bin>]` lists the ranges in table order
//! (with a PowerShell snippet that copies them, end to end, into one file);
//! given that file, inflates and walks them and lists the assets they hold.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = a.get(1).expect("usage: iw7_shared <zone.ff> [<ranges.bin>]");
    let file = std::fs::read(path).expect("read zone");
    let h = fastfile_iw7::parse_file_header(&file).expect("zone header");
    println!("shared_virtual {:#x}, {} shared ranges, {} image ranges", h.shared_virtual, h.shared_files.len(), h.stream_files.len());
    let Some(ranges) = a.get(2) else {
        let mut total = 0u64;
        for (i, f) in h.shared_files.iter().enumerate() {
            let n = f.offset_end - f.offset;
            println!("{i:4} assetfile{}.pak {:#x}..{:#x} ({n:#x}){}", f.file_index, f.offset, f.offset_end, if f.localized { " localized" } else { "" });
            total += n;
        }
        println!("total {total:#x}");
        let stem = std::path::Path::new(path).file_stem().unwrap().to_string_lossy().trim_start_matches("iw7_").to_owned();
        let list: Vec<String> = h.shared_files.iter().map(|f| format!("@({},{},{})", f.file_index, f.offset, f.offset_end)).collect();
        println!("\n# PowerShell: copy the ranges, end to end, into C:\\RE-Tools\\samples\\iw7_shared_{stem}.bin");
        println!(
            "$out=[IO.File]::Create('C:\\RE-Tools\\samples\\iw7_shared_{stem}.bin'); foreach($x in @({})){{ $f=[IO.File]::OpenRead(\"E:\\iw_game_files\\assetfile$($x[0]).pak\"); [void]$f.Seek($x[1],'Begin'); $n=$x[2]-$x[1]; $b=New-Object byte[] $n; $g=0; while($g -lt $n){{$k=$f.Read($b,$g,$n-$g); if($k -le 0){{break}}; $g+=$k}}; $f.Close(); $out.Write($b,0,$n) }}; $out.Close()",
            list.join(",")
        );
        return;
    };
    let raw = std::fs::read(ranges).expect("read ranges");
    let sv = fastfile_iw7::pak::SharedVirtual::from_concatenated(&raw, &h.shared_files).expect("inflate shared ranges");
    println!("inflated {:#x} bytes (header: {:#x})", sv.data.len(), h.shared_virtual);
    let (cap, stops) = fastfile_iw7::walk::walk_shared(&sv);
    println!("walked {} assets; {} ranges did not end cleanly", cap.walked, stops.len());
    for s in stops.iter().take(10) {
        println!("  range {}: {:?} at {:#x}: {}", s.asset_index, s.asset_type, s.file_pos, s.error);
    }
    let verts: usize = cap.model_surfs.iter().flat_map(|m| &m.surfs).map(|s| s.vert_count as usize).sum();
    println!("xmodel_surfs {} ({} surfaces, {verts} verts), other assets {}", cap.model_surfs.len(), cap.model_surfs.iter().map(|m| m.surfs.len()).sum::<usize>(), cap.others.len());
    if std::env::var_os("IW7_LIST").is_some() {
        for m in &cap.model_surfs {
            let v: usize = m.surfs.iter().map(|s| s.vert_count as usize).sum();
            let inline = m.surfs.iter().filter(|s| s.verts.is_some()).count();
            println!("  {} surfs {} verts {v} (vertex data inline {inline})", m.name, m.surfs.len());
        }
    }
}
