use super::phase::{Game_Phase, Persistent_Game_Phase, Phase_Id, Phase_Transition};
use inle_common::stringid::String_Id;
use inle_input::input_state::Game_Action;

/// Manages a PDA of Game_Phases.
pub struct Phase_Manager<Phase_Args> {
    /// Only the topmost phase is updated and queried for actions at any time.
    phase_stack: Vec<Phase_Id>,

    /// Persistent phases are always updated and their handle_actions is always called.
    /// Update is called for every persistent phase *before* the regular phases, whereas
    /// handle_actions is called after.
    persistent_phases: Vec<(Phase_Id, Box<dyn Persistent_Game_Phase<Args = Phase_Args>>)>,

    phases: Vec<(Phase_Id, Box<dyn Game_Phase<Args = Phase_Args>>)>,
}

impl<Phase_Args> Default for Phase_Manager<Phase_Args> {
    fn default() -> Self {
        Self {
            phase_stack: vec![],
            persistent_phases: vec![],
            phases: vec![],
        }
    }
}

impl<Phase_Args> Phase_Manager<Phase_Args> {
    // Returns true if the game should quit
    #[must_use]
    pub fn update(&mut self, args: &mut Phase_Args) -> bool {
        for (_, phase) in &mut self.persistent_phases {
            phase.update(args);
        }

        if let Some(phase) = self.current_phase() {
            match phase.update(args) {
                Phase_Transition::None => {}
                Phase_Transition::Push(new_phase) => self.push_phase(new_phase, args),
                Phase_Transition::Replace(new_phase) => self.replace_phase(new_phase, args),
                Phase_Transition::Pop => self.pop_phase(args),
                Phase_Transition::Flush_All_And_Replace(new_phase) => {
                    self.flush_all_and_replace(new_phase, args)
                }
                Phase_Transition::Quit_Game => {
                    return true;
                }
            }
        }
        false
    }

    pub fn handle_actions(&mut self, actions: &[Game_Action], args: &mut Phase_Args) {
        if let Some(phase) = self.current_phase() {
            phase.handle_actions(actions, args);
        }

        for (_, phase) in &mut self.persistent_phases {
            phase.handle_actions(actions, args);
        }
    }

    pub fn add_persistent_phase(
        &mut self,
        phase_id: Phase_Id,
        mut phase: Box<dyn Persistent_Game_Phase<Args = Phase_Args>>,
        args: &mut Phase_Args,
    ) {
        phase.on_start(args);
        self.persistent_phases.push((phase_id, phase));
    }

    #[inline]
    fn current_phase(&mut self) -> Option<&mut dyn Game_Phase<Args = Phase_Args>> {
        let len = self.phase_stack.len();
        if len > 0 {
            Some(&mut **self.get_phase(self.phase_stack[len - 1]))
        } else {
            None
        }
    }

    pub fn register_phase(
        &mut self,
        phase_id: String_Id,
        phase: Box<dyn Game_Phase<Args = Phase_Args>>,
    ) {
        self.phases.push((Phase_Id::from(phase_id), phase));
    }

    pub fn push_phase<P: Into<Phase_Id>>(&mut self, phase_id: P, args: &mut Phase_Args) {
        if let Some(s) = self.current_phase() {
            s.on_pause(args);
        }
        let phase_id = phase_id.into();
        self.get_phase(phase_id).on_start(args);
        self.phase_stack.push(phase_id);
    }

    pub fn register_and_push_phase(
        &mut self,
        phase_id: String_Id,
        phase: Box<dyn Game_Phase<Args = Phase_Args>>,
        args: &mut Phase_Args
    ) {
        self.register_phase(phase_id, phase);
        self.push_phase(phase_id, args);
    }

    fn get_phase(&mut self, phase_id: Phase_Id) -> &mut Box<dyn Game_Phase<Args = Phase_Args>> {
        let new_phase_idx = self
            .phases
            .iter()
            .position(|(id, _)| *id == phase_id)
            .unwrap();
        &mut self.phases[new_phase_idx].1
    }

    fn pop_phase(&mut self, args: &mut Phase_Args) {
        if let Some(prev_phase_id) = self.phase_stack.pop() {
            self.get_phase(prev_phase_id).on_end(args);
        } else {
            lerr!("Tried to pop phase, but phase stack is empty!");
        }

        if let Some(phase) = self.current_phase() {
            phase.on_resume(args);
        }
    }

    fn replace_phase(&mut self, phase_id: Phase_Id, args: &mut Phase_Args) {
        if let Some(prev_phase) = self.phase_stack.pop() {
            self.get_phase(prev_phase).on_end(args);
        }

        self.get_phase(phase_id).on_start(args);
        self.phase_stack.push(phase_id);
    }

    fn flush_all_and_replace(&mut self, phase_id: Phase_Id, args: &mut Phase_Args) {
        while let Some(prev_phase) = self.phase_stack.pop() {
            self.get_phase(prev_phase).on_end(args);
        }

        self.get_phase(phase_id).on_start(args);
        self.phase_stack.push(phase_id);
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct Test_Phase_Data {
        pub started: bool,
        pub paused: bool,
        pub resumed: bool,
        pub ended: bool,
        pub updated: i32,
        pub handled_actions: i32,
    }

    #[derive(Default)]
    struct Test_Phase_1 {
        pub data: Rc<RefCell<Test_Phase_Data>>,
    }

    impl Game_Phase<Args=Phase_Args> for Test_Phase_1 {
        fn on_start(&mut self, _phase: &mut Engine_Phase, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().started = true;
        }
        fn on_end(&mut self, _phase: &mut Engine_Phase, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().ended = true;
        }
        fn on_pause(&mut self, _phase: &mut Engine_Phase, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().paused = true;
        }
        fn on_resume(&mut self, _phase: &mut Engine_Phase, _gs: &mut Gameplay_System) {
            self.data.borrow_mut().resumed = true;
        }
        fn update(
            &mut self,
            _phase: &mut Engine_Phase,
            _gs: &mut Gameplay_System,
        ) -> Phase_Transition {
            self.data.borrow_mut().updated += 1;
            if self.data.borrow().updated < 2 {
                Phase_Transition::None
            } else {
                Phase_Transition::Pop
            }
        }
        fn handle_actions(
            &mut self,
            _actions: &[Game_Action],
            _phase: &mut Engine_Phase,
            _gs: &mut Gameplay_System,
        ) -> bool {
            self.data.borrow_mut().handled_actions += 1;
            false
        }
    }

    // @Incomplete: pause/resume and persistent states are not covered
    #[test]
    fn phase_manager() {
        let data = Rc::new(RefCell::new(Test_Phase_Data::default()));
        let phase = Box::new(Test_Phase_1 { data: data.clone() });
        let env = ecs_engine::core::env::Env_Info::gather().unwrap();
        let config = ecs_engine::cfg::Config::new_from_dir(&env.cfg_root);
        let mut engine_phase = ecs_engine::core::app::create_engine_phase(
            env,
            config,
            ecs_engine::core::app_config::App_Config {
                title: String::from(""),
                target_win_size: (0, 0),
                in_replay_file: None,
            },
        )
        .unwrap();
        let mut gs = Gameplay_System::new();
        let mut smgr = Phase_Manager::new();
        smgr.push_phase(phase, &mut engine_phase, &mut gs);

        assert!(data.borrow().started, "Phase was not started");
        assert!(!data.borrow().ended, "Phase was ended");
        assert!(!data.borrow().resumed, "Phase was resumed");
        assert!(!data.borrow().paused, "Phase was paused");
        assert_eq!(data.borrow().updated, 0);
        assert_eq!(data.borrow().handled_actions, 0);

        smgr.update(&mut engine_phase, &mut gs);
        assert_eq!(data.borrow().updated, 1);
        assert_eq!(data.borrow().handled_actions, 0);

        let actions = [];
        smgr.handle_actions(&actions, &mut engine_phase, &mut gs);
        assert_eq!(data.borrow().handled_actions, 1);

        smgr.update(&mut engine_phase, &mut gs); // this pops the phase
        assert_eq!(data.borrow().updated, 2, "Phase was not updated");
        assert!(data.borrow().ended, "Phase was not ended");

        smgr.handle_actions(&actions, &mut engine_phase, &mut gs);
        assert_eq!(
            data.borrow().handled_actions,
            1,
            "Phase was handled but should have been popped"
        );

        smgr.update(&mut engine_phase, &mut gs);
        assert_eq!(
            data.borrow().updated,
            2,
            "Phase was updated but should have been popped"
        );
    }
}
*/
