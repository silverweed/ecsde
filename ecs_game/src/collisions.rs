use inle_physics::layers::{Collision_Layer, Collision_Matrix};
use std::convert::TryFrom;

enum Collision_Type {
    None,
    Trigger,
    Solid,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Game_Collision_Layer {
    Entities,
    Ground,
    Sky,
    _Count,
}

impl TryFrom<Collision_Layer> for Game_Collision_Layer {
    type Error = String;

    fn try_from(cl: Collision_Layer) -> Result<Game_Collision_Layer, Self::Error> {
        if cl < Game_Collision_Layer::_Count as u8 {
            Ok(unsafe { std::mem::transmute(cl) })
        } else {
            Err(format!("Invalid collision layer: {}", cl))
        }
    }
}

pub fn init_collision_layers(matrix: &mut Collision_Matrix) {
    use Game_Collision_Layer as L;

    matrix.set_layers_collide(L::Entities as _, L::Entities as _);
    matrix.set_layers_collide(L::Entities as _, L::Ground as _);
    matrix.set_layers_collide(L::Entities as _, L::Sky as _);
}

//pub fn collision_is_solid(a: Collision_Mask, b: Collision_Mask) -> bool {
//let and_mask = a & b;

//// 0110010 &
//// 1010011
//// =
//// 0010010
//let mut idx = 0;
//loop {
//let lz = and_mask.leading_zeroes();
//idx += lz;
//and_mask <<= lz;
//if COLLISION_MATRIX[
//}
//}
