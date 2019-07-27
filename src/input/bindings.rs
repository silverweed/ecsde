use crate::core::common::stringid::String_Id;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::Vec;

mod joystick;
mod keymap;
mod mouse;

use joystick::Joystick_Button;
use mouse::Mouse_Button;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Input_Action {
    Key(keymap::Key),
    Joystick(Joystick_Button),
    Mouse(Mouse_Button),
}

pub struct Input_Bindings {
    /// { action_name => [keys] }
    action_bindings: HashMap<String_Id, Vec<Input_Action>>,
}

impl Input_Bindings {
    pub fn create_from_config(bindings_file_path: &Path) -> Result<Input_Bindings, String> {
        Ok(Input_Bindings {
            action_bindings: parse_bindings_file(bindings_file_path)?,
        })
    }
}

pub(super) fn parse_bindings_file(
    path: &Path,
) -> Result<HashMap<String_Id, Vec<Input_Action>>, String> {
    let file = File::open(path).or_else(|_| Err(format!("Failed to open file {:?}!", path)))?;
    let lines = BufReader::new(file).lines().filter_map(|l| Some(l.ok()?));
    eprintln!("[ OK ] Parsed bindings file {:?}", path);
    Ok(parse_bindings_lines(lines))
}

const COMMENT_START: char = '#';

/// File format:
/// -------------
/// # this is a comment
/// action_name: Key1, Key2, ... # whitespace doesn't matter
///
// @Cutnpaste from cfg/parsing.rs
fn parse_bindings_lines(
    lines: impl std::iter::Iterator<Item = String>,
) -> HashMap<String_Id, Vec<Input_Action>> {
    let mut bindings = HashMap::new();

    // Strip comments
    let lines = lines.map(|mut line| {
        if let Some(comment_start) = line.find(COMMENT_START) {
            line.truncate(comment_start);
        }
        line
    });

    for (lineno, line) in lines.enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let (action_name, action_values_raw) = {
            let tokens: Vec<_> = line.splitn(2, ':').map(str::trim).collect();
            if tokens.len() != 2 {
                eprintln!(
                    "[ WARNING ] Invalid line {} while parsing key bindings: '{}'.",
                    lineno, line
                );
                continue;
            } else {
                (tokens[0], tokens[1])
            }
        };

        let mut keys: Vec<Input_Action> = action_values_raw
            .split(',')
            .filter_map(|tok| parse_action(tok.trim()))
            .collect();
        keys.sort_unstable();
        keys.dedup();
        bindings.insert(String_Id::from(action_name), keys);
    }

    bindings
}

fn parse_action(s: &str) -> Option<Input_Action> {
    if s.starts_with("Joy_") {
        joystick::string_to_joy_btn(&s["Joy_".len()..]).map(Input_Action::Joystick)
    } else if s.starts_with("Mouse_") {
        mouse::string_to_mouse_btn(&s["Mouse_".len()..]).map(Input_Action::Mouse)
    } else {
        keymap::string_to_key(s).map(Input_Action::Key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // @Temporary since enum variants on type aliases are experimental
    #[cfg(feature = "use-sfml")]
    use sfml::window::Key;

    #[test]
    fn test_parse_action() {
        assert_eq!(parse_action("Space"), Some(Input_Action::Key(Key::Space)));
        assert_eq!(parse_action("Spacex"), None);
        assert_eq!(parse_action("Dash"), Some(Input_Action::Key(Key::Dash)));
        assert_eq!(parse_action("  Dash  "), None);
        assert_eq!(parse_action("Joy_"), None);
        assert_eq!(
            parse_action("Joy_Shoulder_Right"),
            Some(Input_Action::Joystick(Joystick_Button::Shoulder_Right))
        );
        assert_eq!(parse_action("Mouse_"), None);
        assert_eq!(
            parse_action("Mouse_Left"),
            Some(Input_Action::Mouse(Mouse_Button::Left))
        );
        assert_eq!(parse_action(""), None);
    }

    #[test]
    fn test_parse_bindings_lines() {
        let lines: Vec<String> = vec![
            "# This is a sample file",
            "action1: Num0",
            "action2: Num1,Num2#This is an action",
            "   action3   :   Num3,",
            " action4:",
            "",
            "##############",
            "action5:Num4,Num5,Num6 # Num7",
            "action6:Num0,Num0,Num0,Num0,   Num0,       Num0",
            "action7: Nummmmmm0",
            "action8: Mouse_Left, Mouse_Right, Mouse_MIDDLE",
            "action9: Num1",
            "action9: Num2",
            "action10: Joy_Face_Bottom, Joy_Special_Left",
            "action11: J, Joy_Stick_Right, Mouse_Middle",
        ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
        let parsed = parse_bindings_lines(lines.into_iter());

        assert_eq!(parsed.len(), 11);
        assert_eq!(
            parsed[&String_Id::from("action1")],
            vec![Input_Action::Key(Key::Num0)]
        );
        assert_eq!(
            parsed[&String_Id::from("action2")],
            vec![Input_Action::Key(Key::Num1), Input_Action::Key(Key::Num2)]
        );
        assert_eq!(
            parsed[&String_Id::from("action3")],
            vec![Input_Action::Key(Key::Num3)]
        );
        assert_eq!(parsed[&String_Id::from("action4")], vec![]);
        assert_eq!(
            parsed[&String_Id::from("action5")],
            vec![
                Input_Action::Key(Key::Num4),
                Input_Action::Key(Key::Num5),
                Input_Action::Key(Key::Num6)
            ]
        );
        assert_eq!(
            parsed[&String_Id::from("action6")],
            vec![Input_Action::Key(Key::Num0)]
        );
        assert_eq!(parsed[&String_Id::from("action7")], vec![]);
        assert_eq!(
            parsed[&String_Id::from("action8")],
            vec![
                Input_Action::Mouse(Mouse_Button::Left),
                Input_Action::Mouse(Mouse_Button::Right)
            ]
        );
        assert_eq!(
            parsed[&String_Id::from("action9")],
            vec![Input_Action::Key(Key::Num2)]
        );
        assert_eq!(
            parsed[&String_Id::from("action10")],
            vec![
                Input_Action::Joystick(Joystick_Button::Face_Bottom),
                Input_Action::Joystick(Joystick_Button::Special_Left)
            ]
        );
        assert_eq!(
            parsed[&String_Id::from("action11")],
            vec![
                Input_Action::Key(Key::J),
                Input_Action::Joystick(Joystick_Button::Stick_Right),
                Input_Action::Mouse(Mouse_Button::Middle)
            ]
        );
    }
}
