mod cache;

use super::asset_path;
use super::loaders;
use inle_core::env::Env_Info;
use inle_gfx_backend::render::{self, Font, Shader, Texture, Image};
use inle_common::colors;
use inle_common::stringid::{const_sid_from_str, String_Id};

pub type Texture_Handle = loaders::Res_Handle;
pub type Font_Handle = loaders::Res_Handle;
pub type Shader_Handle = loaders::Res_Handle;

pub struct Gfx_Resources<'l> {
    textures: cache::Texture_Cache<'l>,
    fonts: cache::Font_Cache<'l>,
}

impl<'l> Gfx_Resources<'l> {
    pub fn new() -> Self {
        if !render::shaders_are_available() {
            lwarn!("This platform does not support shaders.");
        } else if !render::geom_shaders_are_available() {
            lwarn!("This platform does not support geometry shaders.");
        }

		let mut tex_cache = cache::Texture_Cache::new();

		// This occurs once here.
		unsafe { 
			create_white_image(); 
			create_white_texture(&mut tex_cache); 
		}

        Gfx_Resources {
            textures: tex_cache,
            fonts: cache::Font_Cache::new(),
        }
    }

    pub fn load_texture(&mut self, fname: &str) -> Texture_Handle {
        self.textures.load(fname)
    }

    pub fn get_texture(&self, handle: Texture_Handle) -> &Texture<'_> {
        self.textures.must_get(handle)
    }

    pub fn get_texture_mut<'a>(&'a mut self, handle: Texture_Handle) -> &'a mut Texture<'l>
    where
        'l: 'a,
    {
        self.textures.must_get_mut(handle)
    }

	pub fn get_white_texture_handle(&self) -> Texture_Handle {
		Some(WHITE_TEXTURE_KEY)
	}

    pub fn load_font(&mut self, fname: &str) -> Font_Handle {
        self.fonts.load(fname)
    }

    pub fn get_font(&self, handle: Font_Handle) -> &Font<'_> {
        assert!(handle != None, "Invalid Font_Handle in get_font!");
        self.fonts.must_get(handle)
    }
}

pub struct Shader_Cache<'l>(cache::Shader_Cache<'l>);

impl<'l> Shader_Cache<'l> {
    pub fn new() -> Self {
		let mut shader_cache = cache::Shader_Cache::new();

		shader_cache.cache.insert(ERROR_SHADER_KEY, load_error_shader());
		shader_cache.cache.insert(BASIC_SHADER_KEY, load_basic_shader());

        Self(shader_cache)
    }

    pub fn load_shader(&mut self, fname: &str) -> Shader_Handle {
        if render::shaders_are_available() {
            self.0.load(fname, false)
        } else {
            None
        }
    }

    pub fn load_shader_with_geom(&mut self, fname: &str) -> Shader_Handle {
        if render::geom_shaders_are_available() {
            self.0.load(fname, true)
        } else {
            None
        }
    }

    pub fn get_shader(&self, handle: Shader_Handle) -> &Shader<'_> {
        debug_assert!(render::shaders_are_available());
        self.0.must_get(handle)
    }

    pub fn get_shader_mut<'a>(&'a mut self, handle: Shader_Handle) -> &'a mut Shader<'l>
    where
        'l: 'a,
    {
        debug_assert!(render::shaders_are_available());
        self.0.must_get_mut(handle)
    }

	pub fn get_error_shader_handle(&self) -> Shader_Handle {
		Some(ERROR_SHADER_KEY)
	}

	pub fn get_basic_shader_handle(&self) -> Shader_Handle {
		Some(BASIC_SHADER_KEY)
	}
}

pub fn tex_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "textures", file)
}

pub fn font_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "fonts", file)
}

pub fn shader_path(env: &Env_Info, file: &str) -> String {
    asset_path(env, "shaders", file)
}

const WHITE_TEXTURE_KEY: String_Id = const_sid_from_str("__white__");

/// This must be created with create_white_image()
static mut WHITE_IMAGE: Option<Image> = None;

/// # Safety
/// Must not be called from multiple threads
unsafe fn create_white_image() {
	let mut img = render::new_image(1, 1);
	render::set_image_pixel(&mut img, 0, 0, colors::WHITE);
	WHITE_IMAGE.replace(img);
}

/// # Safety
/// Must not be called from multiple threads
unsafe fn create_white_texture(tex_cache: &mut cache::Texture_Cache) {
	let img = WHITE_IMAGE.as_ref().expect("white image was not created yet!");
	let mut tex = render::new_texture_from_image(&img, None).unwrap();
	render::set_texture_repeated(&mut tex, true);
	tex_cache.cache.insert(WHITE_TEXTURE_KEY, tex);
}

const ERROR_SHADER_KEY: String_Id = cache::ERROR_SHADER_KEY;
const BASIC_SHADER_KEY: String_Id = const_sid_from_str("__basic__");

fn load_error_shader<'a>() -> Shader<'a> {
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
    Shader::from_memory(Some(ERROR_SHADER_VERT), None, Some(ERROR_SHADER_FRAG)).unwrap()
}

#[cfg(debug_assertions)]
fn load_basic_shader<'a>() -> Shader<'a> {
	const BASIC_SHADER_VERT: &str = "
		void main() {
			gl_TexCoord[0] = gl_TextureMatrix[0] * gl_MultiTexCoord0;
			gl_Position = gl_ModelViewProjectionMatrix * gl_Vertex;
		}
	";
	const BASIC_SHADER_FRAG: &str = "
		uniform sampler2D texture;
		void main() {
			gl_FragColor = texture2D(texture, gl_TexCoord[0].xy);
		}
	";
    Shader::from_memory(Some(BASIC_SHADER_VERT), None, Some(BASIC_SHADER_FRAG)).unwrap()
}
