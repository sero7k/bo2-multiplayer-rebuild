//! A map's placed static models: where each stands (the `gfx_map`'s
//! instances), which model it is, and the models' first LODs (`xmodel`
//! headers over `xmodelsurfs` meshes).
//!
//! The zone is not walked; each table is found by its shape and checked
//! against the others.
//!
//! - The static model table (`gfx_map`): 16 bytes each, a back-pointer to
//!   the model's asset-list entry (an `xmodel`, rarely another kind) and
//!   a small word of flags.
//! - The collections: 16 bytes each, `u32 first instance, u32 count,
//!   u16 static model, u16, u32 flags`; the first runs from instance 0
//!   and each starts where the one before it ends.
//! - The instances: 24 bytes each,
//!   ```text
//!   +0x00 u16[4] orientation quaternion (x, y, z, w), each v/65535*2-1
//!   +0x08 f16 scale, u16 flags
//!   +0x0c i32[3] origin, in 1/4096 units
//!   ```
//! - An `xmodel` (asset type 9) is a 0xe8-byte header (+0x00 name hash,
//!   +0x08 its name, following: the header's last bytes are followed by
//!   the name), `+0x10 u16 surfaces, u8 LODs`. Further on: the
//!   materials (one back-pointer per surface, to a material entry or a
//!   hash with the top bit set for a material of a shared zone), then the
//!   LODs, 0x48 bytes each:
//!   ```text
//!   +0x00 back-pointer to the LOD's `xmodelsurfs` entry
//!   +0x10 f32[3] distances
//!   +0x1c u16 surfaces, u16 first surface (index into the materials)
//!   ```
//!   The headers stand in the zone in the asset list's order (by name);
//!   the list's entries whose header pointer is `-1` are the ones that
//!   have one (the `-2` entries are something else and are not used here).
//! - The `xmodelsurfs` (asset type 8) are read by [`crate::surfs`]; their
//!   headers stand in the asset list's order too.

use super::index::{FOLLOW, ZoneImage, f32_at, name_hash, u16_at, u32_at, u64_at};
use crate::ptr::INSERT;

/// A placed model.
#[derive(Clone, Copy, Debug)]
pub struct Instance {
    /// Index into the static model table.
    pub smodel: usize,
    pub origin: [f32; 3],
    /// (x, y, z, w).
    pub quat: [f32; 4],
    pub scale: f32,
}

/// One LOD of a model.
#[derive(Clone, Copy, Debug)]
pub struct Lod {
    /// Its mesh: the asset-list entry of its `xmodelsurfs`.
    pub surfs: usize,
    pub distance: f32,
    pub surfaces: usize,
    pub first: usize,
}

/// A model: its name, LODs and per surface its material's entry.
#[derive(Clone, Debug, Default)]
pub struct ModelDef {
    pub name: String,
    pub lods: Vec<Lod>,
    /// Per surface (over all LODs) its material's asset-list entry.
    pub materials: Vec<Option<usize>>,
}

/// What [`read`] found.
#[derive(Default)]
pub struct Placed {
    /// The static model table: each its asset-list entry.
    pub table: Vec<Option<usize>>,
    pub instances: Vec<Instance>,
    /// By asset-list entry of a model, its definition.
    pub models: std::collections::HashMap<usize, ModelDef>,
    /// Asset-list entry of an `xmodelsurfs` -> its header in the image.
    pub surfs_at: std::collections::HashMap<usize, usize>,
}

const TABLE: usize = 16;
const COLLECTION: usize = 16;
const INSTANCE: usize = 24;
const XMODEL_HEADER: usize = 0xe8;
const LOD: usize = 0x48;

/// The asset-list entry a back-pointer names (`base`: the block offset of
/// entry 0).
pub(crate) fn entry_of(zone: &ZoneImage, base: usize, v: u64) -> Option<usize> {
    if v >> 32 != 0x0f || v & 1 == 0 {
        return None;
    }
    let off = ((v & 0xffff_ffff) as usize).checked_sub(1)?;
    let rel = off.checked_sub(base)?;
    (rel % 16 == 0 && rel / 16 < zone.types.len()).then_some(rel / 16)
}

/// The block offset of asset-list entry 0: back-pointers to entries land
/// on one residue (mod 16), the entry-0 offset is the one of that residue
/// just at or before the list's place in the image.
pub fn entry_base(zone: &ZoneImage, from: usize) -> Option<usize> {
    let z = &zone.bytes;
    let mut votes = [0usize; 16];
    let mut seen = 0;
    let mut at = from;
    let list_end = zone.list_at + 16 * zone.types.len();
    while at + 8 <= z.len() && seen < 4000 {
        if z[at + 4..at + 8] == [0x0f, 0, 0, 0] && z[at] & 1 == 1 {
            let off = (u32_at(z, at) - 1) as usize;
            if off + 0x40 >= zone.list_at && off < list_end {
                votes[off % 16] += 1;
                seen += 1;
            }
            at += 8;
            continue;
        }
        at += 1;
    }
    let r = (0..16).max_by_key(|&r| votes[r])?;
    (votes[r] > 0).then(|| {
        let b = zone.list_at - zone.list_at % 16 + r;
        if b > zone.list_at { b - 16 } else { b }
    })
}

/// The quaternion of an instance record and whether it is one.
fn instance_at(z: &[u8], at: usize) -> Option<Instance> {
    if at + INSTANCE > z.len() {
        return None;
    }
    let q: [f32; 4] = std::array::from_fn(|k| f32::from(u16_at(z, at + 2 * k)) / 65535.0 * 2.0 - 1.0);
    let n = q.iter().map(|x| x * x).sum::<f32>();
    if !(0.97..1.03).contains(&n) {
        return None;
    }
    let scale = crate::surfs::f16(u16_at(z, at + 8));
    if !(scale.is_finite() && scale > 1e-3 && scale < 1000.0) {
        return None;
    }
    let o: [f32; 3] = std::array::from_fn(|k| u32_at(z, at + 12 + 4 * k) as i32 as f32 / 4096.0);
    Some(Instance {
        smodel: 0,
        origin: o,
        quat: q,
        scale,
    })
}

/// Finds the static model table, collections and instances of the
/// `gfx_map` whose data starts at about `from`.
fn find_tables(
    zone: &ZoneImage,
    base: usize,
    from: usize,
    report: &mut Vec<String>,
) -> Option<(Vec<Option<usize>>, Vec<Instance>)> {
    let z = &zone.bytes;
    // The table: the longest run of 16-byte records whose first word is a
    // back-pointer to an entry and whose second is small.
    let rec = |at: usize| entry_of(zone, base, u64_at(z, at)).filter(|_| u64_at(z, at + 8) < 0x1_0000);
    let mut best = (0usize, 0usize);
    let mut at = from;
    while at + TABLE <= z.len() {
        if rec(at).is_some() {
            let mut n = 1;
            // (A few entries are not pointers: tolerate single gaps.)
            loop {
                if rec(at + TABLE * n).is_some() {
                    n += 1;
                } else if rec(at + TABLE * (n + 1)).is_some() && u64_at(z, at + TABLE * n + 8) < 0x1_0000 {
                    n += 2;
                } else {
                    break;
                }
            }
            if n > best.1 {
                best = (at, n);
            }
            at += TABLE * n;
            continue;
        }
        at += 1;
    }
    let (table_at, table_len) = best;
    if table_len < 8 {
        report.push("static models: no model table".into());
        return None;
    }
    let table: Vec<Option<usize>> = (0..table_len).map(|k| rec(table_at + TABLE * k)).collect();
    // The collections: the longest chain from instance 0 after the table.
    let chain = |at: usize| -> (usize, u64) {
        let mut n = 0usize;
        let mut next = 0u64;
        loop {
            let r = at + COLLECTION * n;
            if r + COLLECTION > z.len() || u64::from(u32_at(z, r)) != next || u16_at(z, r + 8) as usize >= table_len {
                break;
            }
            let c = u32_at(z, r + 4);
            if c == 0 || c > 1_000_000 {
                break;
            }
            next += u64::from(c);
            n += 1;
        }
        (n, next)
    };
    let mut col = (0usize, 0usize, 0u64);
    let mut at = table_at + TABLE * table_len;
    while at + COLLECTION <= z.len() {
        if u32_at(z, at) == 0 && u32_at(z, at + 4) != 0 {
            let (n, total) = chain(at);
            if n > col.1 {
                col = (at, n, total);
            }
        }
        at += 1;
        if at > table_at + TABLE * table_len + (16 << 20) {
            break;
        }
    }
    let (col_at, col_len, total) = col;
    if col_len == 0 {
        report.push(format!("static models: {table_len} in the table, no collections"));
        return None;
    }
    let total = total as usize;
    // The instances: the first run of `total` instance records after the
    // collections.
    let mut at = col_at + COLLECTION * col_len;
    let mut found = None;
    while at + INSTANCE * total <= z.len() {
        if instance_at(z, at).is_none() {
            at += 1;
            continue;
        }
        let bad = (0..total).find(|&k| instance_at(z, at + INSTANCE * k).is_none());
        match bad {
            None => {
                found = Some(at);
                break;
            }
            Some(k) => at += if k > 0 { INSTANCE * k } else { 1 },
        }
    }
    let Some(inst_at) = found else {
        report.push(format!(
            "static models: {table_len} in the table, {col_len} collections, {total} instances not found"
        ));
        return None;
    };
    let mut instances: Vec<Instance> = (0..total).filter_map(|k| instance_at(z, inst_at + INSTANCE * k)).collect();
    for c in 0..col_len {
        let r = col_at + COLLECTION * c;
        let first = u32_at(z, r) as usize;
        let count = u32_at(z, r + 4) as usize;
        let sm = usize::from(u16_at(z, r + 8));
        for i in instances.iter_mut().skip(first).take(count) {
            i.smodel = sm;
        }
    }
    report.push(format!(
        "static models: table {table_len} at {table_at:#x}, {col_len} collections at {col_at:#x}, {total} instances at {inst_at:#x}"
    ));
    Some((table, instances))
}

/// The `xmodel` headers: (where, name), in the image's order.
fn xmodel_headers(z: &[u8]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    // (The name pointer is -2, or -1 for a few.)
    let mut at = 8usize;
    while at + XMODEL_HEADER < z.len() {
        if z[at + 1..at + 8] != [0xff; 7] || z[at] < 0xfe {
            at += 1;
            continue;
        }
        let h = at - 8;
        let s = h + XMODEL_HEADER;
        let len = z[s..z.len().min(s + 256)].iter().position(|&b| b == 0);
        if let Some(len) = len
            && len > 0
            && z[s..s + len].iter().all(|&b| (0x20..0x7f).contains(&b))
        {
            let name = String::from_utf8_lossy(&z[s..s + len]).into_owned();
            if name_hash(&name) == u64_at(z, h) {
                out.push((h, name));
                at = s + len;
                continue;
            }
        }
        at += 1;
    }
    out
}

/// The `xmodelsurfs` headers in the image's order.
pub fn surfs_headers(z: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut at = 0usize;
    while at + crate::surfs::HEADER_LEN + crate::surfs::SURFACE_LEN <= z.len() {
        let p = u64_at(z, at + 8);
        // A header, and its first surface's shape (+0x10 0x10000, +0x3c
        // -1.0); a mesh whose rigid ranges this reader does not follow still
        // counts (for the order), its end unknown.
        let s = at + crate::surfs::HEADER_LEN;
        if (p == FOLLOW || p == INSERT)
            && u64_at(z, at + 0x18) == 0
            && (1..=1024).contains(&u32_at(z, at + 0x28))
            && u64_at(z, at) != 0
        {
            match crate::surfs::parse(z, at) {
                Ok(r) => {
                    out.push(at);
                    at = r.end.max(at + 1);
                    continue;
                }
                Err(_) if u32_at(z, s + 0x10) == 0x1_0000 && u32_at(z, s + 0x3c) == 0xbf80_0000 => {
                    out.push(at);
                    at += 1;
                    continue;
                }
                Err(_) => {}
            }
        }
        at += 1;
    }
    out
}

/// A model's LODs and materials from its header at `h` (named `name`).
fn model_def(zone: &ZoneImage, base: usize, h: usize, name: &str, limit: usize) -> Result<ModelDef, &'static str> {
    let z = &zone.bytes;
    let surfaces = usize::from(u16_at(z, h + 0x10));
    let lods = usize::from(z[h + 0x12]);
    if surfaces == 0 || lods == 0 || lods > 8 {
        return Err("surface or LOD count");
    }
    let after = h + XMODEL_HEADER + name.len() + 1;
    let is_surfs = |at: usize| entry_of(zone, base, u64_at(z, at)).filter(|&e| zone.types[e] == 8);
    // The LODs: the first a back-pointer to a mesh, each one's first
    // surface where the one before it ends, all of them the model's
    // surfaces. (A coarse LOD's mesh can be a shared zone's: a hash.)
    let lods_at = |p: usize| {
        is_surfs(p)?;
        let mut first = 0usize;
        for l in 0..lods {
            let r = p + LOD * l;
            let n = usize::from(u16_at(z, r + 0x1c));
            if usize::from(u16_at(z, r + 0x1e)) != first || n == 0 {
                return None;
            }
            first += n;
        }
        (first == surfaces).then_some(p)
    };
    let end = limit.min(z.len());
    let first_lod = (after..end.saturating_sub(LOD * lods))
        .find_map(lods_at)
        .ok_or("no LODs")?;
    let mut def = ModelDef {
        name: name.to_owned(),
        ..Default::default()
    };
    for l in 0..lods {
        let r = first_lod + LOD * l;
        let Some(surfs) = is_surfs(r) else { break };
        def.lods.push(Lod {
            surfs,
            distance: f32_at(z, r + 0x10),
            surfaces: usize::from(u16_at(z, r + 0x1c)),
            first: usize::from(u16_at(z, r + 0x1e)),
        });
    }
    // The materials: the `surfaces` words ending at the last material
    // pointer before the LODs.
    let is_material = |at: usize| entry_of(zone, base, u64_at(z, at)).is_some_and(|e| zone.types[e] == 11);
    let mut end = first_lod;
    while end >= after + 8 && end + 0x200 > first_lod && !is_material(end - 8) {
        end -= 8;
    }
    if end >= after + 8 * surfaces && is_material(end - 8) {
        let from = end - 8 * surfaces;
        def.materials = (0..surfaces)
            .map(|k| entry_of(zone, base, u64_at(z, from + 8 * k)).filter(|&e| zone.types[e] == 11))
            .collect();
    } else {
        def.materials = vec![None; surfaces];
    }
    Ok(def)
}

/// Pairs the `xmodelsurfs` entries with the headers found, both in the
/// zone's order: a few headers found are not meshes and a few meshes are
/// not found, so the two are aligned (a banded alignment, as of two
/// sequences) by the surface counts the models' LODs give their meshes.
/// Returns the pairs and the headers passed over.
fn pair_surfs(z: &[u8], entries: &[usize], heads: &[usize], expect: &std::collections::HashMap<usize, usize>) -> (Vec<(usize, usize)>, usize) {
    let (n, m) = (entries.len(), heads.len());
    if n == 0 || m == 0 {
        return (Vec::new(), m);
    }
    let want: Vec<Option<usize>> = entries.iter().map(|e| expect.get(e).copied()).collect();
    let count: Vec<usize> = heads.iter().map(|&h| u32_at(z, h + 0x28) as usize).collect();
    // Shifts j - i within the band.
    let (lo, hi) = ((m as i64 - n as i64).min(0) - 160, (m as i64 - n as i64).max(0) + 160);
    let w = (hi - lo + 1) as usize;
    const NEG: i32 = i32::MIN / 4;
    // score[i][d], from: 0 diagonal, 1 entry passed over, 2 header passed over.
    let mut from = vec![0u8; (n + 1) * w];
    let mut prev = vec![NEG; w];
    let mut cur = vec![NEG; w];
    let col = |i: usize, j: usize| -> Option<usize> {
        let d = j as i64 - i as i64;
        (lo..=hi).contains(&d).then(|| (d - lo) as usize)
    };
    // Row 0: headers passed over before the first entry.
    for j in 0..=m {
        if let Some(c) = col(0, j) {
            prev[c] = -(j as i32);
            from[c] = 2;
        }
    }
    for i in 1..=n {
        cur.fill(NEG);
        for c in 0..w {
            let j = i as i64 + lo + c as i64;
            if j < 0 || j > m as i64 {
                continue;
            }
            let j = j as usize;
            let mut best = NEG;
            let mut how = 0u8;
            // Diagonal: entry i-1 is header j-1.
            if j >= 1 && let Some(pc) = col(i - 1, j - 1) && prev[pc] > NEG {
                let s = match want[i - 1] {
                    Some(k) if k == count[j - 1] => 2,
                    Some(_) => -2,
                    None => 0,
                };
                best = prev[pc] + s;
            }
            // Entry i-1 has no header found.
            if let Some(pc) = col(i - 1, j) && prev[pc] > NEG && prev[pc] - 1 > best {
                best = prev[pc] - 1;
                how = 1;
            }
            // Header j-1 is not a mesh.
            if j >= 1 && c >= 1 && cur[c - 1] > NEG && cur[c - 1] - 1 > best {
                best = cur[c - 1] - 1;
                how = 2;
            }
            cur[c] = best;
            from[i * w + c] = how;
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    // Back from (n, m).
    let mut out = Vec::with_capacity(n);
    let (mut i, mut j) = (n, m);
    let mut passed = 0usize;
    while i > 0 || j > 0 {
        let Some(c) = col(i, j) else { break };
        match if i == 0 { 2 } else { from[i * w + c] } {
            0 => {
                if want[i - 1].is_none_or(|k| k == count[j - 1]) {
                    out.push((entries[i - 1], heads[j - 1]));
                }
                i -= 1;
                j -= 1;
            }
            1 => i -= 1,
            _ => {
                passed += 1;
                j -= 1;
            }
        }
    }
    out.reverse();
    (out, passed)
}

/// Reads the placed models of the map zone (its `gfx_map` data from
/// about `gfx_from`).
pub fn read(zone: &ZoneImage, gfx_from: usize, report: &mut Vec<String>) -> Placed {
    let mut out = Placed::default();
    let z = &zone.bytes;
    let Some(base) = entry_base(zone, gfx_from) else {
        report.push("static models: no back-pointers".into());
        return out;
    };
    let Some((table, instances)) = find_tables(zone, base, gfx_from, report) else {
        return out;
    };
    // Entries by kind: the header pointer -1 entries of a type have headers.
    let kind = |i: usize| u64_at(z, zone.list_at + 16 * i + 8);
    let entries_of = |t: u32, p: u64| -> Vec<usize> {
        (0..zone.types.len()).filter(|&i| zone.types[i] == t && kind(i) == p).collect()
    };
    let heads = xmodel_headers(z);
    let models = entries_of(9, INSERT);
    report.push(format!(
        "static models: {} xmodel headers, {} xmodel entries with one ({} others)",
        heads.len(),
        models.len(),
        entries_of(9, FOLLOW).len()
    ));
    let surfs = surfs_headers(z);
    let surfs_entries: Vec<usize> = (0..zone.types.len()).filter(|&i| zone.types[i] == 8).collect();
    report.push(format!(
        "static models: {} xmodelsurfs headers, {} entries ({} with -1)",
        surfs.len(),
        surfs_entries.len(),
        entries_of(8, INSERT).len()
    ));
    if heads.len() == models.len() {
        let mut wanted: Vec<usize> = table.iter().flatten().copied().collect();
        wanted.sort_unstable();
        wanted.dedup();
        let pos: std::collections::HashMap<usize, usize> = models.iter().enumerate().map(|(k, &e)| (e, k)).collect();
        let mut bad = 0;
        let mut why: std::collections::BTreeMap<&str, (usize, String)> = Default::default();
        for e in wanted {
            let Some(&k) = pos.get(&e) else { continue };
            let (h, ref name) = heads[k];
            let limit = heads.get(k + 1).map_or(h + (1 << 20), |n| n.0);
            match model_def(zone, base, h, name, limit) {
                Ok(d) => {
                    out.models.insert(e, d);
                }
                Err(w) => {
                    bad += 1;
                    why.entry(w).or_insert((0, name.clone())).0 += 1;
                }
            }
        }
        report.push(format!("static models: {} models read, {bad} not {why:?}", out.models.len()));
    } else {
        report.push("static models: xmodel headers and entries differ in number; none read".into());
    }
    let mut expect: std::collections::HashMap<usize, usize> = Default::default();
    for d in out.models.values() {
        for l in &d.lods {
            expect.insert(l.surfs, l.surfaces);
        }
    }
    let (pairs, skipped) = pair_surfs(z, &entries_of(8, INSERT), &surfs, &expect);
    let fits = pairs
        .iter()
        .filter(|(e, h)| expect.get(e).is_some_and(|&n| crate::surfs::parse(z, *h).is_ok_and(|r| r.surfaces.len() == n)))
        .count();
    report.push(format!(
        "static models: {} meshes paired ({skipped} headers passed over), {fits} of {} a model's LOD names fit",
        pairs.len(),
        expect.len()
    ));
    out.surfs_at = pairs.into_iter().collect();
    out.table = table;
    out.instances = instances;
    out
}

/// A placed model's mesh surface, in the model's space.
#[derive(Clone, Debug, Default)]
pub struct PropSurface {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tris: Vec<[u16; 3]>,
    /// Index into the map's materials.
    pub material: Option<usize>,
}

/// A model the map places: its first LOD.
#[derive(Clone, Debug, Default)]
pub struct PropModel {
    pub name: String,
    pub surfaces: Vec<PropSurface>,
    /// Its coarsest LOD (empty when it has one LOD or that one is not read).
    pub coarse: Vec<PropSurface>,
    /// Its last LOD's distance (beyond it the model is not drawn).
    pub draw_distance: f32,
}

/// A placed model.
#[derive(Clone, Copy, Debug)]
pub struct Prop {
    /// Index into the map's prop models.
    pub model: usize,
    pub origin: [f32; 3],
    /// Rows: the model's forward, left and up axes in the world.
    pub axis: [[f32; 3]; 3],
    pub scale: f32,
}

/// The rotation a quaternion (x, y, z, w) makes, as the images of the
/// model's axes.
pub fn axes_of(q: [f32; 4]) -> [[f32; 3]; 3] {
    let n = q.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
    let [x, y, z, w] = q.map(|v| v / n);
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + w * z), 2.0 * (x * z - w * y)],
        [2.0 * (x * y - w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + w * x)],
        [2.0 * (x * z + w * y), 2.0 * (y * z - w * x), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

/// Model names that are no model to draw: shadow casters.
fn drawn(name: &str) -> bool {
    !name.contains("shadow::")
}

/// A LOD's surfaces decoded (its mesh's bytes from the zone or fetched).
fn lod_surfaces(
    z: &[u8],
    rec: &crate::surfs::SurfsRec,
    def: &ModelDef,
    lod: &Lod,
    fetch: &(dyn Fn(u64) -> Option<Vec<u8>> + Sync),
    material_of: &std::collections::HashMap<usize, usize>,
) -> Vec<PropSurface> {
    let blob = match rec.streamed {
        Some((key, len)) if key != 0 => match fetch(key) {
            Some(b) if b.len() >= len => b,
            _ => return Vec::new(),
        },
        Some(_) => return Vec::new(),
        None => z[rec.blob.0..rec.blob.0 + rec.blob.1].to_vec(),
    };
    let mut out = Vec::new();
    for i in 0..rec.surfaces.len() {
        let Ok(g) = crate::surfs::geometry_in(&blob, rec, i) else { continue };
        if g.positions.is_empty() || g.positions.len() > usize::from(u16::MAX) {
            continue;
        }
        let material = def
            .materials
            .get(lod.first + i)
            .copied()
            .flatten()
            .and_then(|e| material_of.get(&e).copied());
        out.push(PropSurface {
            uvs: g.uvs,
            tris: g.indices.chunks_exact(3).map(|t| [t[0] as u16, t[1] as u16, t[2] as u16]).collect(),
            positions: g.positions,
            material,
        });
    }
    out
}

/// The placed models' first LODs as meshes (a streamed mesh's bytes from
/// the packages by `fetch`, several at once) and the placements of the
/// ones that read. `material_of`: a material's asset-list entry -> its
/// index in the map's materials.
pub fn meshes(
    zone: &ZoneImage,
    placed: &Placed,
    fetch: &(dyn Fn(u64) -> Option<Vec<u8>> + Sync),
    material_of: &std::collections::HashMap<usize, usize>,
    report: &mut Vec<String>,
) -> (Vec<PropModel>, Vec<Prop>) {
    let z = &zone.bytes;
    // The static models placed, once each.
    let mut used: Vec<usize> = placed.instances.iter().map(|i| i.smodel).collect();
    used.sort_unstable();
    used.dedup();
    let jobs: Vec<(usize, &ModelDef, crate::surfs::SurfsRec)> = used
        .iter()
        .filter_map(|&sm| {
            let e = placed.table.get(sm).copied().flatten()?;
            let def = placed.models.get(&e).filter(|d| drawn(&d.name))?;
            let lod = def.lods.first()?;
            let h = *placed.surfs_at.get(&lod.surfs)?;
            let rec = crate::surfs::parse(z, h).ok()?;
            (rec.surfaces.len() == lod.surfaces).then_some((sm, def, rec))
        })
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<(usize, PropModel)>> = std::sync::Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().clamp(2, 8));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some((sm, def, rec)) = jobs.get(k) else { break };
                    let mut m = PropModel {
                        name: def.name.clone(),
                        draw_distance: def.lods.last().map_or(0.0, |l| l.distance),
                        surfaces: lod_surfaces(z, rec, def, &def.lods[0], fetch, material_of),
                        ..Default::default()
                    };
                    // A coarse LOD (what it collides as): the last but one
                    // (the last can be a far card).
                    if def.lods.len() > 1
                        && let Some(lod) = def.lods.get(def.lods.len().saturating_sub(2).max(1))
                        && let Some(&h) = placed.surfs_at.get(&lod.surfs)
                        && let Ok(coarse) = crate::surfs::parse(z, h)
                        && coarse.surfaces.len() == lod.surfaces
                    {
                        m.coarse = lod_surfaces(z, &coarse, def, lod, fetch, material_of);
                    }
                    if !m.surfaces.is_empty()
                        && let Ok(mut d) = done.lock()
                    {
                        d.push((*sm, m));
                    }
                }
            });
        }
    });
    let mut built = done.into_inner().unwrap_or_default();
    built.sort_by_key(|b| b.0);
    let index: std::collections::HashMap<usize, usize> = built.iter().enumerate().map(|(k, b)| (b.0, k)).collect();
    let models: Vec<PropModel> = built.into_iter().map(|b| b.1).collect();
    let props: Vec<Prop> = placed
        .instances
        .iter()
        .filter_map(|i| {
            Some(Prop {
                model: *index.get(&i.smodel)?,
                origin: i.origin,
                axis: axes_of(i.quat),
                scale: i.scale,
            })
        })
        .collect();
    report.push(format!(
        "static models: {} of {} models built ({} meshes to read), {} of {} placements",
        models.len(),
        used.len(),
        jobs.len(),
        props.len(),
        placed.instances.len()
    ));
    (models, props)
}
