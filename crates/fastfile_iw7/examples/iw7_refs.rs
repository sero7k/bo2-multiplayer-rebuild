//! Why a map's placed models' references miss: `iw7_refs <map.ff>` prints
//! the fitted deltas and, for each placed model whose model reference
//! named no slot, the reference, its local offset and the nearest XModel
//! slot (with the difference), as a histogram of differences.
use fastfile_iw7::AssetType;
use fastfile_iw7::layout as L;
use fastfile_iw7::ptr::Ptr;
use fastfile_iw7::walk::walk_fitted;
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let (w, stop) = walk_fitted(&zone);
    println!("stop {:?}; hint {:#x}; {} shared ranges; {} deltas", stop.map(|s| s.error), zone.header.shared_virtual, zone.header.shared_files.len(), w.deltas.len());
    for d in w.deltas.iter().take(6).chain(w.deltas.iter().rev().take(6)) {
        println!("  delta from {:#x}: {:#x}", d.0, d.1);
    }
    let slots: Vec<(u64, fastfile_iw7::walk::AssetId)> = {
        let mut s = w.slot_list();
        s.sort_by_key(|x| x.0);
        s
    };
    let xm: Vec<u64> = slots.iter().filter(|(_, id)| id.ty == AssetType::XModel).map(|(k, _)| *k).collect();
    println!("{} slots, {} XModel slots (first {:x?} last {:x?}); spans {}", slots.len(), xm.len(), xm.first(), xm.last(), w.spans.len());
    let g = &w.cap.gfx_worlds[0];
    let Some(at) = g.smodels else { return };
    let im = &zone.image;
    let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut shown = 0;
    let mut distinct_refs: BTreeMap<u32, usize> = BTreeMap::new();
    for i in 0..g.smodel_count as usize {
        let d = at + L::GfxStaticModelDrawInst::SIZE * i;
        let raw = u64::from_le_bytes(im[d + 56..d + 64].try_into().unwrap());
        let p = Ptr::decode(raw);
        let resolved = g.smodel_models[i].is_some();
        *kinds.entry(format!("{} {}", if resolved { "ok" } else { "miss" }, match p { Ptr::Ref { block, .. } => format!("ref{block}"), o => format!("{o:?}") })).or_default() += 1;
        if resolved {
            continue;
        }
        if let Ptr::Ref { block: 8, offset } = p {
            *distinct_refs.entry(offset).or_default() += 1;
            let local = w.local(offset as u64);
            let near = xm.iter().min_by_key(|k| k.abs_diff(local)).copied().unwrap_or(0);
            let diff = near as i64 - local as i64;
            *hist.entry(diff / 16 * 16).or_default() += 1;
            if shown < 12 {
                shown += 1;
                println!("  miss #{i}: ref {offset:#x} local {local:#x} nearest XModel slot {near:#x} ({diff:+#x})");
            }
        }
    }
    println!("{kinds:?}");
    println!("{} distinct missing refs", distinct_refs.len());
    // For each distinct missing ref: the deltas (ref - slot) to XModel
    // slots within 0x8000 of its fitted local; a common delta peaks.
    let mut dh: BTreeMap<u64, usize> = BTreeMap::new();
    for &r in distinct_refs.keys() {
        let local = w.local(r as u64);
        for &k in xm.iter().filter(|k| k.abs_diff(local) <= 0x8000) {
            *dh.entry((r as u64).wrapping_sub(k)).or_default() += 1;
        }
    }
    let mut dv: Vec<_> = dh.into_iter().collect();
    dv.sort_by_key(|x| std::cmp::Reverse(x.1));
    println!("candidate deltas (ref - XModel slot), top: {:x?}", &dv[..dv.len().min(12)]);
    // Per ref, sorted by ref: the fitted delta and the ref's region.
    for (&r, n) in distinct_refs.iter().step_by((distinct_refs.len() / 12).max(1)) {
        println!("  ref {r:#x} x{n}: fitted local {:#x}", w.local(r as u64));
    }
    let mut h: Vec<_> = hist.into_iter().collect();
    h.sort_by_key(|x| std::cmp::Reverse(x.1));
    println!("diff histogram (top): {:?}", &h[..h.len().min(20)]);
    // Ok refs: their raw and local, to see the deltas that worked.
    let mut ok = 0;
    for i in 0..g.smodel_count as usize {
        if g.smodel_models[i].is_none() || ok >= 6 {
            continue;
        }
        ok += 1;
        let d = at + L::GfxStaticModelDrawInst::SIZE * i;
        let raw = u64::from_le_bytes(im[d + 56..d + 64].try_into().unwrap());
        if let Ptr::Ref { offset, .. } = Ptr::decode(raw) {
            println!("  ok #{i}: ref {offset:#x} local {:#x} -> {}", w.local(offset as u64), w.cap.name_of(g.smodel_models[i].unwrap()));
        }
    }
}
