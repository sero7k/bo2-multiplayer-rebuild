//! Which placed models of a map are not drawn and why: `iw7_props <map.ff>`
//! prints each model name with no mesh, how many placements name it, the
//! asset type its reference resolved to, and whether an XModel of that
//! name (or an XModelSurfs) is in the zone.
use fastfile_iw7::AssetType;
use fastfile_iw7::walk::walk_fitted;
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let (w, stop) = walk_fitted(&zone);
    println!("stop {:?}", stop.map(|s| s.error));
    let cap = &w.cap;
    let g = &cap.gfx_worlds[0];
    let mut by: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut none = 0usize;
    for id in g.smodel_models.iter() {
        let Some(id) = id else {
            none += 1;
            continue;
        };
        let name = cap.name_of(*id).trim_start_matches(',').to_owned();
        let e = by.entry(name).or_insert((0, format!("{:?}#{}", id.ty, id.index)));
        e.0 += 1;
    }
    println!("{} placements without a model reference", none);
    for (name, (n, ty)) in &by {
        let x = cap.xmodels.iter().find(|x| &x.name == name);
        let state = match x {
            None => {
                let surfs = cap.model_surfs.iter().any(|m| m.name.trim_start_matches(',') == name);
                format!("NO XMODEL (surfs asset of that name: {surfs})")
            }
            Some(x) => {
                if x.lods.iter().any(|l| l.numsurfs > 0 && l.surfs.is_some()) {
                    continue;
                }
                format!("lods {:?}", x.lods.iter().map(|l| (l.numsurfs, l.surfs.map(|s| cap.name_of(s).to_owned()))).collect::<Vec<_>>())
            }
        };
        println!("{n:5} {name} [{ty}] {state}");
    }
    let _ = AssetType::XModel;
}
