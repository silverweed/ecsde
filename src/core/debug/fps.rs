use crate::core::time;

pub struct Fps_Console_Printer {
    pub update_rate: std::time::Duration,
    frames_elapsed: u32,
    time_elapsed: std::time::Duration,
}

impl Fps_Console_Printer {
    pub fn new(update_rate: &std::time::Duration) -> Fps_Console_Printer {
        Fps_Console_Printer {
            update_rate: *update_rate,
            frames_elapsed: 0,
            time_elapsed: std::time::Duration::new(0, 0),
        }
    }

    pub fn tick(&mut self, time: &time::Time) {
        self.frames_elapsed += 1;
        self.time_elapsed += time.real_dt();

        if self.time_elapsed >= self.update_rate {
            eprintln!(
                "FPS: {}",
                self.frames_elapsed as f32 / time::to_secs_frac(&self.update_rate)
            );
            self.frames_elapsed = 0;
            self.time_elapsed = std::time::Duration::new(0, 0);
        }
    }
}
