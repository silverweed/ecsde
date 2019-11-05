use crate::gfx::render::{Font, Texture};
use crate::resources::loaders;

//// Textures
pub(super) struct Texture_Loader;

impl<'l> loaders::Resource_Loader<'l, Texture<'l>> for Texture_Loader {
    type Args = str;

    fn load(&'l self, fname: &str) -> Result<Texture<'l>, String> {
        // @Incomplete: give a more descriptive error, if that's even possible.
        Texture::from_file(fname).ok_or_else(|| String::from("Texture load failed"))
    }
}

pub(super) type Texture_Cache<'l> = loaders::Cache<'l, Texture<'l>, Texture_Loader>;

impl Texture_Cache<'_> {
    pub fn new() -> Self {
        Self::new_with_loader(&Texture_Loader {})
    }
}

//// Fonts
pub(super) struct Font_Loader;

impl<'l> loaders::Resource_Loader<'l, Font<'l>> for Font_Loader {
    type Args = str;

    fn load(&'l self, fname: &str) -> Result<Font<'l>, String> {
        // @Incomplete: give a more descriptive error, if that's even possible.
        Font::from_file(fname).ok_or_else(|| String::from("Font load failed"))
    }
}

pub(super) type Font_Cache<'l> = loaders::Cache<'l, Font<'l>, Font_Loader>;

impl Font_Cache<'_> {
    pub fn new() -> Self {
        Self::new_with_loader(&Font_Loader {})
    }
}
