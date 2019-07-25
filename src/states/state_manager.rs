use super::state::{Game_State, Persistent_Game_State, State_Transition};
use crate::cfg;
use crate::core::input::Action_List;
use crate::core::world::World;
use std::vec::Vec;

/// Manages a PDA of Game_States.
pub struct State_Manager {
    /// Only the topmost state is updated and queried for actions at any time.
    state_stack: Vec<Box<dyn Game_State>>,

    /// Persistent states are always updated and their handle_actions is always called.
    /// Update is called for every persistent state *before* the regular states, whereas
    /// handle_actions is called after.
    persistent_states: Vec<Box<dyn Persistent_Game_State>>,
}

impl State_Manager {
    pub fn new() -> State_Manager {
        State_Manager {
            state_stack: vec![],
            persistent_states: vec![],
        }
    }

    pub fn with_initial_state(world: &World, mut state: Box<dyn Game_State>) -> State_Manager {
        state.on_start(world);
        State_Manager {
            state_stack: vec![state],
            persistent_states: vec![],
        }
    }

    pub fn update(&mut self, world: &World) {
        for state in &mut self.persistent_states {
            state.update(world);
        }

        if let Some(state) = self.current_state() {
            match state.update(world) {
                State_Transition::None => {}
                State_Transition::Push(new_state) => self.push_state(world, new_state),
                State_Transition::Replace(new_state) => self.replace_state(world, new_state),
                State_Transition::Pop => self.pop_state(world),
            }
        }
    }

    /// Returns true if should quit
    pub fn handle_actions(
        &mut self,
        actions: &Action_List,
        world: &World,
        config: &cfg::Config,
    ) -> bool {
        let mut should_quit = false;

        if let Some(state) = self.current_state() {
            should_quit |= state.handle_actions(actions, world, config);
        }

        for state in &mut self.persistent_states {
            should_quit |= state.handle_actions(actions, world, config);
        }

        should_quit
    }

    pub fn add_persistent_state(
        &mut self,
        world: &World,
        mut state: Box<dyn Persistent_Game_State>,
    ) {
        state.on_start(world);
        self.persistent_states.push(state);
    }

    #[inline]
    fn current_state(&mut self) -> Option<&mut Box<dyn Game_State>> {
        let len = self.state_stack.len();
        if len > 0 {
            Some(&mut self.state_stack[len - 1])
        } else {
            None
        }
    }

    fn push_state(&mut self, world: &World, mut state: Box<dyn Game_State>) {
        self.current_state().map(|s| s.on_pause(world));
        state.on_start(world);
        self.state_stack.push(state);
    }

    fn pop_state(&mut self, world: &World) {
        if let Some(mut prev_state) = self.state_stack.pop() {
            prev_state.on_end(world);
        } else {
            eprintln!("[ ERROR ] Tried to pop state, but state stack is empty!");
        }

        if let Some(state) = self.current_state() {
            state.on_resume(world);
        }
    }

    fn replace_state(&mut self, world: &World, mut state: Box<dyn Game_State>) {
        if let Some(mut prev_state) = self.state_stack.pop() {
            prev_state.on_end(world);
        }

        state.on_start(world);
        self.state_stack.push(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct Test_State_Data {
        pub started: bool,
        pub paused: bool,
        pub resumed: bool,
        pub ended: bool,
        pub updated: i32,
        pub handled_actions: i32,
    }

    #[derive(Default)]
    struct Test_State_1 {
        pub data: Rc<RefCell<Test_State_Data>>,
    }

    impl Game_State for Test_State_1 {
        fn on_start(&mut self, _world: &World) {
            self.data.borrow_mut().started = true;
        }
        fn on_end(&mut self, _world: &World) {
            self.data.borrow_mut().ended = true;
        }
        fn on_pause(&mut self, _world: &World) {
            self.data.borrow_mut().paused = true;
        }
        fn on_resume(&mut self, _world: &World) {
            self.data.borrow_mut().resumed = true;
        }
        fn update(&mut self, _world: &World) -> State_Transition {
            self.data.borrow_mut().updated += 1;
            if self.data.borrow().updated < 2 {
                State_Transition::None
            } else {
                State_Transition::Pop
            }
        }
        fn handle_actions(
            &mut self,
            _actions: &Action_List,
            _world: &World,
            _config: &cfg::Config,
        ) -> bool {
            self.data.borrow_mut().handled_actions += 1;
            false
        }
    }

    // @Incomplete: pause/resume and persistent states are not covered
    #[test]
    fn state_manager() {
        let data = Rc::new(RefCell::new(Test_State_Data::default()));
        let state = Box::new(Test_State_1 { data: data.clone() });
        let world = World::new();
        let mut smgr = State_Manager::with_initial_state(&world, state);

        assert!(data.borrow().started, "State was not started");
        assert!(!data.borrow().ended, "State was ended");
        assert!(!data.borrow().resumed, "State was resumed");
        assert!(!data.borrow().paused, "State was paused");
        assert_eq!(data.borrow().updated, 0);
        assert_eq!(data.borrow().handled_actions, 0);

        smgr.update(&world);
        assert_eq!(data.borrow().updated, 1);
        assert_eq!(data.borrow().handled_actions, 0);

        let actions = Action_List::default();
        let cfg = cfg::Config::new_empty();
        smgr.handle_actions(&actions, &world, &cfg);
        assert_eq!(data.borrow().handled_actions, 1);

        smgr.update(&world); // this pops the state
        assert_eq!(data.borrow().updated, 2, "State was not updated");
        assert!(data.borrow().ended, "State was not ended");

        smgr.handle_actions(&actions, &world, &cfg);
        assert_eq!(
            data.borrow().handled_actions,
            1,
            "State was handled but should have been popped"
        );

        smgr.update(&world);
        assert_eq!(
            data.borrow().updated,
            2,
            "State was updated but should have been popped"
        );
    }
}
