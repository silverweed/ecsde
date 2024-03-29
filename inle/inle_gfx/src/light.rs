use inle_common::colors::{self, Color};
use inle_math::rect::Rectf;
use inle_math::vector::Vec2f;

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rect_Light {
    pub color: Color,
    pub intensity: f32,
    // Light inside this rect is at its maximum intensity
    pub rect: Rectf,
    pub radius: f32,
    pub attenuation: f32,
}

// We're using 'Commands' rather than allowing direct access to the lights
// so we can batch all the updates and we make clear that changing the lights
// has performance implications (as the UBO needs to be updated etc).
pub enum Light_Command {
    Add_Point_Light(Point_Light),
    Add_Rect_Light(Rect_Light),
    Change_Ambient_Light(Ambient_Light),
    Change_Point_Light(usize, Point_Light),
    Change_Rect_Light(usize, Rect_Light),
}

#[derive(Default)]
pub struct Lights {
    point_lights: Vec<Point_Light>,
    rect_lights: Vec<Rect_Light>,
    ambient_light: Ambient_Light,

    cmd_queue: Vec<Light_Command>,
}

impl Lights {
    pub fn point_lights(&self) -> &[Point_Light] {
        &self.point_lights
    }

    pub fn rect_lights(&self) -> &[Rect_Light] {
        &self.rect_lights
    }

    pub fn ambient_light(&self) -> &Ambient_Light {
        &self.ambient_light
    }

    pub fn queue_command(&mut self, cmd: Light_Command) {
        self.cmd_queue.push(cmd);
    }

    /// Returns true if any commands were processed
    pub fn process_commands(&mut self) -> bool {
        let cmds = self.cmd_queue.split_off(0);
        let are_there_cmds = !cmds.is_empty();
        for cmd in cmds {
            match cmd {
                Light_Command::Add_Point_Light(light) => {
                    self.point_lights.push(light);
                }
                Light_Command::Add_Rect_Light(light) => {
                    self.rect_lights.push(light);
                }
                Light_Command::Change_Ambient_Light(light) => {
                    self.ambient_light = light;
                }
                Light_Command::Change_Point_Light(idx, light) => {
                    assert!(
                        idx < self.point_lights.len(),
                        "Invalid point light index {}",
                        idx
                    );
                    self.point_lights[idx] = light;
                }
                Light_Command::Change_Rect_Light(idx, light) => {
                    assert!(
                        idx < self.rect_lights.len(),
                        "Invalid rect light index {}",
                        idx
                    );
                    self.rect_lights[idx] = light;
                }
            }
        }
        are_there_cmds
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

        get_all_lights_sorted_by_distance_within(
            pos,
            radius,
            result,
            at_most,
            &self.point_lights,
            |pl| pl.position,
        );
    }

    #[inline]
    pub fn get_all_rect_lights_sorted_by_distance_within<E: Extend<Rect_Light>>(
        &self,
        pos: Vec2f,
        radius: f32,
        result: &mut E,
        at_most: usize,
    ) {
        trace!("get_all_rect_lights_sorted_by_distance_within");

        get_all_lights_sorted_by_distance_within(
            pos,
            radius,
            result,
            at_most,
            &self.rect_lights,
            |pl| pl.rect.pos_center(),
        );
    }
}

#[inline]
fn get_all_lights_sorted_by_distance_within<L: Copy, E: Extend<L>>(
    pos: Vec2f,
    radius: f32,
    result: &mut E,
    at_most: usize,
    lights: &[L],
    get_pos_fn: fn(&L) -> Vec2f,
) {
    let radius2 = radius * radius;

    let mut nearest_sorted = Vec::with_capacity(at_most + 1);
    let mut nearest_dist_sorted = Vec::with_capacity(at_most + 1);

    for light in lights {
        debug_assert_eq!(nearest_sorted.len(), nearest_dist_sorted.len());

        let new_dist2 = get_pos_fn(light).distance2(pos);
        if new_dist2 > radius2 {
            continue;
        }

        let insert_pos = nearest_dist_sorted.partition_point(|&dist2| dist2 < new_dist2);
        if insert_pos < at_most {
            nearest_sorted.insert(insert_pos, *light);
            nearest_dist_sorted.insert(insert_pos, new_dist2);
            if nearest_sorted.len() > at_most {
                nearest_sorted.pop();
                nearest_dist_sorted.pop();
            }
        }
    }

    result.extend(nearest_sorted);
}
