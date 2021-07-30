use std::time::Duration;
use std::time::Instant;

pub struct Time {
    /// The starting instant, used as reference to compute real_time.
    start_time: Instant,

    /// How much time elapsed since the start of Time, as latest recorded during update()
    real_time: Duration,

    /// The latest updated real delta time
    real_dt: Duration,

    /// How much game time (i.e. real time scaled/paused) elapsed since the start of Time, as latest recorded during update()
    game_time: Duration,

    /// The value of game_time during the previous frame
    prev_game_time: Duration,

    /// The latest updated game delta time
    dt: Duration,

    /// How fast does game_time pass relative to real_time
    pub time_scale: f32,

    /// Whether game_time passes or not
    pub paused: bool,

    /// Holds the previous value of `paused`
    was_paused: bool,

    stepping: bool,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        Time {
            start_time: now,
            real_time: now.elapsed(),
            real_dt: Duration::default(),
            game_time: Duration::default(),
            prev_game_time: Duration::default(),
            dt: Duration::default(),
            time_scale: 1.,
            paused: false,
            was_paused: false,
            stepping: false,
        }
    }
}

impl Time {
    pub fn start(&mut self) {
        assert!(
            self.game_time.as_micros() == 0,
            "Time::start() called while already running!"
        );
        self.start_time = Instant::now();
    }

    pub fn update(&mut self) {
        self.stepping = false;

        // Update real time
        let prev_real_time = self.real_time;
        self.real_time = self.start_time.elapsed();
        self.real_dt = self.real_time - prev_real_time;

        // Update game time
        self.prev_game_time = self.game_time;
        self.was_paused = self.paused;

        self.dt = Duration::from_micros(
            (((self.real_dt.as_micros() * (!self.paused as u128)) as f32) * self.time_scale) as u64,
        );
        self.game_time += self.dt;
    }

    #[inline]
    pub fn step(&mut self, dt: &Duration) {
        self.prev_game_time = self.game_time;
        self.game_time += *dt;
        self.stepping = true;
    }

    #[inline(always)]
    pub fn is_stepping(&self) -> bool {
        self.stepping
    }

    #[inline(always)]
    pub fn dt(&self) -> Duration {
        self.dt
    }

    #[inline(always)]
    pub fn real_dt(&self) -> Duration {
        self.real_dt
    }

    #[inline(always)]
    pub fn dt_secs(&self) -> f32 {
        self.dt().as_secs_f32()
    }

    #[inline(always)]
    pub fn pause_toggle(&mut self) {
        self.paused = !self.paused;
    }

    #[inline(always)]
    pub fn was_paused(&self) -> bool {
        self.was_paused
    }

    #[inline(always)]
    pub fn real_time(&self) -> Duration {
        self.real_time
    }

    #[inline(always)]
    pub fn game_time(&self) -> Duration {
        self.game_time
    }
}

#[inline(always)]
pub fn to_ms_frac(d: &Duration) -> f32 {
    d.as_secs_f32() * 1000.
}

// @WaitForStable: replace with div_duration() when API is stable
#[inline]
pub fn duration_ratio(a: &Duration, b: &Duration) -> f32 {
    a.as_secs_f32() / b.as_secs_f32()
}

    #[inline]
pub fn mul_duration(d: &Duration, s: f32) -> Duration {
    Duration::from_secs_f32(d.as_secs_f32() * s)
}
