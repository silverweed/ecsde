mod cache;
mod font;
mod image;
mod shader;

use super::asset_path;
use super::loaders;
use inle_common::colors;
use inle_common::stringid::{const_sid_from_str, String_Id};
use inle_core::env::Env_Info;
use inle_gfx_backend::render::{self, Font, Image, Shader, Texture};
use std::path::Path;

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

        let tex_cache = cache::Texture_Cache::new();

        Gfx_Resources {
            textures: tex_cache,
            fonts: cache::Font_Cache::new(),
        }
    }

    pub fn init(&mut self) {
        // This occurs once here. Cannot do this in new() because it must be done after the Render_Window is created.
        unsafe {
            create_white_image();
            create_white_texture(&mut self.textures);
        }
    }

    pub fn load_texture(&mut self, fname: &Path) -> Texture_Handle {
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

    pub fn load_font(&mut self, fname: &Path) -> Font_Handle {
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
        Self(cache::Shader_Cache::new())
    }

    pub fn init(&mut self) {
        self.0.cache.insert(ERROR_SHADER_KEY, load_error_shader());

        #[cfg(debug_assertions)]
        self.0
            .cache
            .insert(BASIC_BATCHER_SHADER_KEY, load_basic_batcher_shader());
    }

    pub fn load_shader(&mut self, shader_name: &str) -> Shader_Handle {
        if render::shaders_are_available() {
            self.0.load(shader_name, false)
        } else {
            None
        }
    }

    pub fn load_shader_with_geom(&mut self, shader_name: &str) -> Shader_Handle {
        if render::geom_shaders_are_available() {
            self.0.load(shader_name, true)
        } else {
            lerr!(
                "Cannot load shader {}: geometry shaders are unavailable.",
                shader_name
            );
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

    #[cfg(debug_assertions)]
    pub fn get_basic_batcher_shader_handle(&self) -> Shader_Handle {
        Some(BASIC_BATCHER_SHADER_KEY)
    }
}

pub fn tex_path(env: &Env_Info, file: &str) -> Box<Path> {
    asset_path(env, "textures", file)
}

pub fn font_path(env: &Env_Info, file: &str) -> Box<Path> {
    asset_path(env, "fonts", file)
}

// NOTE: we return this by String because it's more convenient to use due to
// the shader cache API. We may want to change this in the future.
pub fn shader_path(env: &Env_Info, file: &str) -> String {
    String::from(asset_path(env, "shaders", file).to_str().unwrap())
}

const WHITE_TEXTURE_KEY: String_Id = const_sid_from_str("__white__");

/// This must be created with create_white_image()
static mut WHITE_IMAGE: Option<Image> = None;

/// # Safety
/// Must not be called from multiple threads
unsafe fn create_white_image() {
    let mut img = render::new_image(1, 1, render::Color_Type::RGB);
    render::set_image_pixel(&mut img, 0, 0, colors::WHITE);
    WHITE_IMAGE.replace(img);
}

/// # Safety
/// Must not be called from multiple threads
unsafe fn create_white_texture(tex_cache: &mut cache::Texture_Cache) {
    let img = WHITE_IMAGE
        .as_ref()
        .expect("white image was not created yet!");
    let mut tex = render::new_texture_from_image(&img, None);
    render::set_texture_repeated(&mut tex, true);
    tex_cache.cache.insert(WHITE_TEXTURE_KEY, tex);
}

const ERROR_SHADER_KEY: String_Id = cache::ERROR_SHADER_KEY;

#[cfg(debug_assertions)]
const BASIC_BATCHER_SHADER_KEY: String_Id = const_sid_from_str("__basic_batcher__");

fn load_error_shader<'a>() -> Shader<'a> {
    const ERROR_SHADER_VERT: &str = "
		#version 330 core

		layout (location = 1) in vec2 in_pos;

		uniform mat3 vp;

		void main() {
			vec3 pos = vp * vec3(in_pos, 1.0);
			gl_Position = vec4(pos.xy, 0.0, 1.0);
		}
	";
    const ERROR_SHADER_FRAG: &str = "
		#version 330 core

		out vec4 frag_color;

		void main() {
			frag_color = vec4(1.0, 0.0, 1.0, 1.0);
		}
	";

    render::new_shader(
        ERROR_SHADER_VERT.as_bytes(),
        ERROR_SHADER_FRAG.as_bytes(),
        Some("builtin_error_shader"),
    )
}

#[cfg(debug_assertions)]
fn load_basic_batcher_shader<'a>() -> Shader<'a> {
    const BASIC_BATCHER_SHADER_VERT: &str = "
		#version 330 core

		layout (location = 1) in vec2 in_pos;
		layout (location = 2) in vec2 in_tex_coord;

		uniform mat3 vp;

		out vec2 tex_coord;

		void main() {
			vec3 pos = vp * vec3(in_pos, 1.0);
			gl_Position = vec4(pos.xy, 0.0, 1.0);
			tex_coord = in_tex_coord;
		}
	";
    const BASIC_BATCHER_SHADER_FRAG: &str = "
		#version 330 core

		in vec2 tex_coord;

		out vec4 frag_color;

		uniform sampler2D tex;

		void main() {
			vec4 pixel = texture(tex, tex_coord);
			frag_color = pixel;
		}
   	";
    render::new_shader(
        BASIC_BATCHER_SHADER_VERT.as_bytes(),
        BASIC_BATCHER_SHADER_FRAG.as_bytes(),
        Some("builtin_basic_batcher_shader"),
    )
}
