use super::joystick;
use super::keymap;
use super::mouse;
use super::Input_Action;
use crate::core::common::stringid::String_Id;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::Vec;

pub(super) fn parse_bindings_file(
    path: &Path,
) -> Result<HashMap<Input_Action, Vec<String_Id>>, String> {
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
) -> HashMap<Input_Action, Vec<String_Id>> {
    let mut bindings: HashMap<Input_Action, Vec<String_Id>> = HashMap::new();

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
        for key in keys.into_iter() {
            let action_id = String_Id::from(action_name);
            match bindings.entry(key) {
                Entry::Occupied(mut o) => o.get_mut().push(action_id),
                Entry::Vacant(v) => {
                    v.insert(vec![action_id]);
                }
            }
        }
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
    use super::joystick::Joystick_Button;
    use super::mouse::Mouse_Button;

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

        assert_eq!(parsed.len(), 14);
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num0)],
            vec![String_Id::from("action1"), String_Id::from("action6")]
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num1)],
            vec![String_Id::from("action2"), String_Id::from("action9")]
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num2)],
            vec![String_Id::from("action2"), String_Id::from("action9")]
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num3)],
            vec![String_Id::from("action3")]
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num4)],
            vec![String_Id::from("action5")],
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num5)],
            vec![String_Id::from("action5")],
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::Num6)],
            vec![String_Id::from("action5")],
        );
        assert_eq!(
            parsed[&Input_Action::Mouse(Mouse_Button::Left)],
            vec![String_Id::from("action8")],
        );
        assert_eq!(
            parsed[&Input_Action::Mouse(Mouse_Button::Right)],
            vec![String_Id::from("action8")],
        );
        assert_eq!(
            parsed[&Input_Action::Joystick(Joystick_Button::Face_Bottom)],
            vec![String_Id::from("action10")],
        );
        assert_eq!(
            parsed[&Input_Action::Joystick(Joystick_Button::Special_Left)],
            vec![String_Id::from("action10")],
        );
        assert_eq!(
            parsed[&Input_Action::Key(Key::J)],
            vec![String_Id::from("action11")],
        );
        assert_eq!(
            parsed[&Input_Action::Joystick(Joystick_Button::Stick_Right)],
            vec![String_Id::from("action11")],
        );
        assert_eq!(
            parsed[&Input_Action::Mouse(Mouse_Button::Middle)],
            vec![String_Id::from("action11")],
        );
    }
}
