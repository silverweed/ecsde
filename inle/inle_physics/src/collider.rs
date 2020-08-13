use super::layers::Collision_Layer;
use super::phys_world::Physics_Body_Handle;
use inle_ecs::ecs_world::Entity;
use inle_math::vector::Vec2f;
use smallvec::SmallVec;

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
pub struct Collider {
    pub shape: Collision_Shape,
    pub position: Vec2f, // This should not be set directly: it's computed by collision system
    pub offset: Vec2f,
    pub colliding_with: SmallVec<[Entity; 2]>,
    pub is_static: bool,
    pub layer: Collision_Layer,
    pub entity: Entity,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Collider {
    pub handle: Physics_Body_Handle,
}
