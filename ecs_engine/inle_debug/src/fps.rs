use std::time::Duration;

pub struct Fps_Counter {
    pub update_rate: Duration,
    frames_elapsed: u32,
    time_elapsed: Duration,
    latest_calc_fps: f32,
}

impl Fps_Counter {
    pub fn with_update_rate(update_rate: &Duration) -> Self {
        Self {
            update_rate: *update_rate,
            frames_elapsed: 0,
            time_elapsed: Duration::new(0, 0),
            latest_calc_fps: 0.0,
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.latest_calc_fps
    }

    pub fn get_instant_fps(&self) -> f32 {
        self.frames_elapsed as f32 / self.time_elapsed.as_secs_f32()
    }

    pub fn tick(&mut self, dt: &Duration) {
        self.frames_elapsed += 1;
        self.time_elapsed += *dt;

        if self.time_elapsed >= self.update_rate {
            self.latest_calc_fps = self.frames_elapsed as f32 / self.update_rate.as_secs_f32();
            self.frames_elapsed = 0;
            self.time_elapsed = Duration::new(0, 0);
        }
    }
}
