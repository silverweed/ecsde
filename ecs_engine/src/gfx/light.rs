use crate::common::colors::{self, Color};
use crate::common::vector::Vec2f;

#[derive(Copy, Clone, Debug)]
pub struct Point_Light {
    pub color: Color,
    pub position: Vec2f,
    pub radius: f32,
    // Exponent used in 1/d^x
    pub attenuation: f32,
}

impl Default for Point_Light {
    fn default() -> Self {
        Self {
            position: v2!(0., 0.),
            radius: 0.,
            attenuation: 1.,
            color: colors::WHITE,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Ambient_Light {
    pub color: Color,
    pub intensity: f32,
}

impl Default for Ambient_Light {
    fn default() -> Self {
        Self {
            color: colors::WHITE,
            intensity: 1.,
        }
    }
}

const MAX_POINT_LIGHTS: usize = 64;

pub struct Lights {
    pub point_lights: [Point_Light; MAX_POINT_LIGHTS],
    pub n_actual_point_lights: usize,
    pub ambient_light: Ambient_Light,
}

impl Default for Lights {
    fn default() -> Self {
        Self {
            point_lights: [Point_Light::default(); MAX_POINT_LIGHTS],
            n_actual_point_lights: 0,
            ambient_light: Ambient_Light::default(),
        }
    }
}

impl Lights {
    pub fn add_point_light(&mut self, light: Point_Light) {
        assert!(
            self.n_actual_point_lights < self.point_lights.len(),
            "Too many point lights!"
        );
        self.point_lights[self.n_actual_point_lights] = light;
        self.n_actual_point_lights += 1;
    }

    pub fn get_nearest_point_light(&self, pos: Vec2f) -> Option<&Point_Light> {
        let mut nearest = None;
        let mut nearest_dist2 = -1.;
        for pl in &self.point_lights[..self.n_actual_point_lights] {
            let dist2 = pl.position.distance2(pos);
            if nearest_dist2 < 0. || dist2 <= nearest_dist2 {
                nearest = Some(pl);
                nearest_dist2 = dist2;
            }
        }
        nearest
    }
}
