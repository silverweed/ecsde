use crate::gameplay_system::Gameplay_System;
use ecs_engine::cfg::Cfg_Value;
use ecs_engine::common::stringid::String_Id;
use ecs_engine::common::vector::Vec2f;
use ecs_engine::core::app::Engine_State;
use std::borrow::Cow;
use std::fmt;

#[non_exhaustive]
pub enum Console_Cmd {
    Quit,
    Move_Camera { to: Vec2f },
    Get_Cfg_Var { name: String },
    Set_Cfg_Var { name: String, value: Cfg_Value },
    Toggle_Cfg_Var { name: String },
}

// @Improve @Convenience: this is ugly! We must manually synch this list with
// the parse_cmd below *and* the enum declaration above!
// We can @WaitForStable until we can do a const match on the enum, but maybe
// there is a better way.
pub const ALL_CMD_STRINGS: [&str; 5] = ["quit", "cam", "var", "toggle", "fps"];

pub fn execute(cmdline: &str, engine_state: &mut Engine_State, gs: &mut Gameplay_System) {
    match parse_cmd(cmdline) {
        Ok(cmd) => execute_command(cmd, engine_state, gs),
        Err(err) => lerr!("Failed to execute command {}: {}", cmdline, err),
    }
}

fn parse_cmd(cmdline: &str) -> Result<Console_Cmd, Console_Error> {
    let tokens = cmdline.split(' ').collect::<Vec<_>>();
    if tokens.is_empty() {
        Err(Console_Error::new("Empty command"))
    } else {
        match tokens.as_slice() {
            ["quit"] => Ok(Console_Cmd::Quit),
            ["cam", x, y] => Ok(Console_Cmd::Move_Camera {
                to: Vec2f::new(x.parse()?, y.parse()?),
            }),
            ["cam"] => Ok(Console_Cmd::Move_Camera {
                to: Vec2f::default(),
            }),
            ["var", name] => Ok(Console_Cmd::Get_Cfg_Var {
                name: (*name).to_string(),
            }),
            ["var", name, value] => Ok(Console_Cmd::Set_Cfg_Var {
                name: (*name).to_string(),
                value: Cfg_Value::from(*value),
            }),
            ["toggle", name] => Ok(Console_Cmd::Toggle_Cfg_Var {
                name: (*name).to_string(),
            }),
            ["fps"] => Ok(Console_Cmd::Toggle_Cfg_Var {
                name: String::from("engine/debug/draw_fps_graph"),
            }),
            _ => Err(Console_Error::new(format!("Unknown command: {}", cmdline))),
        }
    }
}

fn execute_command(cmd: Console_Cmd, engine_state: &mut Engine_State, gs: &mut Gameplay_System) {
    match cmd {
        Console_Cmd::Quit => engine_state.should_close = true,
        Console_Cmd::Move_Camera { to } => gs.move_camera_to(to),
        Console_Cmd::Get_Cfg_Var { name } => {
            linfo!(
                "{} = {:?}",
                name,
                engine_state.config.read_cfg(String_Id::from(name.as_str()))
            );
        }
        Console_Cmd::Set_Cfg_Var { name, value } => {
            linfo!("Setting {} to {:?}", name, value);
            engine_state
                .config
                .write_cfg(String_Id::from(name.as_str()), value);
        }
        Console_Cmd::Toggle_Cfg_Var { name } => {
            linfo!("Toggling {}", name);
            let val = engine_state
                .config
                .read_cfg(String_Id::from(name.as_str()))
                .unwrap_or(&Cfg_Value::Nil)
                .clone();
            if let Cfg_Value::Bool(val) = val {
                engine_state
                    .config
                    .write_cfg(String_Id::from(name.as_str()), Cfg_Value::Bool(!val));
            } else {
                lerr!("Cfg_Var {} is not a bool!", name);
            }
        }
    }
}

#[derive(Debug)]
struct Console_Error {
    msg: Cow<'static, str>,
}

impl Console_Error {
    pub fn new<T>(msg: T) -> Self
    where
        Cow<'static, str>: From<T>,
    {
        Self {
            msg: Cow::from(msg),
        }
    }
}

impl std::error::Error for Console_Error {}

impl fmt::Display for Console_Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::num::ParseFloatError> for Console_Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Self::new(err.to_string())
    }
}
