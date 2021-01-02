use inle_common::colors::{self, Color};
use inle_math::rect::Rectf;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug)]
pub struct Point_Light {
    pub color: Color,
    pub position: Vec2f,
    pub radius: f32,
    // Exponent used in 1/d^x (not actually used right now)
    pub attenuation: f32,
    pub intensity: f32,
}

impl Default for Point_Light {
    fn default() -> Self {
        Self {
            position: v2!(0., 0.),
            radius: 0.,
            attenuation: 1.,
            color: colors::WHITE,
            intensity: 1.,
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

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect_Light {
    pub color: Color,
    // Light inside this rect is at its maximum intensity
    pub rect: Rectf,
    pub radius: f32,
    pub attenuation: f32,
    pub intensity: f32,
}

pub struct Lights {
    pub point_lights: Vec<Point_Light>,
    pub rect_lights: Vec<Rect_Light>,
    pub ambient_light: Ambient_Light,
}

impl Default for Lights {
    fn default() -> Self {
        Self {
            point_lights: vec![],
            rect_lights: vec![],
            ambient_light: Ambient_Light::default(),
        }
    }
}

impl Lights {
    pub fn add_point_light(&mut self, light: Point_Light) {
        self.point_lights.push(light);
    }

    pub fn add_rect_light(&mut self, light: Rect_Light) {
        self.rect_lights.push(light);
    }

    pub fn get_nearest_point_light(&self, pos: Vec2f) -> Option<&Point_Light> {
        let mut nearest = None;
        let mut nearest_dist2 = -1.;
        for pl in &self.point_lights {
            let dist2 = pl.position.distance2(pos);
            if nearest_dist2 < 0. || dist2 <= nearest_dist2 {
                nearest = Some(pl);
                nearest_dist2 = dist2;
            }
        }
        nearest
    }

    pub fn get_all_point_lights_sorted_by_distance_within<E: Extend<Point_Light>>(
        &self,
        pos: Vec2f,
        radius: f32,
        result: &mut E,
        at_most: usize,
    ) {
        trace!("get_all_point_lights_sorted_by_distance_within");

        let radius2 = radius * radius;
        // @Speed
        let mut lights = self
            .point_lights
            .iter()
            .filter(|pl| pl.position.distance2(pos) < radius2)
            .copied()
            .collect::<Vec<_>>();
        lights.sort_by(|a, b| {
            a.position
                .distance2(pos)
                .partial_cmp(&b.position.distance2(pos))
                .unwrap()
        });
        result.extend(lights.into_iter().take(at_most));
    }
}
