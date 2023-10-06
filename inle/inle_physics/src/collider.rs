use super::layers::Collision_Layer;
use super::phys_world::{Collider_Handle, Physics_Body_Handle};
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Collision_Shape {
    Rect { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl Collision_Shape {
    pub fn extent(self) -> Vec2f {
        match self {
            Collision_Shape::Circle { radius } => v2!(radius, radius) * 2.,
            Collision_Shape::Rect { width, height } => v2!(width, height),
        }
    }
}

impl Default for Collision_Shape {
    fn default() -> Collision_Shape {
        Collision_Shape::Circle { radius: 0. }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Phys_Data {
    pub inv_mass: f32,
    pub restitution: f32,
    pub static_friction: f32,
    pub dyn_friction: f32,
}

impl Phys_Data {
    pub fn with_mass(self, mass: f32) -> Self {
        assert!(mass > 0., "Mass must be positive!");
        Self {
            inv_mass: 1.0 / mass,
            ..self
        }
    }

    pub fn with_infinite_mass(self) -> Self {
        Self {
            inv_mass: 0.,
            ..self
        }
    }

    pub fn with_restitution(self, restitution: f32) -> Self {
        Self {
            restitution,
            ..self
        }
    }

    pub fn with_static_friction(self, static_friction: f32) -> Self {
        Self {
            static_friction,
            ..self
        }
    }

    pub fn with_dyn_friction(self, dyn_friction: f32) -> Self {
        Self {
            dyn_friction,
            ..self
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Collider {
    pub shape: Collision_Shape,
    pub is_static: bool,
    pub layer: Collision_Layer,

    pub offset: Vec2f,

    // XXX: it'd be nice to get rid of this
    pub handle: Collider_Handle,

    // physics::update_collisions() will read and update these values
    pub position: Vec2f,
    pub velocity: Vec2f,

    // @Speed: evaluate whether this would be better off in a different array
    pub phys_data: Option<Phys_Data>,
}
