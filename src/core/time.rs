use std::time::Duration;
use std::time::SystemTime;

pub struct Time {
    time: SystemTime,
    m_dt: Duration,
}

impl Time {
    pub fn new() -> Time {
        Time {
            time: SystemTime::now(),
            m_dt: Duration::new(0, 0),
        }
    }

    pub fn update(&mut self) {
        self.m_dt = self.time.elapsed().unwrap();
        self.time = SystemTime::now();
    }

    pub fn dt(&self) -> std::time::Duration {
        self.m_dt
    }

    pub fn dt_secs(&self) -> f32 {
        self.m_dt.as_secs() as f32 + self.m_dt.subsec_nanos() as f32 * 1e-9
    }
}
