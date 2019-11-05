#[derive(PartialEq, Hash, Copy, Clone, Debug)]
pub enum Game_Action {
    Quit,
    Resize(u32, u32),
    // Note: the zoom factor is an integer rather than a float as it can be hashed.
    // This integer must be divided by 100 to obtain the actual scaling factor.
    Zoom(i32),
    Change_Speed(i32),
    Pause_Toggle,
    Step_Simulation,
    Print_Entity_Manager_Debug_Info,
}
