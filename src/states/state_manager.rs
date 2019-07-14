use super::state::{Game_State, Persistent_Game_State, State_Transition};
use crate::cfg;
use crate::core::input::Action_List;
use crate::core::msg;
use std::time::Duration;
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

    pub fn with_initial_state(state: Box<dyn Game_State>) -> State_Manager {
        State_Manager {
            state_stack: vec![state],
            persistent_states: vec![],
        }
    }

    pub fn update(&mut self, dt: &Duration) {
        for state in &mut self.persistent_states {
            state.update(dt);
        }

        if let Some(state) = self.current_state() {
            match state.update(dt) {
                State_Transition::None => {}
                State_Transition::Push(new_state) => self.push_state(new_state),
                State_Transition::Replace(new_state) => self.replace_state(new_state),
                State_Transition::Pop => self.pop_state(),
            }
        }
    }

    /// Returns true if should quit
    pub fn handle_actions(
        &mut self,
        actions: &Action_List,
        dispatcher: &msg::Msg_Dispatcher,
        config: &cfg::Config,
    ) -> bool {
        let mut should_quit = false;

        if let Some(state) = self.current_state() {
            should_quit |= state.handle_actions(actions, dispatcher, config);
        }

        for state in &mut self.persistent_states {
            should_quit |= state.handle_actions(actions, dispatcher, config);
        }

        should_quit
    }

    pub fn add_persistent_state(&mut self, mut state: Box<dyn Persistent_Game_State>) {
        state.on_start();
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

    fn push_state(&mut self, mut state: Box<dyn Game_State>) {
        self.current_state().map(|s| s.on_pause());
        state.on_start();
        self.state_stack.push(state);
    }

    fn pop_state(&mut self) {
        if let Some(mut prev_state) = self.state_stack.pop() {
            prev_state.on_end();
        } else {
            eprintln!("[ ERROR ] Tried to pop state, but state stack is empty!");
        }

        if let Some(state) = self.current_state() {
            state.on_resume();
        }
    }

    fn replace_state(&mut self, mut state: Box<dyn Game_State>) {
        if let Some(mut prev_state) = self.state_stack.pop() {
            prev_state.on_end();
        }

        state.on_start();
        self.state_stack.push(state);
    }
}
