use super::Shader_Handle;
use crate::loaders;
use crate::loaders::Resource_Loader;
use inle_common::stringid::String_Id;
use inle_gfx_backend::render::{Font, Shader, Texture};
use std::borrow::{Borrow, Cow};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

define_file_loader!(Texture, Texture_Loader, Texture_Cache);
define_file_loader!(Font, Font_Loader, Font_Cache);

// @Cleanup @WaitForStable: this code is mostly @Cutnpaste from loaders.rs, since
// the arguments to the Loader are different.
// Maybe when variadic generics are stable we can do better.
pub(super) struct Shader_Loader<T> {
    _pd: std::marker::PhantomData<T>,
}

impl<'l, T: Into<Cow<'l, str>>> loaders::Resource_Loader<'l, Shader<'l>> for Shader_Loader<T> {
    type Args = (T, T, Option<T>);

    fn load(&'l self, args: &(T, T, Option<T>)) -> Result<Shader<'l>, String> {
        let (vertex, fragment, geometry) = args;
        let vertex: &Cow<str> = vertex.into();
        let fragment: &Cow<str> = fragment.into();
        let geometry: Option<Cow<str>> = geometry.map(|g| g.into());
        Shader::from_file(
            Some(vertex.borrow()),
            geometry.map(|s| s.borrow()),
            Some(fragment.borrow()),
        )
        .ok_or_else(|| {
            format!(
                concat!("[ WARNING ] Failed to load Shader from {} / {} / {:?}"),
                vertex, fragment, geometry
            )
        })
    }
}

const ERROR_SHADER_KEY: String_Id = String_Id::from_u32(0);
const ERROR_SHADER_VERT: &str = "
    void main() {
        gl_Position = gl_ModelViewProjectionMatrix * gl_Vertex;
    }
";
const ERROR_SHADER_FRAG: &str = "
    void main() {
        gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
    }
";

fn load_error_shader<'a>() -> Shader<'a> {
    Shader::from_memory(Some(ERROR_SHADER_VERT), None, Some(ERROR_SHADER_FRAG)).unwrap()
}

pub(super) struct Shader_Cache<'l> {
    loader: &'l Shader_Loader<String>,
    cache: HashMap<String_Id, Shader<'l>>,
}

impl<'l> Shader_Cache<'l> {
    pub(super) fn new() -> Self {
        Self::new_with_loader(&Shader_Loader {
            _pd: std::marker::PhantomData,
        })
    }

    pub(super) fn new_with_loader(loader: &'l Shader_Loader<String>) -> Self {
        let mut cache = HashMap::new();
        cache.insert(ERROR_SHADER_KEY, load_error_shader());
        Shader_Cache { cache, loader }
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
                // @Incomplete: allow loading the geometry shader
                match self.loader.load(&(vs_name, fs_name, None)) {
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