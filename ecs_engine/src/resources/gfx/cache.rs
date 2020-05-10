use super::Shader_Handle;
use crate::common::stringid::String_Id;
use crate::gfx::render::{Font, Shader, Texture};
use crate::resources::loaders;
use crate::resources::loaders::Resource_Loader;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

define_file_loader!(Texture, Texture_Loader, Texture_Cache);
define_file_loader!(Font, Font_Loader, Font_Cache);

// @Cleanup @WaitForStable: this code is mostly @Cutnpaste from loaders.rs, since
// the arguments to the Loader are different.
// Maybe when variadic generics are stable we can do better.
pub(super) struct Shader_Loader;

impl<'l> loaders::Resource_Loader<'l, Shader<'l>> for Shader_Loader {
    type Args = (String, String);

    fn load(&'l self, args: &(String, String)) -> Result<Shader<'l>, String> {
        let (vertex, fragment) = args;
        Shader::from_file(Some(&vertex), None, Some(&fragment)).ok_or_else(|| {
            format!(
                concat!("[ WARNING ] Failed to load Shader from {} / {}"),
                vertex, fragment
            )
        })
    }
}

pub(super) struct Shader_Cache<'l> {
    loader: &'l Shader_Loader,
    cache: HashMap<String_Id, Shader<'l>>,
}

impl<'l> Shader_Cache<'l> {
    pub(super) fn new() -> Self {
        Self::new_with_loader(&Shader_Loader {})
    }

    pub(super) fn new_with_loader(loader: &'l Shader_Loader) -> Self {
        Shader_Cache {
            cache: HashMap::new(),
            loader,
        }
    }

    /// shader_name: the name of the shader(s) without extension.
    /// shader_name.vs and shader_name.fs will automatically be looked for.
    pub fn load(&mut self, shader_name: &str) -> Shader_Handle {
        let id = String_Id::from(shader_name);
        match self.cache.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                let vs_name = format!("{}.vert", shader_name);
                let fs_name = format!("{}.frag", shader_name);
                match self.loader.load(&(vs_name, fs_name)) {
                    Ok(res) => {
                        v.insert(res);
                        lok!("Loaded shader {}", shader_name);
                        Some(id)
                    }
                    Err(err) => {
                        lerr!("Error loading {}: {}", shader_name, err);
                        None
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
