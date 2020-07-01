use std::time::Duration;
use std::time::SystemTime;

type Microseconds = u64;

// @Cleanup: do we still all this?
pub struct Time {
    game_time: Microseconds,
    prev_game_time: Microseconds,
    start_time: SystemTime,
    real_time: SystemTime,
    dt: Duration,
    real_dt: Duration,
    prev_real_time: SystemTime,
    pub time_scale: f32,
    pub paused: bool,
    pub was_paused: bool,
}

impl Default for Time {
    fn default() -> Self {
        let now = SystemTime::now();
        Time {
            real_time: now,
            prev_real_time: now,
            start_time: now,
            game_time: 0,
            prev_game_time: 0,
            dt: Duration::default(),
            real_dt: Duration::default(),
            time_scale: 1.,
            paused: false,
            was_paused: false,
        }
    }
}

impl Time {
    const MAX_FRAME_TIME: Microseconds = 1_000_000 / 15;

    pub fn start(&mut self) {
        assert!(
            self.game_time == 0,
            "Time::start() called while already running!"
        );
        let now = SystemTime::now();
        self.start_time = now;
        self.prev_real_time = now;
    }

    pub fn update(&mut self) {
        let now = SystemTime::now();

        self.prev_real_time = self.real_time;
        self.real_time = now;

        self.prev_game_time = self.game_time;

        self.was_paused = self.paused;
        if !self.paused {
            let real_delta = now
                .duration_since(self.prev_real_time)
                .unwrap_or(Duration::from_secs(0));

            self.game_time += (self.time_scale * real_delta.as_micros() as f32) as Microseconds;
        }

        self.dt = self.calc_dt();
        self.real_dt = self.calc_real_dt();
    }

    pub fn step(&mut self, dt: &Duration) {
        self.prev_game_time = self.game_time;
        self.game_time += (dt.as_secs_f32() * 1_000_000.0) as Microseconds;
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }

    pub fn real_dt(&self) -> Duration {
        self.real_dt
    }

    pub fn dt_secs(&self) -> f32 {
        self.dt().as_secs_f32()
    }

    pub fn pause_toggle(&mut self) {
        self.paused = !self.paused;
    }

    pub fn get_real_time(&self) -> Duration {
        self.real_time.duration_since(self.start_time).unwrap()
    }

    pub fn get_game_time(&self) -> Duration {
        Duration::from_micros(self.game_time)
    }

    fn calc_dt(&self) -> Duration {
        let tscale = self.time_scale;
        let delta_microseconds = self.game_time - self.prev_game_time;
        let scaled_max_frame_time = (Self::MAX_FRAME_TIME as f32 * tscale) as Microseconds;
        let delta_microseconds = if delta_microseconds > scaled_max_frame_time {
            // frame lock
            scaled_max_frame_time
        } else {
            delta_microseconds
        };

        Duration::from_micros(delta_microseconds)
    }

    fn calc_real_dt(&self) -> Duration {
        self.real_time.duration_since(self.prev_real_time).unwrap()
    }
}

#[inline(always)]
pub fn to_ms_frac(d: &Duration) -> f32 {
    d.as_secs_f32() * 1000.
}

// @WaitForStable: replace with div_duration() when API is stable
pub fn duration_ratio(a: &Duration, b: &Duration) -> f32 {
    a.as_secs_f32() / b.as_secs_f32()
}

pub fn mul_duration(d: &Duration, s: f32) -> Duration {
    Duration::from_secs_f32(d.as_secs_f32() * s)
}
