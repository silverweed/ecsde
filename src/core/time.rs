use std::time::Duration;
use std::time::SystemTime;

type Time_t = u64;

pub struct Time {
    game_time: Time_t, // in microseconds
    prev_game_time: Time_t,
    real_time: SystemTime,
    time_scale: f32,
}

impl Time {
    const MAX_FRAME_TIME: Time_t = 1_000_000 / 30;

    pub fn new() -> Time {
        Time {
            game_time: 0,
            prev_game_time: 0,
            real_time: SystemTime::now(),
            time_scale: 1.0,
        }
    }

    pub fn update(&mut self) {
        let now = SystemTime::now();
        let real_delta = now
            .duration_since(self.real_time)
            .unwrap_or(Duration::from_secs(0));
        self.real_time = now;

        self.prev_game_time = self.game_time;
        // TODO use `as_micros()` when the feature becomes stable
        self.game_time += (to_secs_frac(&real_delta) * self.time_scale * 1_000_000.0) as Time_t;
    }

    pub fn dt(&self) -> std::time::Duration {
        let tscale = self.time_scale;
        let delta_microseconds = self.game_time - self.prev_game_time;
        let scaled_max_frame_time = (Self::MAX_FRAME_TIME as f32 * tscale) as Time_t;
        let delta_microseconds = if delta_microseconds > scaled_max_frame_time {
            // frame lock
            scaled_max_frame_time
        } else {
            delta_microseconds
        };

        std::time::Duration::from_micros(delta_microseconds)
    }

    pub fn dt_secs(&self) -> f32 {
        to_secs_frac(&self.dt())
    }

    pub fn real_dt(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.real_time).unwrap()
    }

    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale;
    }

    pub fn get_time_scale(&self) -> f32 {
        self.time_scale
    }
}

pub fn to_secs_frac(d: &Duration) -> f32 {
    d.as_secs() as f32 + d.subsec_nanos() as f32 * 1e-9
}

pub fn duration_ratio(a: &Duration, b: &Duration) -> f32 {
    to_secs_frac(a) / to_secs_frac(b)
}
