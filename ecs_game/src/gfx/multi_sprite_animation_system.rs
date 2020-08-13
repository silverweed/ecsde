use inle_alloc::temp::*;
use inle_gfx::components::C_Multi_Renderable;
use inle_ecs::ecs_world::Ecs_World;
use std::time::Duration;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Animation_Track {
    None,
    Sinusoidal {
        freq_hz: f32,
        amplitude: f32,
        phase: f32,
        exp: i32,
    },
    Abs_Sinusoidal {
        freq_hz: f32,
        amplitude: f32,
        phase: f32,
        exp: f32,
    },
}

impl Default for Animation_Track {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct C_Multi_Renderable_Animation {
    pub anim_tracks_x: [Animation_Track; C_Multi_Renderable::MAX_RENDERABLES],
    pub anim_tracks_y: [Animation_Track; C_Multi_Renderable::MAX_RENDERABLES],
    pub anim_t: [f32; C_Multi_Renderable::MAX_RENDERABLES],
}

pub fn update(dt: &Duration, world: &mut Ecs_World, frame_alloc: &mut Temp_Allocator) {
    foreach_entity!(world, +C_Multi_Renderable, +C_Multi_Renderable_Animation, |entity| {
        let mr = world.get_component::<C_Multi_Renderable>(entity).unwrap();
        let n_renderables = mr.n_renderables;

        let mr_anim = world.get_component_mut::<C_Multi_Renderable_Animation>(entity).unwrap();

        #[derive(Copy, Clone)]
        struct Anim_Result {
            pub filled_x: bool,
            pub filled_y: bool,
            pub result_x: f32,
            pub result_y: f32
        }

        fn fill_result(track: Animation_Track, t: f32, result: &mut f32) {
            match track {
                Animation_Track::Sinusoidal {
                    freq_hz,
                    amplitude,
                    exp,
                    phase
                } => {
                    *result = f32::powi(amplitude * f32::sin(t * freq_hz + phase), exp);
                },
                Animation_Track::Abs_Sinusoidal {
                    freq_hz,
                    amplitude,
                    exp,
                    phase
                } => {
                    *result = f32::powf((amplitude * f32::sin(t * freq_hz + phase)).abs(), exp);
                },
                _ => ()
            }
        }

        let mut anim_results = excl_temp_array(frame_alloc);
        for i in 0..n_renderables as usize {
            mr_anim.anim_t[i] += dt.as_secs_f32();
            let t = mr_anim.anim_t[i];

            let track_x = mr_anim.anim_tracks_x[i];
            let filled_x = track_x != Animation_Track::None;

            let track_y = mr_anim.anim_tracks_y[i];
            let filled_y = track_y != Animation_Track::None;

            let mut result = Anim_Result {
                filled_x,
                filled_y,
                result_x: 0.,
                result_y: 0.
            };

            fill_result(track_x, t, &mut result.result_x);
            fill_result(track_y, t, &mut result.result_y);

            anim_results.push(result);
        }

        let mr = world.get_component_mut::<C_Multi_Renderable>(entity).unwrap();
        for (i, result) in anim_results.iter().enumerate() {
            if result.filled_x {
                let pos = mr.rend_transforms[i].position();
                mr.rend_transforms[i].set_position(result.result_x, pos.y);
            }
            if result.filled_y {
                let pos = mr.rend_transforms[i].position();
                mr.rend_transforms[i].set_position(pos.x, result.result_y);
            }
        }
    });
}
