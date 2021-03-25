use super::{Primitive_Type, Uniform_Value};
use crate::backend_common::alloc::{Buffer_Allocator, Buffer_Allocator_Id, Buffer_Handle};
use crate::backend_common::misc::check_gl_err;
use crate::render_window::Render_Window_Handle;
use gl::types::*;
use inle_common::colors::{Color, Color3};
use inle_common::paint_props::Paint_Properties;
use inle_math::matrix::Matrix3;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::sync::Once;
use std::{mem, ptr, str};

fn max_texture_units() -> usize {
    static mut MAX_TEXTURE_UNITS: usize = 0;
    static INIT: Once = Once::new();

    unsafe {
        INIT.call_once(|| {
            gl::GetIntegerv(
                gl::MAX_TEXTURE_IMAGE_UNITS,
                &mut MAX_TEXTURE_UNITS as *mut usize as *mut i32,
            );
            check_gl_err();
        });

        MAX_TEXTURE_UNITS
    }
}

/// A Vertex_Buffer is an unresizable vertex buffer that accepts vertices in this format:
/// (location = 0) vec4 color;
/// (location = 1) vec2 pos;
/// (location = 2) vec2 tex_coords;
pub struct Vertex_Buffer {
    cur_vertices: u32,
    max_vertices: u32,
    primitive_type: Primitive_Type,
    buf: Buffer_Handle,
}

impl Vertex_Buffer {
    const LOC_IN_COLOR: GLuint = 0;
    const LOC_IN_POS: GLuint = 1;
    const LOC_IN_TEXCOORD: GLuint = 2;

    fn new(
        buffer_allocator: &mut Buffer_Allocator,
        primitive_type: Primitive_Type,
        max_vertices: u32,
    ) -> Self {
        let buf = buffer_allocator.allocate(max_vertices as usize * mem::size_of::<Vertex>());

        if max_vertices > 0 {
            unsafe {
                gl::VertexAttribPointer(
                    Self::LOC_IN_COLOR,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<Vertex>() as _,
                    // @Robustness: use offsetof or similar
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_COLOR);

                gl::VertexAttribPointer(
                    Self::LOC_IN_POS,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<Vertex>() as _,
                    // @Robustness: use offsetof or similar
                    mem::size_of::<Glsl_Vec4>() as *const c_void,
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_POS);

                gl::VertexAttribPointer(
                    Self::LOC_IN_TEXCOORD,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<Vertex>() as _,
                    // @Robustness: use offsetof or similar
                    (mem::size_of::<Glsl_Vec4>() + mem::size_of::<Vec2f>()) as *const c_void,
                );
                gl::EnableVertexAttribArray(Self::LOC_IN_TEXCOORD);
            }
        } else {
            lwarn!("Creating a Vertex_Buffer with max_vertices = 0");
        }

        Self {
            buf,
            cur_vertices: 0,
            max_vertices,
            primitive_type,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct Glsl_Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

const_assert!(mem::size_of::<Glsl_Vec4>() == mem::size_of::<GLfloat>() * 4);
const_assert!(mem::size_of::<Vec2f>() == mem::size_of::<GLfloat>() * 2);

impl From<Color> for Glsl_Vec4 {
    fn from(c: Color) -> Self {
        Self {
            x: c.r as f32 / 255.0,
            y: c.g as f32 / 255.0,
            z: c.b as f32 / 255.0,
            w: c.a as f32 / 255.0,
        }
    }
}

impl From<Glsl_Vec4> for Color {
    fn from(c: Glsl_Vec4) -> Self {
        Self {
            r: (c.x * 255.0) as u8,
            g: (c.y * 255.0) as u8,
            b: (c.z * 255.0) as u8,
            a: (c.w * 255.0) as u8,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct Glsl_Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

const_assert!(mem::size_of::<Glsl_Vec3>() == mem::size_of::<GLfloat>() * 3);

impl From<Color3> for Glsl_Vec3 {
    fn from(c: Color3) -> Self {
        Self {
            x: c.r as f32 / 255.0,
            y: c.g as f32 / 255.0,
            z: c.b as f32 / 255.0,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Vertex {
    color: Glsl_Vec4,  // 16 B
    position: Vec2f,   // 8 B
    tex_coords: Vec2f, // 8 B
}

impl Vertex {
    pub fn color(&self) -> Color {
        self.color.into()
    }

    pub fn set_color(&mut self, c: Color) {
        self.color = c.into();
    }

    pub fn position(&self) -> Vec2f {
        self.position
    }

    pub fn set_position(&mut self, v: Vec2f) {
        self.position = v;
    }

    pub fn tex_coords(&self) -> Vec2f {
        self.tex_coords
    }

    pub fn set_tex_coords(&mut self, tc: Vec2f) {
        self.tex_coords = tc;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Color_Type {
    Grayscale,
    RGB,
    Indexed,
    Grayscale_Alpha,
    RGBA,
}

pub struct Image {
    bytes: Vec<u8>,

    width: u32,
    height: u32,
    color_type: Color_Type,

    // NOTE: this is currently always expected to be 8 for all manipulation purposes.
    // It can only be != 8 while loading a texture from an image, since in that case it needs
    // not be manipulated by our API calls.
    bit_depth: u8,
}

impl Image {
    fn bit_depth_as_gl_type(&self) -> GLenum {
        match self.bit_depth {
            8 => gl::UNSIGNED_BYTE,
            16 => gl::UNSIGNED_SHORT,
            32 => gl::UNSIGNED_INT,
            _ => fatal!("Unsupported bit depth {}", self.bit_depth),
        }
    }

    fn color_type_as_gl_type(&self) -> GLenum {
        match (self.color_type, self.bit_depth) {
            (Color_Type::Grayscale, 8) => gl::R8,
            (Color_Type::Grayscale_Alpha, 8) => gl::RG8,
            (Color_Type::RGB, 8) => gl::RGB8,
            (Color_Type::RGB, 16) => gl::RGB16,
            (Color_Type::RGBA, 8) => gl::RGBA8,
            (Color_Type::RGBA, 16) => gl::RGBA16,
            _ => fatal!(
                "combination {:?} / {}bits is not implemented",
                self.color_type,
                self.bit_depth
            ),
        }
    }

    fn pixel_format_as_gl_type(&self) -> GLenum {
        match self.color_type_as_gl_type() {
            gl::RGB8 | gl::RGB16 => gl::RGB,
            gl::RGBA8 | gl::RGBA16 => gl::RGBA,
            gl::R8 => gl::RED,
            gl::RG8 => gl::RG,
            x => fatal!("color type {} is unsupported", x),
        }
    }
}

#[derive(Debug)]
pub struct Texture<'a> {
    id: GLuint,

    width: u32,
    height: u32,

    pixel_type: GLenum,

    _pd: PhantomData<&'a ()>,
}

pub struct Shader<'texture> {
    id: GLuint,

    // [uniform location => texture id]
    textures: HashMap<GLint, GLuint>,

    _pd: PhantomData<&'texture ()>,
}

impl Uniform_Value for f32 {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        unsafe {
            gl::UseProgram(shader.id); // @Speed: don't do this every time!
            gl::Uniform1f(get_uniform_loc(shader.id, name), self);
        }
    }
}

impl Uniform_Value for Vec2f {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        unsafe {
            gl::UseProgram(shader.id); // @Speed: don't do this every time!
            gl::Uniform2f(get_uniform_loc(shader.id, name), self.x, self.y);
        }
    }
}

impl Uniform_Value for &Matrix3<f32> {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        unsafe {
            gl::UseProgram(shader.id); // @Speed: don't do this every time!
            gl::UniformMatrix3fv(
                get_uniform_loc(shader.id, name),
                1,
                gl::FALSE,
                self.as_slice().as_ptr() as _,
            );
        }
    }
}

impl Uniform_Value for Color {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        let v: Glsl_Vec4 = self.into();
        unsafe {
            gl::UseProgram(shader.id); // @Speed: don't do this every time!
            gl::Uniform4f(get_uniform_loc(shader.id, name), v.x, v.y, v.z, v.w);
        }
    }
}

impl Uniform_Value for Color3 {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        let v: Glsl_Vec3 = self.into();
        unsafe {
            gl::UseProgram(shader.id); // @Speed: don't do this every time!
            gl::Uniform3f(get_uniform_loc(shader.id, name), v.x, v.y, v.z);
        }
    }
}

impl Uniform_Value for &Texture<'_> {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        use std::collections::hash_map::Entry;

        let loc = get_uniform_loc(shader.id, name);
        if loc == -1 {
            return;
        }

        let n_tex = shader.textures.len();
        match shader.textures.entry(loc) {
            Entry::Occupied(mut v) => {
                v.insert(self.id);
            }
            Entry::Vacant(v) => {
                if n_tex == max_texture_units() {
                    lerr!("Cannot set uniform {:?}: texture units are full.", name);
                } else {
                    v.insert(self.id);
                }
            }
        }
    }
}

pub struct Font<'a> {
    pub atlas: Texture<'a>,
    pub metadata: Font_Metadata,
}

pub struct Font_Metadata {
    pub glyph_data: [Glyph_Data; 256],
}

impl Default for Font_Metadata {
    fn default() -> Self {
        Self {
            glyph_data: [Glyph_Data::default(); 256],
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Glyph_Data {
    pub advance: f32,

    /// Bounding box relative to the baseline
    pub plane_bounds: Glyph_Bounds,

    /// Normalized coordinates (uv) inside atlas
    pub normalized_atlas_bounds: Glyph_Bounds,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Glyph_Bounds {
    pub left: f32,
    pub bot: f32,
    pub right: f32,
    pub top: f32,
}

impl Font_Metadata {
    fn get_glyph_data(&self, glyph: char) -> &Glyph_Data {
        &self.glyph_data[glyph as usize]
    }
}

pub struct Text<'font> {
    string: String,
    font: &'font Font<'font>,
    size: u16,
}

pub fn new_shader_internal(vert_src: &[u8], frag_src: &[u8], shader_name: &str) -> GLuint {
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        check_gl_err();

        let c_str_vert = CString::new(vert_src)
            .unwrap_or_else(|_| fatal!("Vertex source did not contain a valid string."));

        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        const INFO_LOG_CAP: GLint = 512;
        let mut info_log = Vec::with_capacity(INFO_LOG_CAP as usize);
        info_log.set_len(INFO_LOG_CAP as usize - 1); // subtract 1 to skip the trailing null character

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        let mut info_len = 0;
        if success != gl::TRUE.into() {
            gl::GetShaderInfoLog(
                vertex_shader,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            fatal!(
                "Vertex shader `{}` failed to compile:\n----------\n{}\n-----------",
                shader_name,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(frag_src).unwrap();

        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetShaderInfoLog(
                fragment_shader,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            fatal!(
                "Fragment shader `{}` failed to compile:\n----------\n{}\n-----------",
                shader_name,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetProgramInfoLog(
                shader_program,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            );
            fatal!(
                "Shader `{}` failed to link:\n----------\n{}\n-----------",
                shader_name,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        debug_assert!(shader_program != 0);
        ldebug!(
            "Shader `{}` ({}) linked successfully.",
            shader_name,
            shader_program
        );

        shader_program
    }
}

pub fn new_shader<'a>(vert_src: &[u8], frag_src: &[u8], shader_name: Option<&str>) -> Shader<'a> {
    Shader {
        id: new_shader_internal(vert_src, frag_src, shader_name.unwrap_or("(unnamed)")),
        textures: HashMap::default(),
        _pd: PhantomData,
    }
}

pub fn fill_color_rect<R>(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    rect: R,
) where
    R: Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    let rect = rect.into();

    // @FIXME: this outline also fills the shape, and it shouldn't!
    if paint_props.border_thick > 0. {
        let outline_rect = Rect::new(
            rect.x - paint_props.border_thick,
            rect.y - paint_props.border_thick,
            rect.width + 2. * paint_props.border_thick,
            rect.height + 2. * paint_props.border_thick,
        );
        use_rect_shader(window, paint_props.border_color, &outline_rect);
        unsafe {
            gl::BindVertexArray(window.gl.rect_vao);
            window
                .gl
                .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
        }
    }

    use_rect_shader(window, paint_props.color, &rect);

    unsafe {
        gl::BindVertexArray(window.gl.rect_vao);
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
}

pub fn fill_color_rect_ws<T>(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    rect: T,
    transform: &Transform2D,
    camera: &Transform2D,
) where
    T: std::convert::Into<Rect<f32>> + Copy + Clone + std::fmt::Debug,
{
    let rect = rect.into();

    // @FIXME: this outline also fills the shape, and it shouldn't!
    if paint_props.border_thick > 0. {
        let outline_rect = Rect::new(
            rect.x - paint_props.border_thick,
            rect.y - paint_props.border_thick,
            rect.width + 2. * paint_props.border_thick,
            rect.height + 2. * paint_props.border_thick,
        );
        use_rect_ws_shader(
            window,
            paint_props.border_color,
            &outline_rect,
            transform,
            camera,
        );
        unsafe {
            gl::BindVertexArray(window.gl.rect_vao);
            window
                .gl
                .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
        }
    }

    use_rect_ws_shader(window, paint_props.color, &rect, transform, camera);

    unsafe {
        gl::BindVertexArray(window.gl.rect_vao);
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
}

pub fn fill_color_circle(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
) {
    // @FIXME: this outline also fills the shape, and it shouldn't!
    if paint_props.border_thick > 0. {
        let outline = shapes::Circle {
            center: circle.center,
            radius: circle.radius + 2. * paint_props.border_thick,
        };

        use_circle_shader(window, paint_props.border_color, outline);

        unsafe {
            gl::BindVertexArray(window.gl.circle_vao);
            window
                .gl
                .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
        }
    }

    use_circle_shader(window, paint_props.color, circle);

    unsafe {
        gl::BindVertexArray(window.gl.circle_vao);
        window
            .gl
            .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
    }
}

pub fn fill_color_circle_ws(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    // @FIXME: this outline also fills the shape, and it shouldn't!
    if paint_props.border_thick > 0. {
        let outline = shapes::Circle {
            center: circle.center,
            radius: circle.radius + 2. * paint_props.border_thick,
        };

        use_circle_ws_shader(window, paint_props.border_color, outline, camera);

        unsafe {
            gl::BindVertexArray(window.gl.circle_vao);
            window
                .gl
                .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
        }
    }

    use_circle_ws_shader(window, paint_props.color, circle, camera);

    unsafe {
        gl::BindVertexArray(window.gl.circle_vao);
        window
            .gl
            .draw_arrays(gl::TRIANGLE_FAN, 0, window.gl.n_circle_vertices());
    }
}

pub fn render_text(
    window: &mut Render_Window_Handle,
    text: &mut Text,
    paint_props: &Paint_Properties,
    screen_pos: Vec2f,
) {
    use inle_common::colors;

    if text.string.is_empty() {
        return;
    }

    use_text_shader(window, paint_props, &Transform2D::from_pos(screen_pos));

    // @Temporary
    let atlas_rect = Rect::new(0., 0., 216., 216.);

    let mut vbuf = new_vbuf_temp(
        window,
        Primitive_Type::Triangles,
        (6 * text.string.len() as u32),
    );
    let mut pos_x = 0.;
    for chr in text.string.chars() {
        let glyph_data = text.font.metadata.get_glyph_data(chr);
        let bounds = &glyph_data.normalized_atlas_bounds;
        // @Temporary
        let rect = Rect::new(pos_x, 0., 50., 50.);
        pos_x += 50. * glyph_data.advance + 50.;
        let uv = [
            bounds.left,
            bounds.top,
            bounds.right,
            bounds.top,
            bounds.right,
            bounds.bot,
            bounds.left,
            bounds.bot,
        ];

        let v1 = new_vertex(
            v2!(rect.x, rect.y),
            colors::WHITE,
            v2!(bounds.left, bounds.top),
        );
        let v2 = new_vertex(
            v2!(rect.x + rect.width, rect.y),
            colors::WHITE,
            v2!(bounds.right, bounds.top),
        );
        let v3 = new_vertex(
            v2!(rect.x + rect.width, rect.y + rect.height),
            colors::WHITE,
            v2!(bounds.right, bounds.bot),
        );
        let v4 = new_vertex(
            v2!(rect.x, rect.y + rect.height),
            colors::WHITE,
            v2!(bounds.left, bounds.bot),
        );
        add_vertices(window, &mut vbuf, &[v1, v2, v3, v3, v4, v1]);
    }

    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, text.font.atlas.id);

        gl::BindVertexArray(vbuf.buf.vao());
        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
    }
}

pub fn render_text_ws(
    _window: &mut Render_Window_Handle,
    _text: &mut Text,
    _paint_props: &Paint_Properties,
    _transform: &Transform2D,
    _camera: &Transform2D,
) {
}

pub fn get_texture_size(texture: &Texture) -> (u32, u32) {
    (texture.width, texture.height)
}

pub fn get_image_size(image: &Image) -> (u32, u32) {
    (image.width, image.height)
}

pub fn get_text_size(_text: &Text) -> Vec2f {
    // @Incomplete
    v2!(30.0, 30.0)
}

pub fn new_image(width: u32, height: u32, color_type: Color_Type) -> Image {
    Image {
        width,
        height,
        bytes: vec![0; 8 * (width * height) as usize],
        color_type,
        bit_depth: 8,
    }
}

pub fn new_image_with_data(
    width: u32,
    height: u32,
    color_type: Color_Type,
    bit_depth: u8,
    bytes: Vec<u8>,
) -> Image {
    Image {
        width,
        height,
        color_type,
        bit_depth,
        bytes,
    }
}

#[inline(always)]
pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    vbuf.primitive_type
}

#[inline(always)]
pub fn new_vbuf(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    Vertex_Buffer::new(
        window
            .gl
            .buffer_allocators
            .get_buffer_mut(Buffer_Allocator_Id::Array_Permanent),
        primitive,
        n_vertices,
    )
}

#[inline(always)]
pub fn new_vbuf_temp(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    Vertex_Buffer::new(
        window
            .gl
            .buffer_allocators
            .get_buffer_mut(Buffer_Allocator_Id::Array_Temporary),
        primitive,
        n_vertices,
    )
}

#[inline(always)]
pub fn add_vertices(
    window: &mut Render_Window_Handle,
    vbuf: &mut Vertex_Buffer,
    vertices: &[Vertex],
) {
    debug_assert!(
        vbuf.cur_vertices as usize + vertices.len() <= vbuf.max_vertices as usize,
        "vbuf max vertices exceeded! ({})",
        vbuf.max_vertices
    );
    update_vbuf(window, vbuf, vertices, vbuf.cur_vertices);
}

#[inline(always)]
pub fn update_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &mut Vertex_Buffer,
    vertices: &[Vertex],
    offset: u32,
) {
    let alloc = window
        .gl
        .buffer_allocators
        .get_buffer_mut(vbuf.buf.allocator_id());
    alloc.update_buffer(
        &vbuf.buf,
        offset as usize * mem::size_of::<Vertex>(),
        vertices.len() * mem::size_of::<Vertex>(),
        vertices.as_ptr() as _,
    );

    let written_len = offset as usize + vertices.len();

    debug_assert!(
        written_len <= vbuf.max_vertices as usize,
        "Written past max vertices! {}/{}",
        written_len,
        vbuf.max_vertices
    );
    vbuf.cur_vertices = vbuf.cur_vertices.max(written_len as u32);
}

#[inline(always)]
pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.cur_vertices
}

#[inline(always)]
pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.max_vertices
}

#[inline(always)]
pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    vbuf.cur_vertices = cur_vertices;
}

pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex {
        position: pos,
        color: col.into(),
        tex_coords,
    }
}

pub fn render_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    use_vbuf_shader(window, transform);

    unsafe {
        gl::BindVertexArray(vbuf.buf.vao());
        check_gl_err();

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
        check_gl_err();
    }
}

pub fn render_vbuf_ws(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    use_vbuf_ws_shader(window, transform, camera, window.gl.vbuf_shader);

    unsafe {
        gl::BindVertexArray(vbuf.buf.vao());
        check_gl_err();

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
        check_gl_err();
    }
}

pub fn render_vbuf_ws_with_texture(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
    camera: &Transform2D,
    texture: &Texture,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    use_vbuf_ws_shader(window, transform, camera, window.gl.vbuf_texture_shader);

    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture.id);

        gl::BindVertexArray(vbuf.buf.vao());
        check_gl_err();

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
        check_gl_err();
    }
}

pub fn render_vbuf_with_shader(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    shader: &Shader,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    unsafe {
        gl::UseProgram(shader.id);

        for (i, (loc, tex)) in shader.textures.iter().enumerate() {
            gl::Uniform1i(*loc, i as i32);
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            gl::BindTexture(gl::TEXTURE_2D, *tex);
            check_gl_err();
        }
        check_gl_err();

        gl::BindVertexArray(vbuf.buf.vao());
        check_gl_err();

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            0,
            vbuf.cur_vertices as _,
        );
        check_gl_err();
    }
}

pub fn create_text<'a>(string: &str, font: &'a Font, size: u16) -> Text<'a> {
    Text {
        string: String::from(string),
        font,
        size,
    }
}

pub fn render_line(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    use_line_shader(window, start, end);

    unsafe {
        // We reuse the rect VAO since it has no vertices associated to it.
        gl::BindVertexArray(window.gl.rect_vao);

        window.gl.draw_arrays(gl::LINES, 0, 2);
    }
}

pub fn copy_texture_to_image(texture: &Texture) -> Image {
    // NOTE: currently we always get pixels as unsigned bytes.
    let bit_depth = 1u8;
    let mut pixels = vec![0; (texture.width * texture.height * bit_depth as u32) as usize];
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture.id);
        gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as _,
        );
    }

    Image {
        width: texture.width,
        height: texture.height,
        bytes: pixels,
        color_type: Color_Type::RGBA,
        bit_depth,
    }
}

pub fn new_texture_from_image<'img, 'tex>(
    image: &'img Image,
    _rect: Option<Rect<i32>>,
) -> Texture<'tex> {
    assert!(image.width < gl::MAX_TEXTURE_SIZE);
    assert!(image.height < gl::MAX_TEXTURE_SIZE);

    let mut id = 0;
    let pixel_type = image.bit_depth_as_gl_type();
    unsafe {
        gl::GenTextures(1, &mut id);
        check_gl_err();

        debug_assert!(id != 0);

        gl::BindTexture(gl::TEXTURE_2D, id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

        gl::TexStorage2D(
            gl::TEXTURE_2D,
            1,
            image.color_type_as_gl_type(),
            image.width as _,
            image.height as _,
        );
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            image.width as _,
            image.height as _,
            image.pixel_format_as_gl_type(),
            pixel_type,
            image.bytes.as_ptr() as _,
        );
        check_gl_err();
    }

    ldebug!(
        "Loaded texture ({}) with size {}x{}, color type {:?} and pixel type {}",
        id,
        image.width,
        image.height,
        image.color_type,
        pixel_type
    );

    Texture {
        id,
        width: image.width,
        height: image.height,
        pixel_type,
        _pd: PhantomData,
    }
}

pub fn get_image_pixel(image: &Image, x: u32, y: u32) -> Color {
    debug_assert_eq!(image.bit_depth, 8);

    let b = &image.bytes[..];
    let i = (image.width * y + x) as usize;
    Color {
        r: b[i],
        g: b[i + 1],
        b: b[i + 2],
        a: b[i + 3],
    }
}

pub fn set_image_pixel(image: &mut Image, x: u32, y: u32, val: Color) {
    debug_assert_eq!(image.bit_depth, 8);

    let i = (y * image.width + x) as usize;
    image.bytes[i] = val.r;
    match image.color_type {
        Color_Type::Grayscale => {}
        Color_Type::Grayscale_Alpha => {
            image.bytes[i + 1] = val.g;
        }
        Color_Type::RGB => {
            image.bytes[i + 1] = val.g;
            image.bytes[i + 2] = val.b;
        }
        Color_Type::RGBA => {
            image.bytes[i + 1] = val.g;
            image.bytes[i + 2] = val.b;
            image.bytes[i + 3] = val.a;
        }
        _ => unimplemented!(),
    }
}

pub fn get_image_pixels(image: &Image) -> &[Color] {
    const_assert!(mem::size_of::<Color>() == 4);
    debug_assert_eq!(image.bytes.len() % 4, 0);
    debug_assert_eq!(image.bit_depth, 8);
    unsafe { std::slice::from_raw_parts(image.bytes.as_ptr() as *const _, image.bytes.len() / 4) }
}

pub fn swap_vbuf(a: &mut Vertex_Buffer, b: &mut Vertex_Buffer) -> bool {
    mem::swap(a, b);
    true
}

pub fn update_texture_pixels(texture: &mut Texture, rect: &Rect<u32>, pixels: &[Color]) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture.id);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            rect.x as _,
            rect.y as _,
            rect.width as _,
            rect.height as _,
            gl::RGBA,
            texture.pixel_type,
            pixels.as_ptr() as _,
        );
        check_gl_err();
    }
}

pub fn shaders_are_available() -> bool {
    true
}

pub fn geom_shaders_are_available() -> bool {
    false
}

pub fn set_texture_repeated(texture: &mut Texture, repeated: bool) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture.id);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            if repeated {
                gl::REPEAT
            } else {
                gl::CLAMP_TO_EDGE
            } as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            if repeated {
                gl::REPEAT
            } else {
                gl::CLAMP_TO_EDGE
            } as _,
        );
    }
}

pub fn set_texture_smooth(texture: &mut Texture, smooth: bool) {
    // @TODO: make this work
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture.id);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            if smooth { gl::LINEAR } else { gl::NEAREST } as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            if smooth { gl::LINEAR } else { gl::NEAREST } as _,
        );
    }
}

// -----------------------------------------------------------------------

fn use_rect_shader(window: &mut Render_Window_Handle, color: Color, rect: &Rect<f32>) {
    // @Volatile: order must be consistent with render_window::backend::RECT_INDICES
    let rect_vertices = [
        rect.x,
        rect.y,
        rect.x + rect.width,
        rect.y,
        rect.x + rect.width,
        rect.y + rect.height,
        rect.x,
        rect.y + rect.height,
    ];
    let mvp = get_mvp_screen_matrix(window, &Transform2D::default());
    let shader = window.gl.rect_shader;

    // @TODO: consider using UBOs
    unsafe {
        gl::UseProgram(window.gl.rect_shader);
        check_gl_err();

        gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );
        check_gl_err();

        gl::Uniform2fv(
            get_uniform_loc(shader, c_str!("rect")),
            (rect_vertices.len() / 2) as _,
            rect_vertices.as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

fn use_rect_ws_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    rect: &Rect<f32>,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    // @Volatile: order must be consistent with render_window::backend::RECT_INDICES
    let rect_vertices = [
        rect.x,
        rect.y,
        rect.x + rect.width,
        rect.y,
        rect.x + rect.width,
        rect.y + rect.height,
        rect.x,
        rect.y + rect.height,
    ];
    let mvp = get_mvp_matrix(window, transform, camera);
    let shader = window.gl.rect_shader;

    // @TODO: consider using UBOs
    unsafe {
        gl::UseProgram(window.gl.rect_shader);
        check_gl_err();

        gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );
        check_gl_err();

        gl::Uniform2fv(
            get_uniform_loc(shader, c_str!("rect")),
            (rect_vertices.len() / 2) as _,
            rect_vertices.as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

fn use_vbuf_shader(window: &mut Render_Window_Handle, transform: &Transform2D) {
    let mvp = get_mvp_screen_matrix(window, transform);
    unsafe {
        gl::UseProgram(window.gl.vbuf_shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.vbuf_shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );
    }
}

fn use_vbuf_ws_shader(
    window: &mut Render_Window_Handle,
    transform: &Transform2D,
    camera: &Transform2D,
    shader: GLuint,
) {
    let mvp = get_mvp_matrix(window, transform, camera);
    unsafe {
        gl::UseProgram(shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );
        check_gl_err();
    }
}

fn use_line_shader(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    let (ww, wh) = inle_win::window::get_window_target_size(window);
    let ww = ww as f32 * 0.5;
    let wh = wh as f32 * 0.5;
    unsafe {
        gl::UseProgram(window.gl.line_shader);

        gl::Uniform2fv(
            get_uniform_loc(window.gl.line_shader, c_str!("pos")),
            2,
            [
                (start.position.x - ww) / ww,
                (wh - start.position.y) / wh,
                (end.position.x - ww) / ww,
                (wh - end.position.y) / wh,
            ]
            .as_ptr(),
        );
        check_gl_err();

        gl::Uniform4fv(
            get_uniform_loc(window.gl.line_shader, c_str!("color")),
            2,
            [
                start.color.x,
                start.color.y,
                start.color.z,
                start.color.w,
                end.color.x,
                end.color.y,
                end.color.z,
                end.color.w,
            ]
            .as_ptr(),
        );
        check_gl_err();
    }
}

fn use_circle_shader(window: &mut Render_Window_Handle, color: Color, circle: shapes::Circle) {
    let transform = Transform2D::from_pos_rot_scale(
        circle.center,
        inle_math::angle::Angle::default(),
        v2!(circle.radius, circle.radius) * 2.0,
    );

    let mvp = get_mvp_screen_matrix(window, &transform);
    let shader = window.gl.circle_shader;

    unsafe {
        gl::UseProgram(shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

fn use_circle_ws_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    let transform = Transform2D::from_pos_rot_scale(
        circle.center,
        inle_math::angle::Angle::default(),
        v2!(circle.radius, circle.radius) * 2.0,
    );

    let mvp = get_mvp_matrix(window, &transform, camera);

    unsafe {
        gl::UseProgram(window.gl.circle_shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.circle_shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(window.gl.circle_shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

fn use_text_shader(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    transform: &Transform2D,
) {
    let mvp = get_mvp_screen_matrix(window, &transform);
    let shader = window.gl.text_shader;

    unsafe {
        gl::UseProgram(shader);

        gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        );

        gl::Uniform4f(
            get_uniform_loc(shader, c_str!("color")),
            paint_props.color.r as f32 / 255.0,
            paint_props.color.g as f32 / 255.0,
            paint_props.color.b as f32 / 255.0,
            paint_props.color.a as f32 / 255.0,
        );
        check_gl_err();
    }
}

#[inline]
fn get_uniform_loc(shader: GLuint, name: &CStr) -> GLint {
    unsafe {
        let loc = gl::GetUniformLocation(shader, name.as_ptr());
        #[cfg(debug_assertions)]
        if loc == -1 {
            let key = format!("{}.{:?}", shader, name);
            lerr_once!(
                &key,
                "Failed to get location of uniform `{:?}` in shader {}",
                name,
                shader
            );
        }
        loc
    }
}

fn get_mvp_matrix(
    window: &Render_Window_Handle,
    transform: &Transform2D,
    camera: &Transform2D,
) -> Matrix3<f32> {
    let (width, height) = inle_win::window::get_window_target_size(window);
    let model = transform;
    let view = camera.inverse();
    let projection = Matrix3::new(
        2. / width as f32,
        0.,
        0.,
        0.,
        -2. / height as f32,
        0.,
        0.,
        0.,
        1.,
    );
    projection * view.get_matrix() * model.get_matrix()
}

fn get_mvp_screen_matrix(window: &Render_Window_Handle, transform: &Transform2D) -> Matrix3<f32> {
    let (width, height) = inle_win::window::get_window_target_size(window);
    let model = transform;
    let view_projection = Matrix3::new(
        2. / width as f32,
        0.,
        -1.,
        0.,
        -2. / height as f32,
        1.,
        0.,
        0.,
        1.,
    );
    view_projection * model.get_matrix()
}

fn to_gl_primitive_type(prim: Primitive_Type) -> GLenum {
    match prim {
        Primitive_Type::Points => gl::POINTS,
        Primitive_Type::Lines => gl::LINES,
        Primitive_Type::Line_Strip => gl::LINE_STRIP,
        Primitive_Type::Triangles => gl::TRIANGLES,
        Primitive_Type::Triangle_Strip => gl::TRIANGLE_STRIP,
        Primitive_Type::Triangle_Fan => gl::TRIANGLE_FAN,
    }
}
