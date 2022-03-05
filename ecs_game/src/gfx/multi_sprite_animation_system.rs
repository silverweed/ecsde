use crate::systems::interface::{Game_System, Update_Args};
use inle_alloc::temp::*;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_gfx::components::C_Multi_Renderable;

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

pub struct Multi_Sprite_Animation_System {
    query: Ecs_Query,
}

impl Multi_Sprite_Animation_System {
    pub fn new() -> Self {
        Self {
            query: Ecs_Query::default()
                .require::<C_Multi_Renderable>()
                .require::<C_Multi_Renderable_Animation>(),
        }
    }
}

impl Game_System for Multi_Sprite_Animation_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }

    fn update(&self, args: &mut Update_Args) {
        let Update_Args {
            dt,
            ecs_world: world,
            engine_state,
            ..
        } = args;
        let frame_alloc = &mut engine_state.frame_alloc;

        foreach_entity!(self.query, world,
            read: ;
            write: C_Multi_Renderable, C_Multi_Renderable_Animation;
            |entity, (), (mr, mr_anim): (&mut C_Multi_Renderable, &mut C_Multi_Renderable_Animation)| {
            let n_renderables = mr.n_renderables;

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

            let mut mr = world.get_component_mut::<C_Multi_Renderable>(entity).unwrap();
            for (i, result) in anim_results.iter().enumerate() {
                let transform = &mut mr.renderables[i].sprite_local_transform;
                let pos = transform.position();
                let pos_x = if result.filled_x { result.result_x } else { pos.x };
                let pos_y = if result.filled_y { result.result_y } else { pos.y };
                transform.set_position(pos_x, pos_y);
            }
        });
    }
}
