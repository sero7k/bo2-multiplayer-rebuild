//! A Black Ops II mod zone's inventory (a Plutonium-style `mod.ff`): its
//! weapons, attachments, camos, models, materials, pictures, animations,
//! sound banks, effects, string tables, text and raw files, and what
//! `asset_t6::t6_mod::sanitize_capture` leaves out of a match.
//! `t6_mod_inventory <mod.ff> [mod.iwd listing.txt]` (the listing: one
//! archive path a line, as `images/x.iwi` and `xanim/y`).
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("usage: t6_mod_inventory <mod.ff> [iwd listing]");
    let listing: Vec<String> = a
        .next()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| t.lines().map(|l| l.trim().to_ascii_lowercase()).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default();
    let mut c = asset_t6::capture_zone(std::path::Path::new(&path)).expect("capture");
    let real = |n: &str| !n.is_empty() && !n.starts_with(',');
    println!("zone {}", c.zone);
    println!("weapons {}:", c.weapons.len());
    for w in &c.weapons {
        println!(
            "  {} display={} camo={} view={:?} world={:?} anims={}",
            w.name,
            w.display_name,
            w.camo,
            w.gun_models.iter().find(|s| !s.is_empty()),
            w.world_models.iter().find(|s| !s.is_empty()),
            w.xanims.iter().filter(|s| !s.is_empty()).count()
        );
    }
    println!("attachments {}: {}", c.attachments.len(), c.attachments.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(" "));
    println!("attachment uniques {}", c.attachment_uniques.len());
    println!("weapon camos {}: {}", c.weapon_camos.len(), c.weapon_camos.iter().map(|k| k.name.as_str()).collect::<Vec<_>>().join(" "));
    println!("xmodels {} ({} own)", c.xmodels.len(), c.xmodels.iter().filter(|m| real(&m.name)).count());
    println!("materials {} ({} own)", c.materials.len(), c.materials.iter().filter(|m| real(&m.name)).count());
    let streamed = c.images.iter().filter(|i| i.streamed_parts > 0).count();
    let embedded = c.images.iter().filter(|i| i.embedded.is_some()).count();
    let in_iwd = c
        .images
        .iter()
        .filter(|i| listing.contains(&format!("images/{}.iwi", i.name.trim_start_matches(',').to_ascii_lowercase())))
        .count();
    println!("images {} ({streamed} streamed, {embedded} in the zone, {in_iwd} in the mod's archive)", c.images.len());
    let inline: std::collections::BTreeSet<String> = c.xanims.iter().map(|x| x.name.to_ascii_lowercase()).collect();
    let mut wanted: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for w in &c.weapons {
        wanted.extend(w.xanims.iter().filter(|n| real(n)).map(|n| n.to_ascii_lowercase()));
    }
    for u in &c.attachment_uniques {
        wanted.extend(u.xanims.iter().filter(|n| real(n)).map(|n| n.to_ascii_lowercase()));
    }
    let from_iwd = wanted.iter().filter(|n| !inline.contains(*n) && listing.contains(&format!("xanim/{n}"))).count();
    let elsewhere = wanted.iter().filter(|n| !inline.contains(*n) && !listing.contains(&format!("xanim/{n}"))).count();
    println!(
        "xanims: {} in the zone; the weapons name {}: {} in the zone, {from_iwd} in the mod's archive, {elsewhere} from Black Ops II's zones",
        c.xanims.len(),
        wanted.len(),
        wanted.iter().filter(|n| inline.contains(*n)).count()
    );
    println!("sound banks {}:", c.sound_banks.len());
    for b in &c.sound_banks {
        println!("  {} ({} aliases)", b.name, b.aliases.len());
    }
    println!("fx {}, tracers {}, impact tables {}", c.fx.len(), c.tracers.len(), c.impact_tables.len());
    println!("string tables: {}", c.string_tables.iter().map(|t| format!("{} {}x{}", t.name, t.rows, t.columns)).collect::<Vec<_>>().join(", "));
    println!("localize {}", c.localize.len());
    println!("raw files: {}", c.raw_files.iter().map(|(n, b)| format!("{n} ({} bytes)", b.len())).collect::<Vec<_>>().join(", "));
    let left_out = asset_t6::t6_mod::sanitize_capture(&mut c);
    println!("left out of a match: {}", left_out.join("; "));
}
