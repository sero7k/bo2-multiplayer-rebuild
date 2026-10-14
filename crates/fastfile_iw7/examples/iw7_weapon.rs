//! What an Infinite Warfare weapon points at:
//! `iw7_weapon <zone.ff> <weapon name>`: its models, materials, effects,
//! animation package (each animation's name), sound aliases and strings.
use fastfile_iw7::asset_type::AssetType;
use fastfile_iw7::walk::{WeaponField, walk_fitted};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&args[1])).expect("zone");
    let (w, stop) = walk_fitted(&zone);
    if let Some(stop) = stop {
        eprintln!("walk stopped: {stop:?}");
    }
    let cap = &w.cap;
    for weapon in cap.weapons.iter().filter(|r| r.name == args[2]) {
        println!("{} \"{}\"", weapon.name, weapon.display_name);
        if let Some(def) = weapon.def {
            let q = |o: usize| u64::from_le_bytes(zone.image[def.pos + o..def.pos + o + 8].try_into().unwrap());
            println!("  anim packages {:#x} {:#x} {:#x}", q(448), q(456), q(464));
        }
        for (field, id) in &weapon.assets {
            let at = match field {
                WeaponField::Def(o) => format!("def+{o}"),
                WeaponField::Complete(o) => format!("complete+{o}"),
            };
            println!("  {at:<14} {:<14} {}", id.ty.name(), cap.name_of(*id));
            if id.ty == AssetType::AnimPackage
                && let Some(p) = cap.anim_packages.get(id.index as usize)
            {
                for (i, a) in p.anims.iter().enumerate() {
                    if let Some(a) = a {
                        println!("      anim {i:3} {}", cap.name_of(*a));
                    }
                }
            }
            if id.ty == AssetType::SfxPackage
                && let Some(p) = cap.sfx_packages.get(id.index as usize)
            {
                for (o, s) in &p.sounds {
                    println!("      sound +{o:3} {s}");
                }
            }
        }
        for (field, s) in &weapon.strings {
            println!("  {field:?} \"{s}\"");
        }
        for (k, list) in weapon.attachments.iter().enumerate() {
            let names: Vec<&str> = list.iter().flatten().map(|id| cap.name_of(*id)).collect();
            if !names.is_empty() {
                println!("  attachments{} {}", k + 1, names.join(" "));
            }
        }
    }
}
