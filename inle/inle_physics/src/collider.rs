use super::layers::Collision_Layer;
use super::phys_world::Collider_Handle;
use inle_cfg::Cfg_Var;
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

#[derive(Clone, Debug, Default)]
pub struct Phys_Data {
    pub inv_mass: Cfg_Var<f32>,
    pub restitution: Cfg_Var<f32>,
    pub static_friction: Cfg_Var<f32>,
    pub dyn_friction: Cfg_Var<f32>,
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
