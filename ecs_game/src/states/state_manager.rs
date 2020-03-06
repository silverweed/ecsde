use super::state::{Game_State, Persistent_Game_State, State_Transition};
use crate::gameplay_system::Gameplay_System;
use ecs_engine::core::app::Engine_State;
use ecs_engine::input::input_system::Game_Action;
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

    pub fn update(&mut self, engine_state: &mut Engine_State, gs: &mut Gameplay_System) {
        for state in &mut self.persistent_states {
            state.update(engine_state, gs);
        }

        if let Some(state) = self.current_state() {
            match state.update(engine_state, gs) {
                State_Transition::None => {}
                State_Transition::Push(new_state) => self.push_state(new_state, engine_state, gs),
                State_Transition::Replace(new_state) => {
                    self.replace_state(engine_state, gs, new_state)
                }
                State_Transition::Pop => self.pop_state(engine_state, gs),
            }
        }
    }

    /// Returns true if should quit
    pub fn handle_actions(
        &mut self,
        actions: &[Game_Action],
        engine_state: &mut Engine_State,
        gs: &mut Gameplay_System,
    ) -> bool {
        let mut should_quit = false;

        if let Some(state) = self.current_state() {
            should_quit |= state.handle_actions(actions, engine_state, gs);
        }

        for state in &mut self.persistent_states {
            should_quit |= state.handle_actions(actions, engine_state, gs);
        }

        should_quit
    }

    pub fn add_persistent_state(
        &mut self,
        engine_state: &mut Engine_State,
        gs: &mut Gameplay_System,
        mut state: Box<dyn Persistent_Game_State>,
    ) {
        state.on_start(engine_state, gs);
        self.persistent_states.push(state);
    }

    #[inline]
    fn current_state(&mut self) -> Option<&mut dyn Game_State> {
        let len = self.state_stack.len();
        if len > 0 {
            Some(&mut *self.state_stack[len - 1])
        } else {
            None
        }
    }

    fn push_state(
        &mut self,
        mut state: Box<dyn Game_State>,
        engine_state: &mut Engine_State,
        gs: &mut Gameplay_System,
    ) {
        if let Some(s) = self.current_state() {
            s.on_pause(engine_state, gs);
        }
        state.on_start(engine_state, gs);
        self.state_stack.push(state);
    }

    fn pop_state(&mut self, engine_state: &mut Engine_State, gs: &mut Gameplay_System) {
        if let Some(mut prev_state) = self.state_stack.pop() {
            prev_state.on_end(engine_state, gs);
        } else {
            lerr!("Tried to pop state, but state stack is empty!");
        }

        if let Some(state) = self.current_state() {
            state.on_resume(engine_state, gs);
        }
    }

    fn replace_state(
        &mut self,
        engine_state: &mut Engine_State,
        gs: &mut Gameplay_System,
        mut state: Box<dyn Game_State>,
    ) {
        if let Some(mut prev_state) = self.state_stack.pop() {
            prev_state.on_end(engine_state, gs);
        }

        state.on_start(engine_state, gs);
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
        fn on_start(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().started = true;
        }
        fn on_end(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().ended = true;
        }
        fn on_pause(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().paused = true;
        }
        fn on_resume(&mut self, _state: &mut Engine_State, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().resumed = true;
        }
        fn update(
            &mut self,
            _state: &mut Engine_State,
            _gs: &mut Gameplay_System,
        ) -> State_Transition {
            self.data.borrow_mut().updated += 1;
            if self.data.borrow().updated < 2 {
                State_Transition::None
            } else {
                State_Transition::Pop
            }
        }
        fn handle_actions(
            &mut self,
            _actions: &[Game_Action],
            _state: &mut Engine_State,
            _gs: &mut Gameplay_System,
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
        let env = ecs_engine::core::env::Env_Info::gather().unwrap();
        let config = ecs_engine::cfg::Config::new_from_dir(&env.cfg_root);
        let mut engine_state = ecs_engine::core::app::create_engine_state(
            env,
            config,
            ecs_engine::core::app_config::App_Config {
                title: String::from(""),
                target_win_size: (0, 0),
                in_replay_file: None,
            },
        );
        let mut gs = Gameplay_System::new();
        let mut smgr = State_Manager::new();
        smgr.push_state(state, &mut engine_state, &mut gs);

        assert!(data.borrow().started, "State was not started");
        assert!(!data.borrow().ended, "State was ended");
        assert!(!data.borrow().resumed, "State was resumed");
        assert!(!data.borrow().paused, "State was paused");
        assert_eq!(data.borrow().updated, 0);
        assert_eq!(data.borrow().handled_actions, 0);

        smgr.update(&mut engine_state, &mut gs);
        assert_eq!(data.borrow().updated, 1);
        assert_eq!(data.borrow().handled_actions, 0);

        let actions = [];
        smgr.handle_actions(&actions, &mut engine_state, &mut gs);
        assert_eq!(data.borrow().handled_actions, 1);

        smgr.update(&mut engine_state, &mut gs); // this pops the state
        assert_eq!(data.borrow().updated, 2, "State was not updated");
        assert!(data.borrow().ended, "State was not ended");

        smgr.handle_actions(&actions, &mut engine_state, &mut gs);
        assert_eq!(
            data.borrow().handled_actions,
            1,
            "State was handled but should have been popped"
        );

        smgr.update(&mut engine_state, &mut gs);
        assert_eq!(
            data.borrow().updated,
            2,
            "State was updated but should have been popped"
        );
    }
}
