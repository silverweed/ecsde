use crate::gameplay_system::Gameplay_System;
use inle_app::app::Engine_State;
use inle_cfg::Cfg_Value;
use inle_common::colors::{self, Color};
use inle_common::stringid::String_Id;
use inle_math::vector::Vec2f;
use std::borrow::Cow;
use std::fmt;

#[non_exhaustive]
pub enum Console_Cmd {
    Quit,
    Move_Camera { to: Vec2f },
    Get_Cfg_Var { name: String },
    Set_Cfg_Var { name: String, value: Cfg_Value },
    Toggle_Cfg_Var { name: String },
    Trace_Fn { fn_name: String },
    Toggle_Log_Window,
}

// @Improve @Convenience: this is ugly! We must manually synch this list with
// the parse_cmd below *and* the enum declaration above!
// We can @WaitForStable until we can do a const match on the enum, but maybe
// there is a better way.
pub const ALL_CMD_STRINGS: [&str; 7] = ["quit", "cam", "var", "toggle", "fps", "trace", "log"];

// Parses and executes 'cmdline'. May return a string to output to the console.
pub fn execute(
    cmdline: &str,
    engine_state: &mut Engine_State,
    gs: &mut Gameplay_System,
) -> Option<(String, Color)> {
    match parse_cmd(cmdline) {
        Ok(cmd) => execute_command(cmd, engine_state, gs),
        Err(err) => Some((
            format!("Failed to execute command {}: {}", cmdline, err),
            colors::RED,
        )),
    }
}

fn parse_cmd(cmdline: &str) -> Result<Console_Cmd, Console_Error> {
    let tokens = cmdline
        .split(' ')
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>();
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
                name: String::from("engine/debug/graphs/fps"),
            }),
            ["trace", fn_name] => Ok(Console_Cmd::Trace_Fn {
                fn_name: (*fn_name).to_string(),
            }),
            ["trace"] => Ok(Console_Cmd::Trace_Fn {
                fn_name: String::default(),
            }),
            ["log"] => Ok(Console_Cmd::Toggle_Cfg_Var {
                name: String::from("engine/debug/log_window/display"),
            }),
            _ => Err(Console_Error::new(format!("Unknown command: {}", cmdline))),
        }
    }
}

fn execute_command(
    cmd: Console_Cmd,
    engine_state: &mut Engine_State,
    gs: &mut Gameplay_System,
) -> Option<(String, Color)> {
    match cmd {
        Console_Cmd::Quit => {
            engine_state.should_close = true;
            None
        }
        Console_Cmd::Move_Camera { to } => {
            gs.levels
                .foreach_active_level(|level| level.move_camera_to(to));
            None
        }
        Console_Cmd::Get_Cfg_Var { name } => Some((
            format!(
                "{} = {:?}",
                name,
                engine_state.config.read_cfg(String_Id::from(name.as_str()))
            ),
            colors::WHITE,
        )),
        Console_Cmd::Set_Cfg_Var { name, value } => {
            linfo!("Setting {} to {:?}", name, value);
            engine_state
                .config
                .write_cfg(String_Id::from(name.as_str()), value);
            None
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
                None
            } else {
                Some((format!("Cfg_Var {} is not a bool!", name), colors::RED))
            }
        }
        Console_Cmd::Trace_Fn { fn_name } => {
            inle_app::app::set_traced_fn(&mut engine_state.debug_systems, fn_name);
            None
        }
        Console_Cmd::Toggle_Log_Window => {
            let state = engine_state
                .debug_systems
                .debug_ui
                .is_log_window_enabled(sid!("log_window"));
            engine_state
                .debug_systems
                .debug_ui
                .set_log_window_enabled(sid!("log_window"), !state);
            None
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
