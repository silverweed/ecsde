use crate::core::time;
use std::time::Duration;

pub struct Fps_Console_Printer {
    pub update_rate: Duration,
    tag: String,
    frames_elapsed: u32,
    time_elapsed: Duration,
}

impl Fps_Console_Printer {
    pub fn new(update_rate: &Duration, tag: &str) -> Fps_Console_Printer {
        Fps_Console_Printer {
            update_rate: *update_rate,
            tag: String::from(tag),
            frames_elapsed: 0,
            time_elapsed: Duration::new(0, 0),
        }
    }

    pub fn tick(&mut self, dt: &Duration) {
        self.frames_elapsed += 1;
        self.time_elapsed += *dt;

        if self.time_elapsed >= self.update_rate {
            eprintln!(
                "[{}] Avg. FPS: {} | Time: {:.4} ms",
                self.tag,
                self.frames_elapsed as f32 / time::to_secs_frac(&self.update_rate),
                (self.time_elapsed / self.frames_elapsed).as_nanos() as f32 * 0.000_001
            );
            self.frames_elapsed = 0;
            self.time_elapsed = Duration::new(0, 0);
        }
    }
}
