//! bo2mp: Black Ops III's surface look for its maps' world surfaces and
//! placed models (`fastfile_t7::look`): each lit material's colour map
//! tinted (its `colorTint` by its tint mask), its metal's diffuse and its
//! ambient occlusion taken out, its normal map (with its detail normal
//! map) and its specular map (reflectance from its metal mask and
//! `specColorTint`, gloss from its gloss map over its `glossRange` less its
//! normal map's variance), in the form Black Ops II's lit draw reads. The
//! reflections the specular maps show come from the lighting side's probes
//! (`T6_PROBES_MATERIAL`); without them the renderer draws the colour maps
//! alone, as before.
//!
//! A self-contained path: `BO2MP_T7_MATERIALS=0` turns it off (colour maps
//! only, as before). Other games' maps never come here.

use std::collections::HashMap;
use std::sync::Arc;

use asset_core::{AssetNamespace, AssetRef, T6Blend, T6Draw};
use asset_material::{AuthoredImage, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP, TS_NORMAL_MAP, TS_SPECULAR_MAP};
use fastfile_t7::look::Cover;
use fastfile_t7::map::T7Map;
use fastfile_t7::texture::ImageLevels;

/// `R_HashString` of Black Ops II's normal and specular map slots.
const NORMAL_MAP_HASH: u32 = 0x59d3_0d0f;
const SPECULAR_MAP_HASH: u32 = 0x34ec_ccb3;

/// The largest specular map kept (gloss and reflectance vary slowly).
const SPECULAR_MAX: u32 = 256;

/// Whether Black Ops III maps get their own look (`BO2MP_T7_MATERIALS=0`:
/// colour maps only).
pub(super) fn on() -> bool {
    std::env::var("BO2MP_T7_MATERIALS").map_or(true, |v| v.trim() != "0")
}

/// A material's look, linked: (colour, normal, specular) catalog images.
pub(super) type Linked = [Option<usize>; 3];

/// What `Looks` did, for the report.
#[derive(Default)]
pub(super) struct Stats {
    pub materials: usize,
    pub normal: usize,
    pub specular: usize,
    /// Tinted, metal, gloss map, occlusion, detail normals.
    pub used: [usize; 5],
    pub failures: Vec<String>,
}

/// The looks of a map's materials, each made once.
pub(super) struct Looks {
    max: u32,
    done: HashMap<usize, Linked>,
    pub stats: Stats,
}

impl Looks {
    pub fn new(max: u32) -> Self {
        Self { max, done: HashMap::new(), stats: Stats::default() }
    }

    /// Whether a material drawn as `draw` gets a look, and how it covers.
    pub fn cover(draw: &T6Draw) -> Option<Cover> {
        if draw.unlit || draw.effect || draw.water {
            return None;
        }
        match draw.blend {
            T6Blend::Opaque if !draw.alpha_test => Some(Cover::Opaque),
            T6Blend::Opaque | T6Blend::Blend => Some(Cover::Alpha),
            _ => None,
        }
    }

    /// Material `index`'s look, linked into `catalog`.
    pub fn link(&mut self, map: &T7Map, catalog: &mut MaterialCatalog, index: usize, cover: Cover) -> Linked {
        if let Some(l) = self.done.get(&index) {
            return *l;
        }
        let m = &map.materials[index];
        let linked = match map.look(m, self.max, SPECULAR_MAX.min(self.max), cover) {
            Ok(look) => {
                self.stats.materials += 1;
                for (k, u) in look.used.iter().enumerate() {
                    self.stats.used[k] += usize::from(*u);
                }
                let key = format!("t7look:{}#{}", m.name, self.max);
                let mut link = |levels: Option<ImageLevels>, semantic: u8, what: &str| {
                    levels.and_then(|l| link_levels(catalog, &mut self.stats.failures, format!("{key}#{what}"), l, semantic))
                };
                let c = link(look.colour, TS_COLOR_MAP, "colour");
                let n = link(look.normal, TS_NORMAL_MAP, "normal");
                let s = link(look.specular, TS_SPECULAR_MAP, "specular");
                self.stats.normal += usize::from(n.is_some());
                self.stats.specular += usize::from(s.is_some());
                [c, n, s]
            }
            Err(e) => {
                if self.stats.failures.len() < 32 {
                    self.stats.failures.push(format!("{}: {e}", m.name));
                }
                [None; 3]
            }
        };
        self.done.insert(index, linked);
        linked
    }

    /// The normal and specular bindings of a look.
    pub fn bindings(l: &Linked) -> Vec<MaterialTextureBinding> {
        [(NORMAL_MAP_HASH, TS_NORMAL_MAP, l[1]), (SPECULAR_MAP_HASH, TS_SPECULAR_MAP, l[2])]
            .into_iter()
            .filter_map(|(name_hash, semantic, image)| {
                image.map(|image| MaterialTextureBinding { name_hash, name_start: 0, name_end: 0, sampler_state: 0x13, semantic, image: Some(image) })
            })
            .collect()
    }

    pub fn report(&self) -> String {
        let s = &self.stats;
        format!(
            "t7 look: {} materials with Black Ops III's look ({} normal maps, {} specular maps; {} tinted, {} with metal, {} with gloss maps, {} with occlusion, {} with detail normals), {} failed{}",
            s.materials,
            s.normal,
            s.specular,
            s.used[0],
            s.used[1],
            s.used[2],
            s.used[3],
            s.used[4],
            s.failures.len(),
            s.failures.first().map_or(String::new(), |f| format!(" (first: {f})"))
        )
    }
}

/// Block-compressed levels into the catalog.
fn link_levels(catalog: &mut MaterialCatalog, failures: &mut Vec<String>, key: String, l: ImageLevels, semantic: u8) -> Option<usize> {
    let px = fastfile_iw6::image::ImagePixels { width: l.width, height: l.height, levels: l.levels, dxgi_format: l.format, data: l.data };
    let srgb = semantic != TS_NORMAL_MAP;
    match super::iw6::gpu_image(&px, srgb) {
        Ok(gpu) => Some(catalog.link_image(AuthoredImage {
            namespace: AssetNamespace::T6,
            name: AssetRef::Real(key),
            map_type: 0,
            semantic,
            category: 0,
            use_srgb_reads: srgb,
            width: px.width as u16,
            height: px.height as u16,
            depth: 1,
            level_count: px.levels as u8,
            format: 0,
            payload: Arc::new(Vec::new()),
            decoded: Some(Arc::new(gpu)),
            common_owned: false,
            decoded_variant: None,
            decoded_by: None,
            pending_decode: None,
        })),
        Err(e) => {
            if failures.len() < 32 {
                failures.push(format!("{key}: {e}"));
            }
            None
        }
    }
}
