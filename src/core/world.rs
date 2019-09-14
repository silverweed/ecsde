use super::env::Env_Info;
use super::systems;
use super::time::Time;
use std::cell::RefCell;
use std::time::Duration;

pub struct World {
    pub time: RefCell<Time>,
    systems: systems::Core_Systems,

    // Cache delta time values
    dt: Duration,
    real_dt: Duration,
}

impl World {
    pub fn new(env: &Env_Info) -> World {
        World {
            time: RefCell::new(Time::new()),
            systems: systems::Core_Systems::new(env),
            dt: Duration::new(0, 0),
            real_dt: Duration::new(0, 0),
        }
    }

    pub fn init(&mut self) {}

    pub fn update(&mut self) {
        let time = &mut self.time.borrow_mut();
        time.update();
        self.dt = time.dt();
        self.real_dt = time.real_dt();
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }

    pub fn real_dt(&self) -> Duration {
        self.real_dt
    }

    pub fn get_systems(&self) -> &systems::Core_Systems {
        &self.systems
    }
}
