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
        let img = icons
            .and_then(|i| i.0.iter().find(|(k, _)| k == name).map(|(_, i)| i.clone()))
            .or_else(|| crate::foreign_maps::map_picture(name))
            .or_else(|| crate::foreign_maps::prestige_picture(name));
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
