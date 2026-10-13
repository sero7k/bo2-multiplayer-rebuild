//! bo2zm: print a T6 zone's sound aliases whose name holds a filter.
//! `t6alias <filter> <zone.ff>...` (one line per alias: bank, name, variants;
//! T6ALIAS_ROWS=1 adds the first variant's row).
fn main() {
    let mut args = std::env::args().skip(1);
    let filter = args.next().unwrap_or_default();
    for zone in args {
        let c = match asset_t6::capture_zone(std::path::Path::new(&zone)) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{zone}: {e}");
                continue;
            }
        };
        for bank in &c.sound_banks {
            for a in bank.aliases.iter().filter(|a| a.name.contains(&filter)) {
                println!("{}: {} ({} variants)", bank.name, a.name, a.entries.len());
                // T6ALIAS_ROWS=1: the first variant's whole row (volumes,
                // distances, flags, bus).
                if std::env::var_os("T6ALIAS_ROWS").is_some()
                    && let Some(e) = a.entries.first()
                {
                    println!("  {e:?}");
                }
            }
        }
        // T6ALIAS_GROUPS=1: the zone's volume groups (index, name, parent,
        // attenuation sp and mp out of 65535).
        if std::env::var_os("T6ALIAS_GROUPS").is_some()
            && let Some(g) = &c.snd_globals
        {
            for (i, (name, parent, sp, mp)) in g.groups.iter().enumerate() {
                println!("group {i} {name} parent {parent:?} sp {sp} mp {mp}");
            }
            for m in &g.masters {
                println!(
                    "master {} eq {} comp {:?} limit {:?} bus_g {:?} bus_e {:?}",
                    m.name, m.eq_g, m.comp, m.limit, m.bus_g, m.bus_e
                );
            }
        }
    }
}
