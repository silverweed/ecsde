use super::Shader_Handle;
use crate::loaders;
use crate::loaders::Resource_Loader;
use inle_common::stringid::{const_sid_from_str, String_Id};
use inle_gfx_backend::render::{Font, Shader, Texture};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

define_file_loader!(
    Texture,
    Texture_Loader,
    Texture_Cache,
    super::image::load_texture_from_file
);
define_file_loader!(
    Font,
    Font_Loader,
    Font_Cache,
    super::font::load_font_from_file
);

// @Cleanup @WaitForStable: this code is mostly @Cutnpaste from loaders.rs, since
// the arguments to the Loader are different.
// Maybe when variadic generics are stable we can do better.
pub(super) struct Shader_Loader;

impl<'l> loaders::Resource_Loader<'l, Shader<'l>> for Shader_Loader {
    type Args = (String, String, Option<String>);

    fn load(&'l self, args: &Self::Args) -> Result<Shader<'l>, String> {
        let (vertex, fragment, geometry) = args;
        Shader::from_file(
            Some(&vertex),
            geometry.as_ref().map(|s| s.as_str()),
            Some(&fragment),
        )
        .ok_or_else(|| {
            format!(
                concat!("[ WARNING ] Failed to load Shader from {} / {} / {:?}"),
                vertex, fragment, geometry
            )
        })
    }
}

pub(super) const ERROR_SHADER_KEY: String_Id = const_sid_from_str("__error__");

pub(super) struct Shader_Cache<'l> {
    loader: &'l Shader_Loader,
    pub(super) cache: HashMap<String_Id, Shader<'l>>,
}

impl<'l> Shader_Cache<'l> {
    pub(super) fn new() -> Self {
        Self::new_with_loader(&Shader_Loader {})
    }

    pub(super) fn new_with_loader(loader: &'l Shader_Loader) -> Self {
        Shader_Cache {
            loader,
            cache: HashMap::new(),
        }
    }

    /// shader_name: the name of the shader(s) without extension.
    pub fn load(&mut self, shader_name: &str, with_geom: bool) -> Shader_Handle {
        let id = String_Id::from(shader_name);
        match self.cache.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                let vs_name = format!("{}.vert", shader_name);
                let fs_name = format!("{}.frag", shader_name);
                let gs_name = if with_geom {
                    Some(format!("{}.geom", shader_name))
                } else {
                    None
                };
                match self.loader.load(&(vs_name, fs_name, gs_name)) {
                    Ok(res) => {
                        v.insert(res);
                        lok!("Loaded shader {}", shader_name);
                        Some(id)
                    }
                    Err(err) => {
                        lerr!("Error loading {}: {}", shader_name, err);
                        Some(ERROR_SHADER_KEY)
                    }
                }
            }
        }
    }

    pub fn must_get(&self, handle: Shader_Handle) -> &Shader {
        &self.cache[&handle.unwrap()]
    }

    pub fn must_get_mut<'a>(&'a mut self, handle: Shader_Handle) -> &'a mut Shader<'l> {
        self.cache.get_mut(&handle.unwrap()).unwrap()
    }

    pub fn n_loaded(&self) -> usize {
        self.cache.len()
    }
}
