//! bo2mp shell: pictures by name - the front end's own (BO2's menu
//! pictures), else another game's map or prestige picture (read off the
//! main thread; `None` until it is in).

use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct ShellPics {
    found: HashMap<String, Handle<Image>>,
    /// Names asked for and not (yet) found, with how many frames ago.
    tried: HashMap<String, u32>,
}

impl ShellPics {
    /// The first of `names` there is a picture for.
    pub fn first(
        &mut self,
        names: &[String],
        icons: Option<&assets::T6HudIcons>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        names.iter().find_map(|n| self.get(n, icons, images))
    }

    pub fn get(&mut self, name: &str, icons: Option<&assets::T6HudIcons>, images: &mut Assets<Image>) -> Option<Handle<Image>> {
        if name.is_empty() {
            return None;
        }
        if let Some(h) = self.found.get(name) {
            return Some(h.clone());
        }
        // (An async read is asked again until it is in; one never found is
        // given up after a while.)
        if let Some(n) = self.tried.get_mut(name) {
            *n += 1;
            if *n >= 600 {
                return None;
            }
        }
        // (The controller's button pictures are built into the game.)
        if name.starts_with("xenon")
            && let Some(img) = crate::pad_glyphs::icon_image(name)
        {
            let h = images.add(img);
            self.found.insert(name.to_owned(), h.clone());
            return Some(h);
        }
        // (The Zombies screens' pictures: Black Ops II's Zombies zones, the
        // Zombies rebuild's screenshots. While the zones are first read the
        // wait does not count towards giving up.)
        if super::zombies::is_zm_picture(name) {
            let img = super::zombies::picture(name);
            if img.is_none() && super::zombies::pictures_busy() {
                self.tried.insert(name.to_owned(), 1);
                return None;
            }
            return self.keep(name, img, images);
        }
        let own = icons.and_then(|i| i.0.iter().find(|(k, _)| k == name).map(|(_, i)| i.clone()));
        // (A small map shot of BO2's own is made bigger off the main thread
        // too; the original stands in should that fail.)
        let own = match own {
            Some(img) => match upscale_job(&img) {
                Some((srgb, raw)) => match crate::foreign_maps::picture_async(&format!("up:{name}"), srgb, move || {
                    Some(crate::foreign_maps::upscale_map_shot(raw))
                }) {
                    Some(big) => Some(big),
                    None => {
                        self.tried.entry(name.to_owned()).or_insert(1);
                        return None;
                    }
                },
                None => Some(img),
            },
            None => None,
        };
        let img = own
            .or_else(|| crate::foreign_maps::map_picture(name))
            .or_else(|| crate::foreign_maps::prestige_picture(name));
        self.keep(name, img, images)
    }

    /// A picture read (kept, clamped at its edges), or not (yet).
    fn keep(&mut self, name: &str, img: Option<std::sync::Arc<Image>>, images: &mut Assets<Image>) -> Option<Handle<Image>> {
        match img {
            Some(img) => {
                let mut img = (*img).clone();
                img.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
                    address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
                    ..bevy::image::ImageSamplerDescriptor::linear()
                });
                let h = images.add(img);
                self.found.insert(name.to_owned(), h.clone());
                self.tried.remove(name);
                Some(h)
            }
            None => {
                self.tried.entry(name.to_owned()).or_insert(1);
                None
            }
        }
    }

    /// Pictures asked for that are not in yet (some may still be reading).
    pub fn pending(&self) -> bool {
        self.tried.values().any(|n| *n < 600)
    }

    /// How many pictures are in (a view that shows pictures redraws when
    /// this changes).
    pub fn count(&self) -> usize {
        self.found.len()
    }
}

/// A node size that covers `area` with a picture of `size`, keeping its
/// shape (the overflow is cut by the parent).
pub(crate) fn cover(size: Vec2, area: Vec2) -> Vec2 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return area;
    }
    let s = (area.x / size.x).max(area.y / size.y);
    size * s
}

impl ShellPics {
    /// A picture given by name (the preview's stand-ins).
    pub fn insert(&mut self, name: &str, h: Handle<Image>) {
        self.found.insert(name.to_owned(), h);
    }
}

/// A small map picture (the games' map-select and loading pictures are
/// often 256-512 px) to make bigger: its colour space and pixels; other
/// pictures (and compressed ones) are left as they are.
fn upscale_job(img: &Image) -> Option<(bool, (u32, u32, Vec<u8>))> {
    use bevy::render::render_resource::TextureFormat;
    let size = img.size();
    if size.x == 0 || size.x >= 1024 || size.y >= 1024 || size.x < 128 || (size.x as f32) < size.y as f32 * 1.3 {
        return None;
    }
    let srgb = match img.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb => true,
        TextureFormat::Rgba8Unorm => false,
        _ => return None,
    };
    Some((srgb, (size.x, size.y, img.data.as_ref()?.clone())))
}

impl ShellPics {
    /// Asks for a few of the map picker's pictures each frame, so they are
    /// in (read and made bigger off the main thread) before it opens.
    pub fn prefetch(&mut self, cursor: &mut usize, cat: &super::bridge::Catalog, icons: Option<&assets::T6HudIcons>, images: &mut Assets<Image>) {
        if cat.maps.is_empty() || *cursor >= cat.maps.len() * 4 {
            return;
        }
        for _ in 0..3 {
            let m = &cat.maps[*cursor % cat.maps.len()];
            self.first(&m.pictures, icons, images);
            *cursor += 1;
        }
    }
}
