use super::{Primitive_Type, Uniform_Value};
use crate::backend_common::alloc::{
    Buffer_Allocator_Id, Buffer_Allocator_Ptr, Buffer_Handle, EMPTY_BUFFER_HANDLE,
};
use crate::backend_common::misc::check_gl_err;
use crate::backend_common::types::*;
use crate::render::get_mvp_matrix;
use crate::render_window::Render_Window_Handle;
use gl::types::*;
use inle_common::colors::{Color, Color3};
use inle_common::paint_props::Paint_Properties;
use inle_math::matrix::Matrix3;
use inle_math::rect::Rect;
use inle_math::shapes;
use inle_math::transform::Transform2D;
use inle_math::vector::Vec2f;
use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::sync::Once;
use std::{mem, ptr, str};

macro_rules! glcheck {
    ($expr: expr) => {{
        let res = $expr;
        check_gl_err();
        res
    }};
}

pub type Vertex = crate::backend_common::types::Vertex;

fn max_texture_units() -> usize {
    static mut MAX_TEXTURE_UNITS: usize = 0;
    static INIT: Once = Once::new();

    unsafe {
        INIT.call_once(|| {
            glcheck!(gl::GetIntegerv(
                gl::MAX_TEXTURE_IMAGE_UNITS,
                &mut MAX_TEXTURE_UNITS as *mut usize as *mut i32,
            ));
        });

        MAX_TEXTURE_UNITS
    }
}

#[inline(always)]
fn assert_shader_in_use(shader: &Shader) {
    #[cfg(debug_assertions)]
    {
        let mut id: GLint = 0;
        unsafe {
            glcheck!(gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut id));
        }
        assert!(
            id as GLuint == shader.id,
            "Expected shader {} to be in use, but current one is {}!",
            shader.id,
            id
        );
    }
}

/// A Vertex_Buffer is an unresizable vertex buffer that accepts vertices in this format:
/// (location = 0) vec4 color;
/// (location = 1) vec2 pos;
/// (location = 2) vec2 tex_coords;
pub struct Vertex_Buffer {
    max_vertices: u32,
    primitive_type: Primitive_Type,
    buf: Buffer_Handle,
    parent_alloc: Option<Buffer_Allocator_Ptr>,
    vertices: Vec<Vertex>,
    needs_transfer_to_gpu: Cell<bool>,

    #[cfg(debug_assertions)]
    buf_alloc_id: Buffer_Allocator_Id,
}

impl Vertex_Buffer {
    fn empty() -> Self {
        Self {
            max_vertices: 0,
            primitive_type: Primitive_Type::Points,
            buf: EMPTY_BUFFER_HANDLE,
            parent_alloc: None,
            vertices: Vec::default(),
            needs_transfer_to_gpu: Cell::default(),
            #[cfg(debug_assertions)]
            buf_alloc_id: Buffer_Allocator_Id::Invalid,
        }
    }

    fn new(
        buf_allocator_ptr: &Buffer_Allocator_Ptr,
        primitive_type: Primitive_Type,
        max_vertices: u32,
        #[cfg(debug_assertions)] buf_alloc_id: Buffer_Allocator_Id,
    ) -> Self {
        let mut buffer_allocator = buf_allocator_ptr.borrow_mut();
        let buf = buffer_allocator.allocate(max_vertices as usize * mem::size_of::<Vertex>());

        if max_vertices == 0 {
            lwarn!("Creating a Vertex_Buffer with max_vertices = 0");
        }

        Self {
            buf,
            max_vertices,
            primitive_type,
            parent_alloc: Some(buf_allocator_ptr.clone()),
            vertices: Vec::with_capacity(max_vertices as usize),
            needs_transfer_to_gpu: Cell::new(false),
            #[cfg(debug_assertions)]
            buf_alloc_id,
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for Vertex_Buffer {
    fn drop(&mut self) {
        assert!(
            self.buf == EMPTY_BUFFER_HANDLE
                || self.buf_alloc_id == Buffer_Allocator_Id::Array_Temporary,
            "A manual lifetime vertex buffer was not deallocated before being dropped!"
        );
    }
}

#[track_caller]
#[inline(always)]
fn check_vbuf_valid(vbuf: &Vertex_Buffer) {
    debug_assert!(
        vbuf.buf != EMPTY_BUFFER_HANDLE,
        "Vertex Buffer was invalid!"
    );
}

#[derive(Copy, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
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
            assert_shader_in_use(shader);
            glcheck!(gl::Uniform1f(get_uniform_loc(shader.id, name), self));
        }
    }
}

impl Uniform_Value for Vec2f {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        unsafe {
            assert_shader_in_use(shader);
            glcheck!(gl::Uniform2f(
                get_uniform_loc(shader.id, name),
                self.x,
                self.y
            ));
        }
    }
}

impl Uniform_Value for &Matrix3<f32> {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        unsafe {
            assert_shader_in_use(shader);
            glcheck!(gl::UniformMatrix3fv(
                get_uniform_loc(shader.id, name),
                1,
                gl::FALSE,
                self.as_slice().as_ptr() as _,
            ));
        }
    }
}

impl Uniform_Value for Color {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        let v: Glsl_Vec4 = self.into();
        unsafe {
            assert_shader_in_use(shader);
            glcheck!(gl::Uniform4f(
                get_uniform_loc(shader.id, name),
                v.x,
                v.y,
                v.z,
                v.w
            ));
        }
    }
}

impl Uniform_Value for Color3 {
    fn apply_to(self, shader: &mut Shader, name: &CStr) {
        let v: Glsl_Vec3 = self.into();
        unsafe {
            assert_shader_in_use(shader);
            glcheck!(gl::Uniform3f(
                get_uniform_loc(shader.id, name),
                v.x,
                v.y,
                v.z
            ));
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

#[inline]
pub fn use_shader(shader: &mut Shader) {
    unsafe {
        glcheck!(gl::UseProgram(shader.id));
    }
}

pub struct Font<'a> {
    pub atlas: Texture<'a>,
    pub metadata: Font_Metadata,
}

pub struct Font_Metadata {
    // @Temporary: we want to support more than ASCII
    glyph_data: [Glyph_Data; 256],
    pub atlas_size: (u32, u32),
    pub max_glyph_height: f32,
}

impl Font_Metadata {
    pub fn with_atlas_size(width: u32, height: u32) -> Self {
        Self {
            atlas_size: (width, height),
            glyph_data: [Glyph_Data::default(); 256],
            max_glyph_height: 0.,
        }
    }

    pub fn add_glyph_data(&mut self, glyph_id: char, data: Glyph_Data) {
        if (glyph_id as usize) < 256 {
            self.glyph_data[glyph_id as usize] = data;
            if data.plane_bounds.height() > self.max_glyph_height {
                self.max_glyph_height = data.plane_bounds.height();
            }
        } else {
            lwarn!("We currently don't support non-ASCII characters: discarding glyph data for {} (0x{:X})"
                , glyph_id, glyph_id as usize);
        }
    }

    fn get_glyph_data(&self, glyph: char) -> Option<&Glyph_Data> {
        // @Temporary
        if (glyph as usize) < self.glyph_data.len() {
            Some(&self.glyph_data[glyph as usize])
        } else {
            None
        }
    }

    /// plane_bounds * scale_factor = size_of_glyph_in_pixel
    fn scale_factor(&self, font_size: f32) -> f32 {
        let base_line_height = self.max_glyph_height;
        debug_assert!(base_line_height > 0.);

        // NOTE: this scale factor is chosen so the maximum possible text height is equal to font_size px.
        // We may want to change this and use font_size as the "main corpus" size,
        // but for now it seems like a reasonable choice.
        font_size / base_line_height
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

impl Glyph_Bounds {
    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.top - self.bot
    }
}

pub struct Text {
    string: String,
    font_atlas_id: GLuint,
    vertices: Vertex_Buffer,
    font_size: u16,
    size: Vec2f,
}

impl Drop for Text {
    fn drop(&mut self) {
        dealloc_vbuf(&mut self.vertices);
    }
}

#[inline]
pub fn new_shader_internal(vert_src: &[u8], frag_src: &[u8], shader_name: &str) -> GLuint {
    unsafe {
        let vertex_shader = glcheck!(gl::CreateShader(gl::VERTEX_SHADER));

        let c_str_vert = CString::new(vert_src)
            .unwrap_or_else(|_| fatal!("Vertex source did not contain a valid string."));

        glcheck!(gl::ShaderSource(
            vertex_shader,
            1,
            &c_str_vert.as_ptr(),
            ptr::null()
        ));
        glcheck!(gl::CompileShader(vertex_shader));

        const INFO_LOG_CAP: GLint = 512;
        let mut info_log = Vec::with_capacity(INFO_LOG_CAP as usize);
        info_log.set_len(INFO_LOG_CAP as usize - 1); // subtract 1 to skip the trailing null character

        let mut success = gl::FALSE as GLint;
        glcheck!(gl::GetShaderiv(
            vertex_shader,
            gl::COMPILE_STATUS,
            &mut success
        ));
        let mut info_len = 0;
        if success != gl::TRUE.into() {
            glcheck!(gl::GetShaderInfoLog(
                vertex_shader,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            ));
            fatal!(
                "Vertex shader `{}` failed to compile:\n----------\n{}\n-----------",
                shader_name,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }

        let fragment_shader = glcheck!(gl::CreateShader(gl::FRAGMENT_SHADER));
        let c_str_frag = CString::new(frag_src).unwrap();

        glcheck!(gl::ShaderSource(
            fragment_shader,
            1,
            &c_str_frag.as_ptr(),
            ptr::null()
        ));
        glcheck!(gl::CompileShader(fragment_shader));

        glcheck!(gl::GetShaderiv(
            fragment_shader,
            gl::COMPILE_STATUS,
            &mut success
        ));
        if success != gl::TRUE.into() {
            glcheck!(gl::GetShaderInfoLog(
                fragment_shader,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            ));
            fatal!(
                "Fragment shader `{}` failed to compile:\n----------\n{}\n-----------",
                shader_name,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }

        let shader_program = glcheck!(gl::CreateProgram());
        glcheck!(gl::AttachShader(shader_program, vertex_shader));
        glcheck!(gl::AttachShader(shader_program, fragment_shader));
        glcheck!(gl::LinkProgram(shader_program));

        glcheck!(gl::GetProgramiv(
            shader_program,
            gl::LINK_STATUS,
            &mut success
        ));
        if success != gl::TRUE.into() {
            glcheck!(gl::GetProgramInfoLog(
                shader_program,
                INFO_LOG_CAP,
                &mut info_len,
                info_log.as_mut_ptr() as *mut GLchar,
            ));
            fatal!(
                "Shader `{}` failed to link:\n----------\n{}\n-----------",
                shader_name,
                str::from_utf8(&info_log[..info_len as usize]).unwrap()
            );
        }
        glcheck!(gl::DeleteShader(vertex_shader));
        glcheck!(gl::DeleteShader(fragment_shader));

        debug_assert!(shader_program != 0);
        ldebug!(
            "Shader `{}` ({}) linked successfully.",
            shader_name,
            shader_program
        );

        shader_program
    }
}

#[inline]
pub fn new_shader<'a>(vert_src: &[u8], frag_src: &[u8], shader_name: Option<&str>) -> Shader<'a> {
    Shader {
        id: new_shader_internal(vert_src, frag_src, shader_name.unwrap_or("(unnamed)")),
        textures: HashMap::default(),
        _pd: PhantomData,
    }
}

#[inline]
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
            glcheck!(gl::BindVertexArray(window.gl.rect_vao));
            window
                .gl
                .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
        }
    }

    use_rect_shader(window, paint_props.color, &rect);

    unsafe {
        glcheck!(gl::BindVertexArray(window.gl.rect_vao));
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
}

#[inline]
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
            glcheck!(gl::BindVertexArray(window.gl.rect_vao));
            window
                .gl
                .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
        }
    }

    use_rect_ws_shader(window, paint_props.color, &rect, transform, camera);

    unsafe {
        glcheck!(gl::BindVertexArray(window.gl.rect_vao));
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
}

#[inline]
fn fill_color_circle_internal(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    mvp: &Matrix3<f32>,
) {
    // @Incomplete
    //if paint_props.border_thick > 0. {
    //}

    let rect = Rect::new(
        circle.center.x - circle.radius,
        circle.center.y - circle.radius,
        2. * circle.radius,
        2. * circle.radius,
    );

    use_circle_shader(window, paint_props.color, &rect, mvp);

    unsafe {
        glcheck!(gl::BindVertexArray(window.gl.rect_vao));
        window
            .gl
            .draw_indexed(window.gl.n_rect_indices(), window.gl.rect_indices_type());
    }
}

#[inline]
pub fn fill_color_circle(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
) {
    let mvp = get_mvp_screen_matrix(window, &Transform2D::default());
    fill_color_circle_internal(window, paint_props, circle, &mvp);
}

#[inline]
pub fn fill_color_circle_ws(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    circle: shapes::Circle,
    camera: &Transform2D,
) {
    let mvp = get_mvp_matrix(window, &Transform2D::default(), camera);
    fill_color_circle_internal(window, paint_props, circle, &mvp);
}

#[inline]
pub fn render_text(
    window: &mut Render_Window_Handle,
    text: &Text,
    paint_props: &Paint_Properties,
    screen_pos: Vec2f,
) {
    if text.string.is_empty() {
        return;
    }

    let mvp = get_mvp_screen_matrix(window, &Transform2D::from_pos(screen_pos));
    use_text_shader(window, paint_props, &mvp);

    render_text_internal(window, text);
}

#[inline]
pub fn render_text_ws(
    window: &mut Render_Window_Handle,
    text: &Text,
    paint_props: &Paint_Properties,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    if text.string.is_empty() {
        return;
    }

    let mvp = get_mvp_matrix(window, transform, camera);
    use_text_shader(window, paint_props, &mvp);

    render_text_internal(window, text);
}

fn render_text_internal(window: &mut Render_Window_Handle, text: &Text) {
    unsafe {
        trace!("draw_text_vbuf");

        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, text.font_atlas_id));

        render_vbuf_internal(window, &text.vertices);
    }
}

fn fill_text_vbuf(text: &mut Text, font: &Font) {
    use inle_common::colors;

    trace!("fill_text_vbuf");

    let mut pos_x = 0.;
    let scale_factor = font.metadata.scale_factor(text.font_size as f32);
    for chr in text.string.chars() {
        if chr > '\u{256}' {
            lerr_once!(
                &format!("skip_{}", chr),
                "WE ARE NOT SUPPORTING NON-ASCII BUT WE SHOULD! Skipping character {} (0x{:X})",
                chr,
                (chr as usize)
            );
            continue;
        }
        if let Some(glyph_data) = font.metadata.get_glyph_data(chr) {
            let atlas_bounds = &glyph_data.normalized_atlas_bounds;
            let pb = &glyph_data.plane_bounds;
            let rect = Rect::new(
                pos_x + pb.left * scale_factor,
                // Offsetting the y so the text pivot is top-left rather than bottom-left
                (1.0 - pb.top) * scale_factor,
                (pb.right - pb.left) * scale_factor,
                (pb.top - pb.bot) * scale_factor,
            );

            pos_x += scale_factor * glyph_data.advance;

            let v1 = new_vertex(
                v2!(rect.x, rect.y),
                colors::WHITE,
                v2!(atlas_bounds.left, atlas_bounds.top),
            );
            let v2 = new_vertex(
                v2!(rect.x + rect.width, rect.y),
                colors::WHITE,
                v2!(atlas_bounds.right, atlas_bounds.top),
            );
            let v3 = new_vertex(
                v2!(rect.x + rect.width, rect.y + rect.height),
                colors::WHITE,
                v2!(atlas_bounds.right, atlas_bounds.bot),
            );
            let v4 = new_vertex(
                v2!(rect.x, rect.y + rect.height),
                colors::WHITE,
                v2!(atlas_bounds.left, atlas_bounds.bot),
            );

            text.size.x = text.size.x.max(rect.x + rect.width);
            text.size.y = text.size.y.max(rect.y + rect.height);

            add_vertices(&mut text.vertices, &[v1, v2, v3, v3, v4, v1]);
        }
    }
}

#[inline]
pub fn get_texture_size(texture: &Texture) -> (u32, u32) {
    (texture.width, texture.height)
}

#[inline]
pub fn get_image_size(image: &Image) -> (u32, u32) {
    (image.width, image.height)
}

#[inline]
pub fn get_text_string<'a>(text: &'a Text) -> &'a str {
    &text.string
}

#[inline]
pub fn get_text_size(text: &Text) -> Vec2f {
    text.size
}

#[inline]
pub fn new_image(width: u32, height: u32, color_type: Color_Type) -> Image {
    Image {
        width,
        height,
        bytes: vec![0; 8 * (width * height) as usize],
        color_type,
        bit_depth: 8,
    }
}

#[inline]
pub fn new_image_with_data(
    width: u32,
    height: u32,
    color_type: Color_Type,
    bit_depth: u8,
    bytes: Vec<u8>,
) -> Image {
    Image {
        bytes,
        width,
        height,
        color_type,
        bit_depth,
    }
}

#[inline(always)]
pub fn vbuf_primitive_type(vbuf: &Vertex_Buffer) -> Primitive_Type {
    check_vbuf_valid(vbuf);
    vbuf.primitive_type
}

#[inline(always)]
fn new_vbuf_internal(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
    buf_alloc_id: Buffer_Allocator_Id,
) -> Vertex_Buffer {
    if n_vertices > 0 {
        Vertex_Buffer::new(
            window.gl.buffer_allocators.get_alloc(buf_alloc_id),
            primitive,
            n_vertices,
            #[cfg(debug_assertions)]
            buf_alloc_id,
        )
    } else {
        Vertex_Buffer::empty()
    }
}

#[inline]
pub fn new_vbuf_temp(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    new_vbuf_internal(
        window,
        primitive,
        n_vertices,
        Buffer_Allocator_Id::Array_Temporary,
    )
}

#[inline]
pub fn new_vbuf(
    window: &mut Render_Window_Handle,
    primitive: Primitive_Type,
    n_vertices: u32,
) -> Vertex_Buffer {
    new_vbuf_internal(
        window,
        primitive,
        n_vertices,
        Buffer_Allocator_Id::Array_Permanent,
    )
}

#[inline]
pub fn dealloc_vbuf(vbuf: &mut Vertex_Buffer) {
    #[cfg(debug_assertions)]
    {
        assert!(vbuf.buf_alloc_id != Buffer_Allocator_Id::Array_Temporary);
    }

    if vbuf.buf == EMPTY_BUFFER_HANDLE {
        return;
    }

    let mut handle = EMPTY_BUFFER_HANDLE;
    std::mem::swap(&mut vbuf.buf, &mut handle);
    vbuf.parent_alloc
        .as_ref()
        .expect("Since this Vertex_Buffer is not empty it should have a parent allocator!")
        .borrow_mut()
        .deallocate(handle);
    vbuf.max_vertices = 0;
    vbuf.vertices.clear();
}

#[inline]
pub fn add_vertices(vbuf: &mut Vertex_Buffer, vertices: &[Vertex]) {
    trace!("add_vertices");
    debug_assert!(
        vbuf_cur_vertices(vbuf) as usize + vertices.len() <= vbuf.max_vertices as usize,
        "vbuf max vertices exceeded! ({})",
        vbuf.max_vertices
    );
    update_vbuf(vbuf, vertices, vbuf_cur_vertices(vbuf));
}

#[inline]
pub fn update_vbuf(vbuf: &mut Vertex_Buffer, vertices: &[Vertex], offset: u32) {
    trace!("update_vbuf");

    check_vbuf_valid(vbuf);

    #[cfg(debug_assertions)]
    let prev_vertices = vbuf.vertices.len();

    vbuf.vertices.truncate(offset as usize);

    let space_remaining = vbuf.max_vertices as usize - vbuf.vertices.len();
    let vertices_to_copy = vertices.len().min(space_remaining);

    #[cfg(debug_assertions)]
    {
        if vertices_to_copy != vertices.len() {
            lwarn!(
                "Trying to copy too many vertices ({}) into vbuf (remaining space: {}).",
                vertices.len(),
                space_remaining
            );
        }
    }
    vbuf.vertices.extend(&vertices[..vertices_to_copy]);
    vbuf.needs_transfer_to_gpu.set(true);

    #[cfg(debug_assertions)]
    {
        debug_assert_eq!(
            vbuf.vertices.len() as isize - prev_vertices as isize,
            vertices.len() as isize - (prev_vertices as isize - offset as isize)
        );
    }
}

#[inline]
fn vbuf_transfer_to_gpu(vbuf: &Vertex_Buffer) {
    trace!("vbuf_transfer_to_gpu");

    check_vbuf_valid(vbuf);

    let mut alloc = vbuf
        .parent_alloc
        .as_ref()
        .expect("Since this Vertex_Buffer is not empty it should have a parent allocator!")
        .borrow_mut();
    alloc.update_buffer(
        &vbuf.buf,
        0,
        vbuf_cur_vertices(vbuf) as usize * mem::size_of::<Vertex>(),
        vbuf.vertices.as_ptr() as _,
    );

    vbuf.needs_transfer_to_gpu.set(false);
}

#[inline(always)]
pub fn vbuf_cur_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.vertices.len() as _
}

#[inline(always)]
pub fn vbuf_max_vertices(vbuf: &Vertex_Buffer) -> u32 {
    vbuf.max_vertices
}

#[inline(always)]
pub fn set_vbuf_cur_vertices(vbuf: &mut Vertex_Buffer, cur_vertices: u32) {
    check_vbuf_valid(vbuf);
    vbuf.vertices
        .resize(cur_vertices as usize, Vertex::default());
    vbuf.needs_transfer_to_gpu.set(true);
}

#[inline]
pub fn new_vertex(pos: Vec2f, col: Color, tex_coords: Vec2f) -> Vertex {
    Vertex {
        position: pos,
        color: col.into(),
        tex_coords,
    }
}

fn render_vbuf_internal(window: &mut Render_Window_Handle, vbuf: &Vertex_Buffer) {
    check_vbuf_valid(vbuf);
    if vbuf.needs_transfer_to_gpu.get() {
        vbuf_transfer_to_gpu(vbuf);
    }
    unsafe {
        glcheck!(gl::BindVertexArray(vbuf.buf.vao()));

        window.gl.draw_arrays(
            to_gl_primitive_type(vbuf.primitive_type),
            (vbuf.buf.offset_bytes() / mem::size_of::<Vertex>()) as _,
            vbuf_cur_vertices(vbuf) as _,
        );
    }
}

#[inline]
pub fn render_vbuf(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    transform: &Transform2D,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    use_vbuf_shader(window, transform);
    render_vbuf_internal(window, vbuf);
}

#[inline]
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
    render_vbuf_internal(window, vbuf);
}

#[inline]
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
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
    }

    render_vbuf_internal(window, vbuf);
}

#[inline]
pub fn render_vbuf_with_shader(
    window: &mut Render_Window_Handle,
    vbuf: &Vertex_Buffer,
    shader: &Shader,
) {
    if vbuf_cur_vertices(vbuf) == 0 {
        return;
    }

    unsafe {
        glcheck!(gl::UseProgram(shader.id));

        for (i, (loc, tex)) in shader.textures.iter().enumerate() {
            glcheck!(gl::Uniform1i(*loc, i as i32));
            glcheck!(gl::ActiveTexture(gl::TEXTURE0 + i as u32));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, *tex));
        }
    }

    render_vbuf_internal(window, vbuf);
}

#[inline]
pub fn create_text(
    window: &mut Render_Window_Handle,
    string: &str,
    font: &Font,
    size: u16,
) -> Text {
    let vertices = new_vbuf(window, Primitive_Type::Triangles, 6 * string.len() as u32);
    let mut text = Text {
        string: String::from(string),
        vertices,
        font_atlas_id: font.atlas.id,
        font_size: size,
        size: v2!(0., 0.),
    };
    fill_text_vbuf(&mut text, font);
    text
}

#[inline]
pub fn render_line(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    use_line_shader(window, start, end);

    unsafe {
        // We reuse the rect VAO since it has no vertices associated to it.
        glcheck!(gl::BindVertexArray(window.gl.rect_vao));

        window.gl.draw_arrays(gl::LINES, 0, 2);
    }
}

#[inline]
pub fn copy_texture_to_image(texture: &Texture) -> Image {
    // NOTE: currently we always get pixels as unsigned bytes.
    let bit_depth = 1u8;
    let mut pixels = vec![0; (texture.width * texture.height * bit_depth as u32) as usize];
    unsafe {
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
        glcheck!(gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as _,
        ));
    }

    Image {
        width: texture.width,
        height: texture.height,
        bytes: pixels,
        color_type: Color_Type::RGBA,
        bit_depth,
    }
}

#[inline]
pub fn new_texture_from_image<'img, 'tex>(
    image: &'img Image,
    _rect: Option<Rect<i32>>,
) -> Texture<'tex> {
    assert!(image.width < gl::MAX_TEXTURE_SIZE);
    assert!(image.height < gl::MAX_TEXTURE_SIZE);

    let mut id = 0;
    let pixel_type = image.bit_depth_as_gl_type();
    unsafe {
        glcheck!(gl::GenTextures(1, &mut id));

        debug_assert!(id != 0);

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, id));

        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as _
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as _
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as _
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as _
        ));

        glcheck!(gl::TexStorage2D(
            gl::TEXTURE_2D,
            1,
            image.color_type_as_gl_type(),
            image.width as _,
            image.height as _,
        ));
        glcheck!(gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            image.width as _,
            image.height as _,
            image.pixel_format_as_gl_type(),
            pixel_type,
            image.bytes.as_ptr() as _,
        ));
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

#[inline]
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

#[inline]
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

#[inline]
pub fn get_image_pixels(image: &Image) -> &[Color] {
    const_assert!(mem::size_of::<Color>() == 4);
    debug_assert_eq!(image.bytes.len() % 4, 0);
    debug_assert_eq!(image.bit_depth, 8);
    unsafe { std::slice::from_raw_parts(image.bytes.as_ptr() as *const _, image.bytes.len() / 4) }
}

#[inline]
pub fn swap_vbuf(a: &mut Vertex_Buffer, b: &mut Vertex_Buffer) -> bool {
    check_vbuf_valid(a);
    check_vbuf_valid(b);

    mem::swap(a, b);
    true
}

#[inline]
pub fn update_texture_pixels(texture: &mut Texture, rect: &Rect<u32>, pixels: &[Color]) {
    unsafe {
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
        glcheck!(gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            rect.x as _,
            rect.y as _,
            rect.width as _,
            rect.height as _,
            gl::RGBA,
            texture.pixel_type,
            pixels.as_ptr() as _,
        ));
    }
}

#[inline]
pub fn shaders_are_available() -> bool {
    true
}

#[inline]
pub fn geom_shaders_are_available() -> bool {
    false
}

#[inline]
pub fn set_texture_repeated(texture: &mut Texture, repeated: bool) {
    unsafe {
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            if repeated {
                gl::REPEAT
            } else {
                gl::CLAMP_TO_EDGE
            } as _,
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            if repeated {
                gl::REPEAT
            } else {
                gl::CLAMP_TO_EDGE
            } as _,
        ));
    }
}

#[inline]
pub fn set_texture_smooth(texture: &mut Texture, smooth: bool) {
    // @TODO: make this work
    unsafe {
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            if smooth { gl::LINEAR } else { gl::NEAREST } as _,
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            if smooth { gl::LINEAR } else { gl::NEAREST } as _,
        ));
    }
}

pub struct Uniform_Buffer {
    pub id: GLuint,
    pub block_index: GLuint,
    pub mem: *mut u8,
    pub layout: std::alloc::Layout,
    pub needs_transfer_to_gpu: Cell<bool>,

    #[cfg(debug_assertions)]
    name: &'static CStr,
}

pub fn create_or_get_uniform_buffer<'window>(
    window: &'window mut Render_Window_Handle,
    shader: &Shader,
    name: &'static CStr,
) -> &'window mut Uniform_Buffer {
    trace!("create_or_get_uniform_buffer");

    window
        .gl
        .uniform_buffers
        .entry(name)
        .or_insert_with(|| unsafe {
            let block_index = glcheck!(gl::GetUniformBlockIndex(shader.id, name.as_ptr()));
            let mut block_size = 0;
            glcheck!(gl::GetActiveUniformBlockiv(
                shader.id,
                block_index,
                gl::UNIFORM_BLOCK_DATA_SIZE,
                &mut block_size
            ));
            assert!(block_size > 0);

            let layout = std::alloc::Layout::from_size_align(block_size as usize, 1)
                .unwrap_or_else(|e| fatal!("Failed to allocate memory for UBO {:?}: {}", name, e));
            let mem = std::alloc::alloc(layout);
            lverbose!("Allocated UBO {:?} with layout {:?}", name, layout);

            let mut id = 0;
            glcheck!(gl::GenBuffers(1, &mut id));

            Uniform_Buffer {
                id,
                block_index,
                mem,
                layout,
                needs_transfer_to_gpu: Cell::new(true),
                #[cfg(debug_assertions)]
                name,
            }
        })
}

#[inline]
/// # Safety
/// `data` must be valid: it must be non-null, aligned and it must
/// correspond to an allocation of at least `size` bytes.
pub unsafe fn write_into_uniform_buffer(
    ubo: &mut Uniform_Buffer,
    offset: usize,
    align: usize,
    size: usize,
    data: *const u8,
) -> usize {
    trace!("write_into_uniform_buffer");

    debug_assert!(!ubo.mem.is_null());
    debug_assert!(!data.is_null());

    ubo.needs_transfer_to_gpu.set(true);

    let write_p = ubo.mem.add(offset);
    let align_offset = write_p.align_offset(align);
    let write_p = write_p.add(align_offset);

    #[cfg(debug_assertions)]
    {
        debug_assert!(
            write_p.add(size) <= ubo.mem.add(ubo.layout.size()),
            "Writing past bounds of UBO {:?}! (from {:?} to {:?}/{:?})",
            ubo.name,
            write_p.offset_from(ubo.mem),
            write_p.add(size).offset_from(ubo.mem),
            ubo.layout.size()
        );

        lverbose!(
            "Writing into UBO {:?} from {:?} to {:?}/{:?} (len = {})",
            ubo.name,
            write_p.offset_from(ubo.mem),
            write_p.add(size).offset_from(ubo.mem),
            ubo.layout.size(),
            size,
        );
    }

    std::ptr::copy_nonoverlapping(data, write_p, size);

    write_p.add(size).offset_from(ubo.mem) as usize
}

pub fn bind_uniform_buffer(ubo: &Uniform_Buffer) {
    trace!("bind_uniform_buffer");

    unsafe {
        glcheck!(gl::BindBuffer(gl::UNIFORM_BUFFER, ubo.id));

        if ubo.needs_transfer_to_gpu.get() {
            debug_assert!(ubo.layout.size() < std::isize::MAX as usize);

            // @Speed: we may optimize this to only transfer the subrange that changed
            glcheck!(gl::BufferData(
                gl::UNIFORM_BUFFER,
                ubo.layout.size() as isize,
                ubo.mem as *const _,
                gl::DYNAMIC_DRAW
            ));

            #[cfg(debug_assertions)]
            {
                lverbose!("Transfered UBO {:?} to the GPU", ubo.name);
            }

            ubo.needs_transfer_to_gpu.set(false);
        }

        glcheck!(gl::BindBufferBase(
            gl::UNIFORM_BUFFER,
            ubo.block_index,
            ubo.id
        ));
    }
}

#[inline]
pub fn uniform_buffer_needs_transfer_to_gpu(ubo: &Uniform_Buffer) -> bool {
    ubo.needs_transfer_to_gpu.get()
}

// -----------------------------------------------------------------------

fn use_rect_shader_internal(color: Color, rect: &Rect<f32>, mvp: &Matrix3<f32>, shader: GLuint) {
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

    // @TODO: consider using UBOs
    unsafe {
        glcheck!(gl::UseProgram(shader));

        glcheck!(gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        ));

        glcheck!(gl::Uniform2fv(
            get_uniform_loc(shader, c_str!("rect")),
            (rect_vertices.len() / 2) as _,
            rect_vertices.as_ptr(),
        ));

        glcheck!(gl::Uniform4f(
            get_uniform_loc(shader, c_str!("color")),
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ));
    }
}

fn use_rect_shader(window: &mut Render_Window_Handle, color: Color, rect: &Rect<f32>) {
    let mvp = get_mvp_screen_matrix(window, &Transform2D::default());
    use_rect_shader_internal(color, rect, &mvp, window.gl.rect_shader);
}

fn use_rect_ws_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    rect: &Rect<f32>,
    transform: &Transform2D,
    camera: &Transform2D,
) {
    let mvp = get_mvp_matrix(window, transform, camera);
    use_rect_shader_internal(color, rect, &mvp, window.gl.rect_shader);
}

fn use_circle_shader(
    window: &mut Render_Window_Handle,
    color: Color,
    rect: &Rect<f32>,
    mvp: &Matrix3<f32>,
) {
    let shader = window.gl.circle_shader;
    use_rect_shader_internal(color, rect, mvp, shader);

    unsafe {
        glcheck!(gl::Uniform2f(
            get_uniform_loc(shader, c_str!("center")),
            rect.x + rect.width * 0.5,
            rect.y + rect.height * 0.5,
        ));
        glcheck!(gl::Uniform1f(
            get_uniform_loc(shader, c_str!("radius_squared")),
            rect.width * rect.width * 0.25,
        ));
    }
}

fn use_vbuf_shader(window: &mut Render_Window_Handle, transform: &Transform2D) {
    let mvp = get_mvp_screen_matrix(window, transform);
    unsafe {
        glcheck!(gl::UseProgram(window.gl.vbuf_shader));

        glcheck!(gl::UniformMatrix3fv(
            get_uniform_loc(window.gl.vbuf_shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        ));
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
        glcheck!(gl::UseProgram(shader));

        glcheck!(gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        ));
    }
}

fn use_line_shader(window: &mut Render_Window_Handle, start: &Vertex, end: &Vertex) {
    let (ww, wh) = inle_win::window::get_window_target_size(window);
    let ww = ww as f32 * 0.5;
    let wh = wh as f32 * 0.5;
    unsafe {
        glcheck!(gl::UseProgram(window.gl.line_shader));

        glcheck!(gl::Uniform2fv(
            get_uniform_loc(window.gl.line_shader, c_str!("pos")),
            2,
            [
                (start.position.x - ww) / ww,
                (wh - start.position.y) / wh,
                (end.position.x - ww) / ww,
                (wh - end.position.y) / wh,
            ]
            .as_ptr(),
        ));

        glcheck!(gl::Uniform4fv(
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
        ));
    }
}

fn use_text_shader(
    window: &mut Render_Window_Handle,
    paint_props: &Paint_Properties,
    mvp: &Matrix3<f32>,
) {
    let shader = window.gl.text_shader;

    unsafe {
        glcheck!(gl::UseProgram(shader));

        glcheck!(gl::UniformMatrix3fv(
            get_uniform_loc(shader, c_str!("mvp")),
            1,
            gl::FALSE,
            mvp.as_slice().as_ptr(),
        ));

        glcheck!(gl::Uniform4f(
            get_uniform_loc(shader, c_str!("color")),
            paint_props.color.r as f32 / 255.0,
            paint_props.color.g as f32 / 255.0,
            paint_props.color.b as f32 / 255.0,
            paint_props.color.a as f32 / 255.0,
        ));
    }
}

#[inline]
fn get_uniform_loc(shader: GLuint, name: &CStr) -> GLint {
    unsafe {
        let loc = glcheck!(gl::GetUniformLocation(shader, name.as_ptr()));
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

/// This is the equivalent of get_mvp_matrix() with a camera with scale 1, no rotation
/// and positioned in (win_target_size.x / 2, win_target_size.y / 2).
fn get_mvp_screen_matrix(window: &Render_Window_Handle, transform: &Transform2D) -> Matrix3<f32> {
    let (width, height) = inle_win::window::get_window_target_size(window);
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
    view_projection * transform.get_matrix()
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
