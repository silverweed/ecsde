/// These actions are directly handled by the engine, without external configuration from data
#[non_exhaustive]
#[derive(Debug)]
pub enum Core_Action {
    Quit,
    Resize(u32, u32),
    Focus_Lost,
}
