use std::time::Duration;

pub struct Fps_Console_Printer {
    pub update_rate: Duration,
    tag: String,
    frames_elapsed: u32,
    time_elapsed: Duration,
    latest_calc_fps: f32,
}

impl Fps_Console_Printer {
    pub fn new(update_rate: &Duration, tag: &str) -> Fps_Console_Printer {
        Fps_Console_Printer {
            update_rate: *update_rate,
            tag: String::from(tag),
            frames_elapsed: 0,
            time_elapsed: Duration::new(0, 0),
            latest_calc_fps: 0.0,
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.latest_calc_fps
    }

    pub fn tick(&mut self, dt: &Duration) {
        self.frames_elapsed += 1;
        self.time_elapsed += *dt;

        if self.time_elapsed >= self.update_rate {
            self.latest_calc_fps =
                self.frames_elapsed as f32 / self.update_rate.as_secs_f32();
            self.frames_elapsed = 0;
            self.time_elapsed = Duration::new(0, 0);
        }
    }
}
