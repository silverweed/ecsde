enum Collision_Type {
    None,
    Trigger,
    Solid
}

pub const LAYER_ENTITIES = 1;
pub const LAYER_GROUND = 2;
pub const LAYER_SKY = 4;

const COLLISION_MATRIX: [[Collision_Type; N]; N] = [
    [Collision_Type::Solid, Collision_Type::Solid, Collision_Type::Solid],
    [Collision_Type::Solid, Collision_Type::None, Collision_Type::None],
    [Collision_Type::Solid, Collision_Type::None, Collision_Type::None],
];

pub fn collision_is_solid(a: Collision_Mask, b: Collision_Mask) -> bool {
}
