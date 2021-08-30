use inle_cfg::Cfg_Var;
use inle_ecs::components::base::C_Spatial2D;
use inle_ecs::ecs_world::{Ecs_World, Entity};
use inle_math::vector::Vec2f;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct C_Position_History {
    pub sampling_interval: Duration,
    pub min_delta_pos: f32,

    idx: usize,
    time_since_latest_record: Duration,
}

impl C_Position_History {
    pub fn new(entity: Entity, sampling_interval: Duration, min_delta_pos: f32) -> Self {
        Self {
            idx: entity.index as _,
            sampling_interval,
            min_delta_pos,
            time_since_latest_record: Duration::default(),
        }
    }
}

pub struct Position_History_System {
    // indexed by entities' idx
    positions: Vec<VecDeque<Vec2f>>,
    hist_size: Cfg_Var<u32>,
}

impl Position_History_System {
    pub fn new(hist_size: Cfg_Var<u32>) -> Self {
        Self {
            positions: vec![],
            hist_size,
        }
    }

    pub fn get_positions_of(&self, comp: &C_Position_History) -> &VecDeque<Vec2f> {
        &self.positions[comp.idx]
    }

    pub fn update(&mut self, world: &mut Ecs_World, dt: Duration, cfg: &inle_cfg::Config) {
        trace!("position_history_system::update");

        let hist_size = self.hist_size.read(cfg) as usize;

        foreach_entity!(world, +C_Position_History, +C_Spatial2D, |entity| {
            let spatial = world.get_component::<C_Spatial2D>(entity).unwrap();
            let pos = spatial.transform.position();

            let pos_hist = world.get_component_mut::<C_Position_History>(entity).unwrap();
            pos_hist.time_since_latest_record += dt;

            if self.positions.len() <= pos_hist.idx {
                self.positions.resize_with(pos_hist.idx + 1, || VecDeque::with_capacity(hist_size));
            }
            let prev_positions = &mut self.positions[pos_hist.idx];

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
