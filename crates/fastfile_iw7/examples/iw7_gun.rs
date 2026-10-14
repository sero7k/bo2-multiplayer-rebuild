//! Everything one Infinite Warfare gun needs, and where each piece is:
//! `iw7_gun <sample folder> <weapon name>` (e.g. `iw7_m8_mp`).
//!
//! Walks `iw7_global_core_mp.ff` (the weapon defs), `iw7_common_core_mp.ff`
//! (the real anim/sfx/vfx packages, the sound bank) and the gun's transient
//! zones (`iw7_mp_weapon_{view,world}_<gun>_tr.ff`: models, materials,
//! images), plus `iw7_common_mp.ff` / `iw7_patch_common_core_mp.ff` when
//! present, and prints the gun's manifest: models -> surfaces' materials ->
//! images (inline pixels or `imagefile<N>.pak` streams), anim packages ->
//! the 260 slots' animations (data in a zone or not), sound aliases (and the
//! sound files they name), effects and attachments.
//!
//! Names starting with "," are placeholders: each is resolved to the asset
//! of the same name (without the comma) in whichever zone has the real one.
//! `IW7_GUN_ALL_SLOTS=1` also lists the empty anim slots.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use fastfile_iw7::AssetType as T;
use fastfile_iw7::walk::{AssetId, Capture, WeaponField, WeaponRec, walk_zone};

/// `weapAnimFiles_t` (x64-zt `iw7/assets/weapon_anim_package.cpp`).
const ANIM_SLOTS: [&str; 260] = include!("iw7_gun_anim_slots.in");

/// Field names by offset, from the generated layouts (`src/layout.rs`).
struct Layouts(HashMap<String, Vec<(usize, String)>>);

impl Layouts {
    fn parse() -> Self {
        let src = include_str!("../src/layout.rs");
        let mut map: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        let mut cur: Option<String> = None;
        for line in src.lines() {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("pub mod ") {
                cur = Some(rest.trim_end_matches(" {").to_owned());
            } else if l == "}" {
                cur = None;
            } else if let (Some(m), Some(rest)) = (&cur, l.strip_prefix("pub const "))
                && let Some((name, val)) = rest.split_once(": usize = ")
                && name != "SIZE"
                && name != "ALIGN"
            {
                let off: usize = val.trim_end_matches(';').parse().unwrap_or(0);
                map.entry(m.clone()).or_default().push((off, name.to_owned()));
            }
        }
        Self(map)
    }

    /// The field at `off` of `strukt`: the last top-level field at or before it.
    fn field(&self, strukt: &str, off: usize) -> String {
        let Some(fields) = self.0.get(strukt) else {
            return format!("+{off}");
        };
        let best = fields
            .iter()
            .filter(|(o, n)| *o <= off && !n.contains("__"))
            .max_by_key(|(o, _)| *o);
        match best {
            Some((o, n)) if *o == off => n.clone(),
            Some((o, n)) => format!("{n}+{}", off - o),
            None => format!("+{off}"),
        }
    }
}

struct ZoneCap {
    label: String,
    cap: Capture,
    stop: Option<String>,
}

/// Where a name resolves: the zone, the asset, and whether only
/// placeholders were found.
#[derive(Clone, Copy)]
struct Hit {
    zone: usize,
    id: AssetId,
}

struct World {
    zones: Vec<ZoneCap>,
    /// (type, name without comma) -> every capture of it, real ones first.
    index: HashMap<(T, String), Vec<(Hit, bool)>>,
    /// sound alias name -> (zone, bank index, alias index)
    aliases: HashMap<String, (usize, usize, usize)>,
}

/// A string that looks like a name (not a reference the walk resolved to
/// the wrong place).
fn valid_name(s: &str) -> bool {
    s.len() >= 3 && s.bytes().all(|b| b.is_ascii_graphic())
}

fn bare(name: &str) -> &str {
    name.strip_prefix(',').unwrap_or(name)
}

/// Whether a capture has the asset's contents (IW7 also prefixes real,
/// fully loaded materials and technique sets with ","; a placeholder is a
/// header with only its name).
fn has_content(c: &Capture, id: AssetId) -> bool {
    let i = id.index as usize;
    match id.ty {
        T::Material => c.materials[i].techset.is_some() || !c.materials[i].textures.is_empty(),
        T::TechniqueSet => !c.techsets[i].techniques.is_empty(),
        T::XModel => c.xmodels[i].num_bones > 0 || c.xmodels[i].numsurfs > 0,
        T::XModelSurfs => !c.model_surfs[i].surfs.is_empty(),
        T::Image => {
            let im = &c.images[i];
            im.pixels.is_some() || im.streamed || im.data_len1 > 0
        }
        T::XAnimParts => {
            let a = &c.xanims[i];
            a.numframes > 0 || a.bone_count[9] > 0 || a.data_byte.is_some() || a.data_short.is_some() || a.delta.is_some()
        }
        // Packages: a comma-less one is real even when empty.
        T::AnimPackage => !c.anim_packages[i].name.starts_with(',') || c.anim_packages[i].anims.iter().any(|a| a.is_some()),
        T::SfxPackage => !c.sfx_packages[i].name.starts_with(',') || !c.sfx_packages[i].sounds.is_empty(),
        T::VfxPackage => !c.vfx_packages[i].name.starts_with(',') || !c.vfx_packages[i].effects.is_empty(),
        T::Weapon => c.weapons[i].def.is_some(),
        T::Attachment => !c.attachments[i].name.starts_with(',') || !c.attachments[i].assets.is_empty(),
        T::Fx => !c.fx[i].elems.is_empty(),
        _ => {
            let n = c.name_of(id);
            !n.is_empty() && !n.starts_with(',')
        }
    }
}

fn kind(ty: T) -> T {
    // Effects are referenced as either kind.
    if ty == T::Fx { T::Vfx } else { ty }
}

impl World {
    fn new(zones: Vec<ZoneCap>) -> Self {
        let mut index: HashMap<(T, String), Vec<(Hit, bool)>> = HashMap::new();
        let mut aliases = HashMap::new();
        for (z, zc) in zones.iter().enumerate() {
            let c = &zc.cap;
            let mut add = |ty: T, n: usize| {
                for i in 0..n {
                    let id = AssetId { ty, index: i as u32 };
                    let name = c.name_of(id);
                    let real = has_content(c, id);
                    index.entry((kind(ty), bare(name).to_owned())).or_default().push((Hit { zone: z, id }, real));
                }
            };
            add(T::XModel, c.xmodels.len());
            add(T::XModelSurfs, c.model_surfs.len());
            add(T::Material, c.materials.len());
            add(T::Image, c.images.len());
            add(T::TechniqueSet, c.techsets.len());
            add(T::XAnimParts, c.xanims.len());
            add(T::Weapon, c.weapons.len());
            add(T::Attachment, c.attachments.len());
            add(T::AnimPackage, c.anim_packages.len());
            add(T::SfxPackage, c.sfx_packages.len());
            add(T::VfxPackage, c.vfx_packages.len());
            add(T::Fx, c.fx.len());
            for (i, o) in c.others.iter().enumerate() {
                if let Some(ty) = o.ty {
                    let real = !o.name.starts_with(',') && !o.name.is_empty();
                    index
                        .entry((kind(ty), bare(&o.name).to_owned()))
                        .or_default()
                        .push((Hit { zone: z, id: AssetId { ty, index: i as u32 } }, real));
                }
            }
            // Patch zones load last: their aliases override.
            let patch = zc.label.starts_with("patch_");
            for (b, bank) in c.sound_banks.iter().enumerate() {
                for (a, al) in bank.aliases.iter().enumerate() {
                    if !valid_name(&al.name) {
                        continue;
                    }
                    if patch {
                        aliases.insert(al.name.clone(), (z, b, a));
                    } else {
                        aliases.entry(al.name.clone()).or_insert((z, b, a));
                    }
                }
            }
        }
        for v in index.values_mut() {
            // Real first (stable: zone order is the priority).
            v.sort_by_key(|(_, real)| !*real);
        }
        Self { zones, index, aliases }
    }

    fn name(&self, h: Hit) -> &str {
        self.zones[h.zone].cap.name_of(h.id)
    }

    /// The real asset a capture (maybe a placeholder) stands for.
    fn resolve(&self, h: Hit) -> Option<Hit> {
        self.resolve_name(h.id.ty, self.name(h))
    }

    fn resolve_name(&self, ty: T, name: &str) -> Option<Hit> {
        let v = self.index.get(&(kind(ty), bare(name).to_owned()))?;
        v.iter().find(|(_, real)| *real).map(|(h, _)| *h)
    }

    /// `tag`, except that a material found only as a name-only header is
    /// marked as such (IW7 keeps material definitions in the shared asset
    /// streams).
    fn status(&self, ty: T, name: &str, r: Option<Hit>) -> String {
        if r.is_none() && ty == T::Material && self.index.contains_key(&(T::Material, bare(name).to_owned())) {
            return "[name-only header; definition in the shared asset streams]".to_owned();
        }
        self.tag(r)
    }

    /// "name [zone]" or "name [MISSING: placeholder only]".
    fn tag(&self, r: Option<Hit>) -> String {
        match r {
            Some(h) => format!("[{}]", self.zones[h.zone].label),
            None => "[MISSING: placeholder only]".to_owned(),
        }
    }
}

fn dxgi(f: u32) -> String {
    let n = match f {
        2 => "R32G32B32A32_FLOAT",
        10 => "R16G16B16A16_FLOAT",
        11 => "R16G16B16A16_UNORM",
        24 => "R10G10B10A2_UNORM",
        26 => "R11G11B10_FLOAT",
        28 => "R8G8B8A8_UNORM",
        29 => "R8G8B8A8_UNORM_SRGB",
        34 => "R16G16_FLOAT",
        35 => "R16G16_UNORM",
        41 => "R32_FLOAT",
        49 => "R8G8_UNORM",
        54 => "R16_FLOAT",
        56 => "R16_UNORM",
        61 => "R8_UNORM",
        65 => "A8_UNORM",
        71 => "BC1_UNORM",
        72 => "BC1_UNORM_SRGB",
        74 => "BC2_UNORM",
        75 => "BC2_UNORM_SRGB",
        77 => "BC3_UNORM",
        78 => "BC3_UNORM_SRGB",
        80 => "BC4_UNORM",
        81 => "BC4_SNORM",
        83 => "BC5_UNORM",
        84 => "BC5_SNORM",
        87 => "B8G8R8A8_UNORM",
        91 => "B8G8R8A8_UNORM_SRGB",
        95 => "BC6H_UF16",
        96 => "BC6H_SF16",
        98 => "BC7_UNORM",
        99 => "BC7_UNORM_SRGB",
        _ => return format!("DXGI({f})"),
    };
    n.to_owned()
}

#[derive(Default)]
struct Totals {
    models: BTreeSet<String>,
    models_missing: BTreeSet<String>,
    materials: BTreeSet<String>,
    materials_missing: BTreeSet<String>,
    /// Name-only in every zone: the definition is in assetfile*.pak.
    materials_shared: BTreeSet<String>,
    images_inline: BTreeMap<String, u64>,
    images_streamed: BTreeMap<String, u64>,
    images_missing: BTreeSet<String>,
    /// imagefile index -> bytes
    pak_bytes: BTreeMap<u16, u64>,
    surfs_inline: usize,
    surfs_no_verts: usize,
    anims_present: BTreeSet<String>,
    anims_empty: BTreeSet<String>,
    anims_missing: BTreeSet<String>,
    aliases_found: BTreeSet<String>,
    aliases_missing: BTreeSet<String>,
    aliases_unresolved: usize,
    sound_files_loaded: BTreeSet<String>,
    sound_files_streamed: BTreeSet<String>,
    effects: BTreeSet<String>,
    effects_missing: BTreeSet<String>,
}

fn image(wd: &World, img: Option<Hit>, name: &str, t: &mut Totals, indent: &str) {
    let r = img.and_then(|h| wd.resolve(h)).or_else(|| wd.resolve_name(T::Image, name));
    let Some(r) = r else {
        println!("{indent}image {} {}", bare(name), wd.tag(None));
        t.images_missing.insert(bare(name).to_owned());
        return;
    };
    let im = &wd.zones[r.zone].cap.images[r.id.index as usize];
    // A streamed image's header holds its smallest (inline) size; its full
    // size is the biggest stream's.
    let (w, h) = im.streams.iter().filter(|s| s.0 > 0).map(|s| (s.0, s.1)).max().unwrap_or((im.width, im.height));
    let base = format!(
        "{indent}image {} {} {} {}x{}{} levels {}",
        im.name,
        wd.tag(Some(r)),
        dxgi(im.format),
        w,
        h,
        if im.depth > 1 { format!("x{}", im.depth) } else { String::new() },
        im.level_count
    );
    if im.streamed {
        let total: u64 = im.stream_files.iter().flatten().map(|s| s.offset_end.saturating_sub(s.offset)).sum();
        println!("{base} STREAMED ({total} bytes in .pak, as stored; header level count {}, inline data {} bytes)", im.level_count, im.data_len1);
        for (k, (s, f)) in im.streams.iter().zip(im.stream_files.iter()).enumerate() {
            if s.0 == 0 && f.is_none_or(|f| f.offset_end == f.offset) {
                continue;
            }
            match f {
                Some(f) => {
                    println!(
                        "{indent}    stream {k}: {}x{} {} levels, {} bytes -> imagefile{}.pak [{:#x}..{:#x}] ({} bytes){}",
                        s.0,
                        s.1,
                        s.3,
                        s.2,
                        f.file_index,
                        f.offset,
                        f.offset_end,
                        f.offset_end.saturating_sub(f.offset),
                        if f.localized { " localized" } else { "" }
                    );
                    *t.pak_bytes.entry(f.file_index).or_default() += f.offset_end.saturating_sub(f.offset);
                }
                None => println!("{indent}    stream {k}: {}x{} {} levels, {} bytes (no stream file)", s.0, s.1, s.3, s.2),
            }
        }
        t.images_streamed.insert(im.name.clone(), total);
    } else {
        let px = im.pixels.map_or(0, |p| p.len);
        println!("{base} inline ({px} bytes in zone)");
        t.images_inline.insert(im.name.clone(), px as u64);
    }
}

fn material(wd: &World, m: Option<Hit>, name: &str, t: &mut Totals, indent: &str) {
    let r = m.and_then(|h| wd.resolve(h)).or_else(|| wd.resolve_name(T::Material, name));
    let Some(r) = r else {
        let key = bare(name).to_owned();
        let zones: BTreeSet<&str> = wd
            .index
            .get(&(T::Material, key.clone()))
            .map(|v| v.iter().map(|(h, _)| wd.zones[h.zone].label.as_str()).collect())
            .unwrap_or_default();
        if zones.is_empty() {
            println!("{indent}material {key} [not in any walked zone]");
            t.materials_missing.insert(key);
        } else {
            let z: Vec<&str> = zones.into_iter().collect();
            println!("{indent}material {key} [name-only header in {}; definition in the shared asset streams]", z.join(", "));
            t.materials_shared.insert(key);
        }
        return;
    };
    let cap = &wd.zones[r.zone].cap;
    let mat = &cap.materials[r.id.index as usize];
    t.materials.insert(bare(&mat.name).to_owned());
    let techset = mat.techset.map(|id| bare(cap.name_of(id)).to_owned()).unwrap_or_default();
    println!("{indent}material {} {} techset {techset}", bare(&mat.name), wd.tag(Some(r)));
    for tex in &mat.textures {
        let name = tex.image.map(|id| cap.name_of(id).to_owned()).unwrap_or_default();
        let hit = tex.image.map(|id| Hit { zone: r.zone, id });
        image(wd, hit, &name, t, &format!("{indent}    "));
    }
}

fn model(wd: &World, h: Option<Hit>, name: &str, t: &mut Totals, seen: &mut BTreeSet<String>) {
    let key = bare(name).to_owned();
    if !seen.insert(key.clone()) {
        println!("    (listed above)");
        return;
    }
    let r = h.and_then(|h| wd.resolve(h)).or_else(|| wd.resolve_name(T::XModel, name));
    let Some(r) = r else {
        println!("    model {key} {}", wd.tag(None));
        t.models_missing.insert(key);
        return;
    };
    t.models.insert(key.clone());
    let cap = &wd.zones[r.zone].cap;
    let m = &cap.xmodels[r.id.index as usize];
    println!(
        "    model {key} {} bones {} surfs {} lods {} radius {:.1}",
        wd.tag(Some(r)),
        m.num_bones,
        m.numsurfs,
        m.num_lods,
        m.radius
    );
    for (i, lod) in m.lods.iter().enumerate().take(m.num_lods as usize) {
        let surfs = lod.surfs.map(|id| Hit { zone: r.zone, id });
        let sr = surfs.and_then(|h| wd.resolve(h));
        let desc = match sr {
            Some(s) => {
                let ms = &wd.zones[s.zone].cap.model_surfs[s.id.index as usize];
                let verts: usize = ms.surfs.iter().map(|x| x.vert_count as usize).sum();
                let tris: usize = ms.surfs.iter().map(|x| x.tri_count as usize).sum();
                let inline = ms.surfs.iter().filter(|x| x.verts.is_some()).count();
                t.surfs_inline += inline;
                t.surfs_no_verts += ms.surfs.len() - inline;
                format!(
                    "{} {} surfs {} verts {verts} tris {tris} (vertex data inline for {inline}/{})",
                    ms.name,
                    wd.tag(Some(s)),
                    ms.surfs.len(),
                    ms.surfs.len()
                )
            }
            None => format!("{} {}", surfs.map(|h| wd.name(h).to_owned()).unwrap_or_default(), wd.tag(None)),
        };
        println!("      lod {i} dist {:.0}: {desc}", lod.dist);
    }
    // Surfaces by material (in first-use order).
    let mut order: Vec<(String, Option<Hit>, Vec<usize>)> = Vec::new();
    for (i, mid) in m.materials.iter().enumerate() {
        let name = mid.map(|id| bare(cap.name_of(id)).to_owned()).unwrap_or_default();
        match order.iter_mut().find(|(n, _, _)| *n == name) {
            Some(e) => e.2.push(i),
            None => order.push((name, mid.map(|id| Hit { zone: r.zone, id }), vec![i])),
        }
    }
    for (name, mh, surfs) in order {
        let list: Vec<String> = surfs.iter().map(|s| s.to_string()).collect();
        print!("      surfs {}: ", list.join(","));
        material(wd, mh, &name, t, "");
    }
}

fn xanim_line(wd: &World, h: Hit, t: &mut Totals) -> String {
    let name = bare(wd.name(h)).to_owned();
    match wd.resolve(h) {
        None => {
            t.anims_missing.insert(name.clone());
            format!("{name} [MISSING: placeholder only]")
        }
        Some(r) => {
            let a = &wd.zones[r.zone].cap.xanims[r.id.index as usize];
            let sizes = [a.data_byte, a.data_short, a.data_int, a.random_data_byte, a.random_data_short, a.random_data_int, a.indices];
            let bytes: usize = sizes.iter().flatten().map(|s| s.len).sum();
            let delta = a.delta.as_ref().map_or(0, |d| {
                d.trans.map_or(0, |x| x.2.len) + d.quat.map_or(0, |x| x.1.len) + d.quat2.map_or(0, |x| x.1.len)
            });
            let bones = a.bone_count[9];
            if bytes + delta == 0 && bones == 0 && a.numframes == 0 {
                t.anims_empty.insert(name.clone());
            } else {
                t.anims_present.insert(name.clone());
            }
            format!(
                "{name} {} frames {} @{:.0}fps bones {bones} notifies {} data {bytes} B (byte {} short {} int {} rnd b/s/i {}/{}/{} idx {}) delta {delta} B",
                wd.tag(Some(r)),
                a.numframes,
                a.framerate,
                a.notify_count,
                a.data_byte_count,
                a.data_short_count,
                a.data_int_count,
                a.random_data_byte_count,
                a.random_data_short_count,
                a.random_data_int_count,
                a.index_count
            )
        }
    }
}

/// A zone whose walk stopped before its sound bank: finds the bank's header
/// after the stop (a header whose name string, "<zone>.<language>", follows
/// it) and walks from there to the zone's end.
fn recover_sound_bank(zone: &fastfile_iw7::Zone, stopped: usize, from: usize) -> Option<fastfile_iw7::walk::SoundBankRec> {
    let i = zone.assets.assets.iter().enumerate().skip(stopped).rev().find(|(_, a)| a.asset_type() == Some(T::SoundBank))?.0;
    let img = &zone.image;
    let size = T::SoundBank.header_size();
    let fe = [0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    for p in from..img.len().saturating_sub(size + 8) {
        if img[p..p + 8] != fe {
            continue;
        }
        let name = &img[p + size..];
        let Some(n) = name.iter().take(128).position(|&b| b == 0) else {
            continue;
        };
        let name = &name[..n];
        if n < 3 || !name.iter().all(|b| b.is_ascii_graphic()) || !name.contains(&b'.') {
            continue;
        }
        let mut w = fastfile_iw7::walk::Walker::new(img, &zone.assets, &zone.header.stream_files);
        if w.walk_from(Some((i, p))).is_ok() && w.stream_pos() == img.len() {
            return w.cap.sound_banks.pop();
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: iw7_gun <sample folder> <weapon name, e.g. iw7_m8_mp>");
        std::process::exit(2);
    }
    let dir = Path::new(&args[1]);
    let weapon = &args[2];
    let gun = weapon.strip_suffix("_mp").unwrap_or(weapon);
    let all_slots = std::env::var_os("IW7_GUN_ALL_SLOTS").is_some();
    let lay = Layouts::parse();

    // Priority order for resolving names: the gun's own zones first.
    let names = [
        format!("iw7_mp_weapon_view_{gun}_tr.ff"),
        format!("iw7_mp_weapon_world_{gun}_tr.ff"),
        "iw7_common_core_mp.ff".to_owned(),
        "iw7_global_core_mp.ff".to_owned(),
        "iw7_patch_common_core_mp.ff".to_owned(),
        "iw7_global_mp.ff".to_owned(),
        "iw7_patch_common_mp.ff".to_owned(),
        "iw7_common_mp.ff".to_owned(),
    ];
    let mut zones = Vec::new();
    for n in &names {
        let p = dir.join(n);
        if !p.exists() {
            println!("zone {n}: not present");
            continue;
        }
        let zone = match fastfile_iw7::Zone::open(&p) {
            Ok(z) => z,
            Err(e) => {
                println!("zone {n}: {e}");
                continue;
            }
        };
        let (mut cap, stop) = walk_zone(&zone);
        let mut recovered = None;
        if let Some(st) = &stop
            && let Some(bank) = recover_sound_bank(&zone, st.asset_index, st.file_pos)
        {
            recovered = Some(bank.name.clone());
            cap.sound_banks.push(bank);
        }
        let total = zone.assets.assets.len();
        let stop = stop.map(|s| format!("{:?} at {} ({})", s.asset_type, s.asset_index, s.error));
        println!("zone {n}: walked {}/{total}{}", cap.walked, stop.as_ref().map_or(String::new(), |s| format!(" STOPPED {s}")));
        if let Some(b) = recovered {
            println!("    (its sound bank {b} was found after the stop and read on its own)");
        }
        {
            let h = &zone.header;
            let mut by: BTreeMap<String, (usize, u64)> = BTreeMap::new();
            for (kind, files) in [("assetfile", &h.shared_files), ("imagefile", &h.stream_files)] {
                for f in files.iter() {
                    let e = by.entry(format!("{kind}{}.pak", f.file_index)).or_default();
                    e.0 += 1;
                    e.1 += f.offset_end.saturating_sub(f.offset);
                }
            }
            let list: Vec<String> = by.iter().map(|(k, (n, b))| format!("{k}: {n} ranges {b} B")).collect();
            println!(
                "    shared assets: {} ranges ({:#x} VIRTUAL bytes); image streams: {} ranges; {}",
                h.shared_files.len(),
                h.shared_virtual,
                h.stream_files.len(),
                list.join(", ")
            );
        }
        for b in &cap.sound_banks {
            let entries: usize = b.aliases.iter().map(|a| a.entries.len()).sum();
            let streamed = b.aliases.iter().flat_map(|a| &a.entries).filter(|e| e.1 != 0).count();
            println!("    sound bank {} (zone {}, language {}/{}): {} aliases, {entries} entries ({streamed} streamed/primed)", b.name, b.zone, b.game_language, b.sound_language, b.aliases.len());
            if std::env::var_os("IW7_GUN_DEBUG").is_some() {
                for a in b.aliases.iter().take(20) {
                    eprintln!("      {} {:?}", a.name, a.entries);
                }
            }
        }
        if std::env::var_os("IW7_GUN_DEBUG").is_some() {
            let with = cap.materials.iter().filter(|m| m.techset.is_some() || !m.textures.is_empty()).count();
            let comma = cap.materials.iter().filter(|m| m.name.starts_with(',')).count();
            eprintln!("  materials with content {with}, comma {comma}");
            let ic = cap.images.iter().filter(|m| m.name.starts_with(',')).count();
            let ip = cap.images.iter().filter(|m| m.pixels.is_some() || m.streamed).count();
            eprintln!("  images comma {ic} with data {ip}");
            for m in cap.materials.iter().filter(|m| m.techset.is_some()).take(3) {
                eprintln!("  mat {} techset {:?} tex {}", m.name, m.techset.map(|t| cap.name_of(t).to_owned()), m.textures.len());
            }
            eprintln!("  materials {} images {} xmodels {} xanims {}", cap.materials.len(), cap.images.len(), cap.xmodels.len(), cap.xanims.len());
        }
        let label = n.trim_start_matches("iw7_").trim_end_matches(".ff").to_owned();
        zones.push(ZoneCap { label, cap, stop });
    }
    let wd = World::new(zones);
    let _ = wd.zones.iter().map(|z| &z.stop).count();

    // The weapon: the real def (not a placeholder), first zone that has it.
    let mut found: Option<(usize, &WeaponRec)> = None;
    for (z, zc) in wd.zones.iter().enumerate() {
        if let Some(w) = zc.cap.weapons.iter().find(|w| w.name == *weapon) {
            found = Some((z, w));
            break;
        }
    }
    let Some((wz, w)) = found else {
        println!("weapon {weapon} not found");
        std::process::exit(1);
    };
    let wcap = &wd.zones[wz].cap;
    let at = |h: AssetId| Hit { zone: wz, id: h };
    let field = |f: &WeaponField| match *f {
        WeaponField::Def(o) => lay.field("WeaponDef", o),
        WeaponField::Complete(o) => lay.field("WeaponCompleteDef", o),
        WeaponField::DefArray(o, i) => format!("{}[{i}]", lay.field("WeaponDef", o)),
    };
    println!();
    println!("WEAPON {} display \"{}\" [{}]", w.name, w.display_name, wd.zones[wz].label);
    for (f, s) in &w.strings {
        if !s.is_empty() {
            println!("  {} = \"{s}\"", field(f));
        }
    }

    let mut t = Totals::default();

    // ---- models ----
    println!();
    println!("MODELS");
    let mut seen = BTreeSet::new();
    let mut camo: Vec<(String, AssetId)> = Vec::new();
    for (f, id) in &w.assets {
        match id.ty {
            T::XModel => {
                println!("  {}:", field(f));
                model(&wd, Some(at(*id)), wcap.name_of(*id), &mut t, &mut seen);
            }
            T::Material => camo.push((field(f), *id)),
            _ => {}
        }
    }
    println!();
    println!("MATERIALS (camo overrides, icons, reticles)");
    for (f, id) in &camo {
        print!("  {f}: ");
        material(&wd, Some(at(*id)), wcap.name_of(*id), &mut t, "  ");
    }

    // ---- images ----
    // Materials are name-only in the zones (their texture tables are in the
    // shared asset streams), so the images come from the gun's transient
    // zones' image lists (their streaming info): what its materials sample.
    println!();
    println!("IMAGES (the gun's transient zones' image lists; materials' texture tables are not in the zones)");
    let mut seen_img = BTreeSet::new();
    for (z, zc) in wd.zones.iter().enumerate() {
        if !zc.label.starts_with("mp_weapon_") {
            continue;
        }
        println!("  {}:", zc.label);
        for (i, im) in zc.cap.images.iter().enumerate() {
            if !seen_img.insert(bare(&im.name).to_owned()) {
                continue;
            }
            let id = AssetId { ty: T::Image, index: i as u32 };
            image(&wd, Some(Hit { zone: z, id }), &im.name, &mut t, "    ");
        }
    }

    // ---- anim packages ----
    println!();
    println!("ANIM PACKAGES");
    let mut done = BTreeSet::new();
    for (f, id) in w.assets.iter().filter(|(_, id)| id.ty == T::AnimPackage) {
        let name = bare(wcap.name_of(*id)).to_owned();
        let r = wd.resolve(at(*id));
        println!("  {}: {name} {}", field(f), wd.tag(r));
        if !done.insert(name) {
            continue;
        }
        let Some(r) = r else { continue };
        let pcap = &wd.zones[r.zone].cap;
        let p = &pcap.anim_packages[r.id.index as usize];
        let used = p.anims.iter().filter(|a| a.is_some()).count();
        println!("    {used}/{} slots used", p.anims.len());
        for (i, a) in p.anims.iter().enumerate() {
            let slot = ANIM_SLOTS.get(i).map_or("?", |s| s.trim_start_matches("WEAP_"));
            match a {
                Some(a) => println!("    {i:3} {slot:<40} {}", xanim_line(&wd, Hit { zone: r.zone, id: *a }, &mut t)),
                None if all_slots => println!("    {i:3} {slot:<40} -"),
                None => {}
            }
        }
    }

    // ---- sounds ----
    println!();
    println!("SFX PACKAGES");
    let mut done = BTreeSet::new();
    for (f, id) in w.assets.iter().filter(|(_, id)| id.ty == T::SfxPackage) {
        let name = bare(wcap.name_of(*id)).to_owned();
        let r = wd.resolve(at(*id));
        println!("  {}: {name} {}", field(f), wd.tag(r));
        if !done.insert(name) {
            continue;
        }
        let Some(r) = r else { continue };
        let p = &wd.zones[r.zone].cap.sfx_packages[r.id.index as usize];
        for (o, alias) in &p.sounds {
            if alias.is_empty() {
                continue;
            }
            let fname = lay.field("WeaponSFXPackageSounds", *o);
            if !valid_name(alias) {
                println!("    {fname:<28} <string reference the walk did not resolve>");
                t.aliases_unresolved += 1;
                continue;
            }
            match wd.aliases.get(alias) {
                Some(&(z, b, a)) => {
                    t.aliases_found.insert(alias.clone());
                    let bank = &wd.zones[z].cap.sound_banks[b];
                    let al = &bank.aliases[a];
                    let files: Vec<String> = al
                        .entries
                        .iter()
                        .map(|(file, ty, _)| {
                            let kind = ["loaded", "hybrid", "streamed", "primed"][*ty as usize & 3];
                            let f = file.clone().unwrap_or_else(|| "(no file)".into());
                            if file.is_some() {
                                if *ty == 0 {
                                    t.sound_files_loaded.insert(f.clone());
                                } else {
                                    t.sound_files_streamed.insert(f.clone());
                                }
                            }
                            format!("{f} ({kind})")
                        })
                        .collect();
                    println!("    {fname:<28} {alias} [bank {}] -> {}", bank.name, files.join(", "));
                }
                None => {
                    t.aliases_missing.insert(alias.clone());
                    println!("    {fname:<28} {alias} [alias not in any walked bank]");
                }
            }
        }
    }
    let other_sounds: Vec<_> = w.strings.iter().filter(|(_, s)| !s.is_empty() && wd.aliases.contains_key(s)).collect();
    for (f, s) in other_sounds {
        println!("  {}: alias {s}", field(f));
    }

    // ---- effects ----
    println!();
    println!("VFX PACKAGES");
    let mut done = BTreeSet::new();
    for (f, id) in w.assets.iter().filter(|(_, id)| id.ty == T::VfxPackage) {
        let name = bare(wcap.name_of(*id)).to_owned();
        let r = wd.resolve(at(*id));
        println!("  {}: {name} {}", field(f), wd.tag(r));
        if !done.insert(name) {
            continue;
        }
        let Some(r) = r else { continue };
        let pcap = &wd.zones[r.zone].cap;
        let p = &pcap.vfx_packages[r.id.index as usize];
        for (o, e) in &p.effects {
            let ename = bare(pcap.name_of(*e)).to_owned();
            let er = wd.resolve(Hit { zone: r.zone, id: *e });
            if er.is_some() {
                t.effects.insert(ename.clone());
            } else {
                t.effects_missing.insert(ename.clone());
            }
            println!("    {:<28} {} {ename} {}", lay.field("WeaponVFXPackageDefs", *o), e.ty.name(), wd.tag(er));
        }
    }
    for (f, id) in w.assets.iter().filter(|(_, id)| matches!(id.ty, T::Vfx | T::Fx)) {
        let r = wd.resolve(at(*id));
        println!("  {}: {} {} {}", field(f), id.ty.name(), bare(wcap.name_of(*id)), wd.tag(r));
    }

    // ---- other assets ----
    println!();
    println!("OTHER");
    for (f, id) in &w.assets {
        if matches!(id.ty, T::XModel | T::Material | T::AnimPackage | T::SfxPackage | T::VfxPackage | T::Vfx | T::Fx) {
            continue;
        }
        let r = wd.resolve(at(*id));
        println!("  {}: {} {} {}", field(f), id.ty.name(), bare(wcap.name_of(*id)), wd.status(id.ty, wcap.name_of(*id), r));
    }

    // ---- attachments ----
    println!();
    println!("ATTACHMENTS");
    for (k, list) in w.attachments.iter().enumerate() {
        for id in list.iter().flatten() {
            let name = bare(wcap.name_of(*id)).to_owned();
            let r = wd.resolve(at(*id));
            let mut line = format!("  attachments{} {name} {}", k + 1, wd.tag(r));
            if let Some(r) = r {
                let acap = &wd.zones[r.zone].cap;
                let a = &acap.attachments[r.id.index as usize];
                let dn = if a.display_name.chars().all(|c| c.is_ascii_graphic() || c == ' ') { a.display_name.as_str() } else { "<unresolved string ref>" };
                line += &format!(" \"{dn}\"");
                println!("{line}");
                let entries = a
                    .assets
                    .iter()
                    .map(|(o, id)| (lay.field("WeaponAttachment", *o), *id))
                    .chain(a.arrays.iter().map(|(o, i, id)| (format!("{}[{i}]", lay.field("WeaponAttachment", *o)), *id)));
                for (label, aid) in entries {
                    let aid = &aid;
                    let an = bare(acap.name_of(*aid)).to_owned();
                    let ar = wd.resolve(Hit { zone: r.zone, id: *aid });
                    println!("      {label:<32} {} {an} {}", aid.ty.name(), wd.status(aid.ty, &an, ar));
                    if aid.ty == T::XModel {
                        if ar.is_some() {
                            t.models.insert(an);
                        } else {
                            t.models_missing.insert(an);
                        }
                    }
                }
            } else {
                println!("{line}");
            }
        }
    }

    // ---- summary ----
    println!();
    println!("SUMMARY for {weapon}");
    println!("  models: {} found, {} missing {:?}", t.models.len(), t.models_missing.len(), t.models_missing);
    println!("  surfaces: vertex data inline {} / not in zone {}", t.surfs_inline, t.surfs_no_verts);
    println!(
        "  materials: {} with a definition in a zone, {} name-only (definition in the shared asset streams), {} not in any zone {:?}",
        t.materials.len(),
        t.materials_shared.len(),
        t.materials_missing.len(),
        t.materials_missing
    );
    let inl: u64 = t.images_inline.values().sum();
    let st: u64 = t.images_streamed.values().sum();
    println!(
        "  images: {} inline ({inl} bytes in zones), {} streamed ({st} bytes in imagefile*.pak), {} missing {:?}",
        t.images_inline.len(),
        t.images_streamed.len(),
        t.images_missing.len(),
        t.images_missing
    );
    for (f, b) in &t.pak_bytes {
        println!("    imagefile{f}.pak: {b} bytes");
    }
    println!(
        "  xanims: {} with data in a zone, {} empty headers, {} missing (placeholder only)",
        t.anims_present.len(),
        t.anims_empty.len(),
        t.anims_missing.len()
    );
    for n in t.anims_missing.iter().take(400) {
        println!("    missing xanim {n}");
    }
    for n in &t.anims_empty {
        println!("    empty xanim {n}");
    }
    println!(
        "  sound aliases: {} found in banks, {} not found, {} unresolved string refs; sound files: {} loaded (.sabl), {} streamed/primed (.sabs)",
        t.aliases_found.len(),
        t.aliases_missing.len(),
        t.aliases_unresolved,
        t.sound_files_loaded.len(),
        t.sound_files_streamed.len()
    );
    for n in &t.aliases_missing {
        println!("    alias not found {n}");
    }
    println!("  effects: {} found, {} missing {:?}", t.effects.len(), t.effects_missing.len(), t.effects_missing);
}
