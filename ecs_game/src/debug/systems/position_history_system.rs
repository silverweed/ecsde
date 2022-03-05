use crate::systems::interface::{Game_System, Update_Args};
use inle_cfg::{Cfg_Var, Config};
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_query_new::Ecs_Query;
use inle_math::vector::Vec2f;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct C_Position_History {
    pub sampling_interval: Duration,
    pub min_delta_pos: f32,
    pub positions: VecDeque<Vec2f>,

    time_since_latest_record: Duration,
}

impl C_Position_History {
    pub fn new(sampling_interval: Duration, min_delta_pos: f32) -> Self {
        Self {
            sampling_interval,
            min_delta_pos,
            time_since_latest_record: Duration::default(),
            positions: VecDeque::default(),
        }
    }
}

pub struct Position_History_System {
    active: Cfg_Var<bool>,
    hist_size: Cfg_Var<u32>,
    query: Ecs_Query,
}

impl Position_History_System {
    pub fn new(cfg: &Config) -> Self {
        Self {
            active: Cfg_Var::new("debug/entities/pos_history/draw", cfg),
            hist_size: Cfg_Var::new("debug/entities/pos_history/hist_size", cfg),
            query: Ecs_Query::default()
                .require::<C_Spatial2D>()
                .require::<C_Position_History>(),
        }
    }
}

impl Game_System for Position_History_System {
    fn get_queries_mut(&mut self) -> Vec<&mut Ecs_Query> {
        vec![&mut self.query]
    }

    fn update(&self, args: &mut Update_Args) {
        trace!("position_history_system::update");

        let Update_Args {
            ecs_world: world,
            engine_state,
            dt,
            ..
        } = args;

        let cfg = &engine_state.config;
        let active = self.active.read(cfg);
        if !active {
            return;
        }

        let hist_size = self.hist_size.read(cfg) as usize;

        foreach_entity!(self.query, world,
            read: C_Spatial2D;
            write: C_Position_History;
            |_e, (spatial,): (&C_Spatial2D,), (pos_hist,): (&mut C_Position_History,)| {
            let pos = spatial.transform.position();

            pos_hist.time_since_latest_record += *dt;

            let prev_positions = &mut pos_hist.positions;

            let min_delta_pos_sqr = pos_hist.min_delta_pos * pos_hist.min_delta_pos;
            if pos_hist.time_since_latest_record >= pos_hist.sampling_interval {
                let dist_sqr = if !prev_positions.is_empty() {
                    (pos - prev_positions[prev_positions.len() - 1]).magnitude2()
                } else {
                    -1.0
                };
                if dist_sqr < 0.0 || dist_sqr >= min_delta_pos_sqr {
                    pos_hist.time_since_latest_record -= pos_hist.sampling_interval;
                    prev_positions.push_back(pos);
                    let eccess = prev_positions.len().saturating_sub(hist_size);
                    if eccess > 0 {
                        prev_positions.drain(0..eccess);
                    }
                }
            }
        });
    }
}
