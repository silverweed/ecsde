use gl::types::*;
use inle_common::colors::{Color, Color3};
use inle_math::vector::Vec2f;
use std::mem;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Glsl_Vec4 {
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
pub struct Glsl_Vec3 {
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
    pub color: Glsl_Vec4,  // 16 B
    pub position: Vec2f,   // 8 B
    pub tex_coords: Vec2f, // 8 B
}

impl Vertex {
    #[inline]
    pub fn color(&self) -> Color {
        self.color.into()
    }

    #[inline]
    pub fn set_color(&mut self, c: Color) {
        self.color = c.into();
    }

    #[inline]
    pub fn position(&self) -> Vec2f {
        self.position
    }

    #[inline]
    pub fn set_position(&mut self, v: Vec2f) {
        self.position = v;
    }

    #[inline]
    pub fn tex_coords(&self) -> Vec2f {
        self.tex_coords
    }

    #[inline]
    pub fn set_tex_coords(&mut self, tc: Vec2f) {
        self.tex_coords = tc;
    }
}
