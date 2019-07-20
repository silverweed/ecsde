use super::msg;
use super::systems;
use super::time_manager;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub struct World {
    time: Rc<RefCell<time_manager::Time_Manager>>,
    systems: systems::Core_Systems,
    dispatcher: msg::Msg_Dispatcher,

    // Cache delta time values
    dt: Duration,
    real_dt: Duration,
}

impl World {
    pub fn new() -> World {
        World {
            time: Rc::new(RefCell::new(time_manager::Time_Manager::new())),
            systems: systems::Core_Systems::new(),
            dispatcher: msg::Msg_Dispatcher::new(),
            dt: Duration::new(0, 0),
            real_dt: Duration::new(0, 0),
        }
    }

    pub fn init(&mut self) {
        let disp = &mut self.dispatcher;
        let systems = &mut self.systems;
        disp.register(self.time.clone());
        disp.register(systems.ui_system.clone());
        disp.register(systems.gameplay_system.clone());
    }

    pub fn update(&mut self) {
        self.time.borrow_mut().time.update();
        let time = &self.time.borrow().time;
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

    pub fn get_dispatcher(&self) -> &msg::Msg_Dispatcher {
        &self.dispatcher
    }
}
