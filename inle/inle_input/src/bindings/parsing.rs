use super::{Input_Action, Input_Action_Modifiers, Input_Action_Simple};
use crate::bindings::{Axis_Bindings, Axis_Emulation_Type};
use crate::joystick;
use crate::keyboard;
use crate::mouse;
use inle_common::stringid::String_Id;
use smallvec::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::Vec;

pub(super) fn parse_action_bindings_file(
    path: &Path,
) -> Result<HashMap<Input_Action, Vec<String_Id>>, String> {
    let file = File::open(path).map_err(|_| format!("Failed to open file {:?}!", path))?;
    let lines = BufReader::new(file).lines().map_while(Result::ok);
    lok!("Parsed action bindings file {:?}", path);
    Ok(parse_action_bindings_lines(lines))
}

pub(super) fn parse_axis_bindings_file(path: &Path) -> Result<Axis_Bindings, String> {
    // @Cutnpaste from above
    let file = File::open(path).map_err(|_| format!("Failed to open file {:?}!", path))?;
    let lines = BufReader::new(file).lines().map_while(Result::ok);
    lok!("Parsed axis bindings file {:?}", path);
    Ok(parse_axis_bindings_lines(lines))
}

fn strip_comments(
    lines: impl std::iter::Iterator<Item = String>,
) -> impl std::iter::Iterator<Item = String> {
    lines.map(|mut line| {
        if let Some(comment_start) = line.find(COMMENT_START) {
            line.truncate(comment_start);
        }
        line
    })
}

const COMMENT_START: char = '#';

/// File format:
/// -------------
/// # this is a comment
/// action_name: Key1, Key2, ... # whitespace doesn't matter
///
/// An action can be of the form:
///     MOD1+MOD2+...+Key
/// e.g.
///     CTRL+SHIFT+A
/// Allowed modifiers are: CTRL, LCTRL, RCTRL, etc (see code). They're case insensitive.
///
/// An action can also be of the form:
///     Key:N
/// where N is a number [1, 8] referring to a specific joystick id. This is only valid for joystick
/// buttons and filters which joysticks are considered for that specific button.
// @Cutnpaste from cfg/parsing.rs
fn parse_action_bindings_lines(
    lines: impl std::iter::Iterator<Item = String>,
) -> HashMap<Input_Action, Vec<String_Id>> {
    let mut bindings: HashMap<Input_Action, Vec<String_Id>> = HashMap::new();

    let lines = strip_comments(lines);

    for (lineno, line) in lines.enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let (action_name, action_values_raw) = {
            let tokens: Vec<_> = line.splitn(2, ':').map(str::trim).collect();
            if tokens.len() != 2 {
                lwarn!(
                    "Invalid line {} while parsing action bindings: '{}'.",
                    lineno,
                    line
                );
                continue;
            } else {
                (tokens[0], tokens[1])
            }
        };

        let mut keys: Vec<Input_Action> = action_values_raw
            .split(',')
            .flat_map(|tok| parse_action(tok.trim()))
            .collect();
        keys.sort_unstable();
        keys.dedup();
        for key in keys.into_iter() {
            lverbose!("Parsed input action {} -> {:?}", action_name, key);
            let action_id = String_Id::from(action_name);
            bindings
                .entry(key)
                .or_insert_with(|| Vec::with_capacity(8))
                .push(action_id);
        }
    }

    bindings
}

// Converts a string like "ctrl + Num2" into one or more Input_Actions.
// If the modifier is one that stands as one of multiple possible modifiers
// (e.g. 'ctrl' stands for 'lctrl or rctrl'), one action per possible modifier
// is returned. If several modifiers are added (like 'ctrl + shift'), their
// cartesian product is returned.
fn parse_action(mods_and_key: &str) -> SmallVec<[Input_Action; 2]> {
    let mods_and_key_split = mods_and_key.split('+').map(str::trim).collect::<Vec<_>>();
    let key_raw = mods_and_key_split[mods_and_key_split.len() - 1];
    if let Some(key) = parse_action_simple(key_raw) {
        let mut modifiers = vec![];
        for modif in &mods_and_key_split[..mods_and_key_split.len() - 1] {
            // Note: certain modifier keys count as "either X or Y", so they produce
            // multiple results.
            let ms = parse_modifier(modif);
            modifiers.push(ms);
        }
        if modifiers.is_empty() {
            smallvec![Input_Action::new(key)]
        } else {
            // Extract all the possible combinations of modifiers
            let mut merged_modifiers = vec![];
            let mut cursors = std::iter::repeat(0usize)
                .take(modifiers.len())
                .collect::<Vec<_>>();
            loop {
                let mut modif = 0;
                for (i, &curs) in cursors.iter().enumerate() {
                    modif |= modifiers[i][curs];
                }
                merged_modifiers.push(modif);
                let mut curs_idx = 0;
                let mut all_maxed = true;
                while curs_idx < cursors.len() {
                    if cursors[curs_idx] == modifiers[curs_idx].len() - 1 {
                        curs_idx += 1;
                    } else {
                        all_maxed = false;
                        for curs in cursors.iter_mut().take(curs_idx) {
                            *curs = 0;
                        }
                        cursors[curs_idx] += 1;
                        break;
                    }
                }
                if all_maxed {
                    break;
                }
            }
            merged_modifiers
                .into_iter()
                .map(|m| Input_Action::new_with_modifiers(key, m))
                .collect()
        }
    } else {
        smallvec![]
    }
}

fn parse_action_simple(s: &str) -> Option<Input_Action_Simple> {
    if let Some(strip) = s.strip_prefix("Joy_") {
        let tokens = strip.splitn(2, ':').collect::<Vec<_>>();
        if tokens.len() == 2 {
            // Has the form Button:N
            if let Ok(id) = tokens[1].parse::<u32>() {
                if id > 0 && id <= joystick::JOY_COUNT as _ {
                    let btn = joystick::string_to_joy_btn(tokens[0])?;
                    dbg!((btn, id));
                    return Some(Input_Action_Simple::Joystick(btn, Some(id - 1)));
                } else {
                    lerr!(
                        "Joystick id {} is invalid (must be between 1 and {})",
                        id,
                        joystick::JOY_COUNT
                    );
                }
            }
            lerr!("Failed to parse axis {}", s);
            None
        } else {
            let btn = joystick::string_to_joy_btn(strip)?;
            Some(Input_Action_Simple::Joystick(btn, None))
        }
    } else if let Some(strip) = s.strip_prefix("Mouse_") {
        mouse::string_to_mouse_btn(strip).map(Input_Action_Simple::Mouse)
    } else if s == "Wheel_Up" {
        Some(Input_Action_Simple::Mouse_Wheel { up: true })
    } else if s == "Wheel_Down" {
        Some(Input_Action_Simple::Mouse_Wheel { up: false })
    } else {
        keyboard::string_to_key(s).map(Input_Action_Simple::Key)
    }
}

fn parse_modifier(s: &str) -> SmallVec<[Input_Action_Modifiers; 2]> {
    use super::modifiers::*;

    match s.to_lowercase().as_str() {
        "ctrl" => smallvec![MOD_LCTRL, MOD_RCTRL],
        "lctrl" => smallvec![MOD_LCTRL],
        "rctrl" => smallvec![MOD_RCTRL],
        "shift" => smallvec![MOD_LSHIFT, MOD_RSHIFT],
        "lshift" => smallvec![MOD_LSHIFT],
        "rshift" => smallvec![MOD_RSHIFT],
        "alt" => smallvec![MOD_LALT, MOD_RALT],
        "lalt" => smallvec![MOD_LALT],
        "altgr" => smallvec![MOD_RALT],
        "super" => smallvec![MOD_LSUPER, MOD_RSUPER],
        "lsuper" => smallvec![MOD_LSUPER],
        "rsuper" => smallvec![MOD_RSUPER],
        _ => smallvec![],
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash)]
enum Virtual_Axis_Mapping {
    // If Joystick_Id is Some, this mapping is only valid for that joystick.
    Axis(joystick::Joystick_Axis, Option<joystick::Joystick_Id>),
    Action_Emulate_Min(Input_Action),
    Action_Emulate_Max(Input_Action),
}

/// File format:
/// -------------
/// # this is a comment
/// axis_name: Axis1, +Key1, -Key2, ... # whitespace doesn't matter
///
/// # note that +Key1 means that Key1 yields the max value for that axis, and -Key2
/// # means Key2 yields the min value. Axis1 is a real axis mapped analogically to the virtual
/// axis.
///
/// If you want only specific joysticks to map to a specific axis, you can use the syntax
///     axis_name: Axis1:N
/// where N is a number from 1 to 8 inclusive. This means that only the joystick with id N will be
/// considered as a binding for axis_name.
// @Cutnpaste from cfg/parsing.rs
fn parse_axis_bindings_lines(lines: impl std::iter::Iterator<Item = String>) -> Axis_Bindings {
    let mut bindings = Axis_Bindings {
        real: std::default::Default::default(),
        emulated: HashMap::new(),
        axes_names: vec![],
    };

    let lines = strip_comments(lines);

    for (lineno, line) in lines.enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let (axis_name, axis_values_raw) = {
            let tokens: Vec<_> = line.splitn(2, ':').map(str::trim).collect();
            if tokens.len() != 2 {
                lwarn!(
                    "Invalid line {} while parsing axis bindings: '{}'.",
                    lineno,
                    line
                );
                continue;
            } else {
                (tokens[0], tokens[1])
            }
        };

        let mut keys: Vec<Virtual_Axis_Mapping> = axis_values_raw
            .split(',')
            .filter_map(|tok| parse_axis(tok.trim()))
            .collect();
        keys.sort_unstable();
        keys.dedup();
        for key in keys.into_iter() {
            let axis_id = String_Id::from(axis_name);
            bindings.axes_names.push(axis_id);
            match key {
                Virtual_Axis_Mapping::Axis(axis, joy_id) => {
                    // convert joy_id to a mask (None means all joysticks)
                    let joy_mask = if let Some(id) = joy_id { 1 << id } else { !0 };
                    bindings.real[axis as usize].push((axis_id, joy_mask));
                }
                Virtual_Axis_Mapping::Action_Emulate_Min(action) => {
                    bindings
                        .emulated
                        .entry(action)
                        .or_insert_with(|| Vec::with_capacity(8))
                        .push((axis_id, Axis_Emulation_Type::Min));
                }
                Virtual_Axis_Mapping::Action_Emulate_Max(action) => {
                    bindings
                        .emulated
                        .entry(action)
                        .or_insert_with(|| Vec::with_capacity(8))
                        .push((axis_id, Axis_Emulation_Type::Max));
                }
            }
        }
    }

    bindings.axes_names.sort_unstable();
    bindings.axes_names.dedup();

    bindings
}

fn parse_axis(s: &str) -> Option<Virtual_Axis_Mapping> {
    match s.chars().next() {
        Some('+') => Some(Virtual_Axis_Mapping::Action_Emulate_Max(
            *parse_action(&s[1..]).get(0)?,
        )),
        Some('-') => Some(Virtual_Axis_Mapping::Action_Emulate_Min(
            *parse_action(&s[1..]).get(0)?,
        )),
        _ => {
            let tokens: Vec<_> = s.splitn(2, ':').collect();
            if tokens.len() == 2 {
                // Axis name has the form `Axis_Name:N`, where N is a joystick id
                if let Ok(id) = tokens[1].parse::<u32>() {
                    if id > 0 && id <= joystick::JOY_COUNT as _ {
                        // NOTE: remapping the id from [1,JOY_COUNT] to [0,JOY_COUNT-1].
                        // We want the user-facing cfg value to start from 1, but internally the
                        // joystick ids start from 0.
                        let axis = joystick::string_to_joy_axis(tokens[0])?;
                        return Some(Virtual_Axis_Mapping::Axis(axis, Some(id - 1)));
                    } else {
                        lerr!(
                            "Joystick id {} is invalid (must be between 1 and {})",
                            id,
                            joystick::JOY_COUNT
                        );
                    }
                }
                lerr!("Failed to parse axis {}", s);
                None
            } else {
                Some(Virtual_Axis_Mapping::Axis(
                    joystick::string_to_joy_axis(s)?,
                    None,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::modifiers::*;
    use super::joystick::Joystick_Button;
    use super::keyboard::Key;
    use super::mouse::Mouse_Button;
    use super::*;

    macro_rules! actionvec {
        ($($x: expr)?) => {{
            let v: SmallVec<[Input_Action; 2]> = smallvec![$($x)?];
            v
        }};
    }

    #[test]
    fn test_parse_action() {
        assert_eq!(
            parse_action("Space"),
            actionvec!(Input_Action::new(Input_Action_Simple::Key(Key::Space)))
        );
        assert_eq!(parse_action("Spacex"), actionvec![]);
        assert_eq!(
            parse_action("Dash"),
            actionvec!(Input_Action::new(Input_Action_Simple::Key(Key::Dash)))
        );
        assert_eq!(
            parse_action("  Dash  "),
            actionvec!(Input_Action::new(Input_Action_Simple::Key(Key::Dash)))
        );
        assert_eq!(parse_action("Joy_"), actionvec![]);
        assert_eq!(
            parse_action("Joy_Shoulder_Right"),
            actionvec!(Input_Action::new(Input_Action_Simple::Joystick(
                Joystick_Button::Shoulder_Right,
                None
            )))
        );
        assert_eq!(
            parse_action("Joy_Shoulder_Right:1"),
            actionvec!(Input_Action::new(Input_Action_Simple::Joystick(
                Joystick_Button::Shoulder_Right,
                Some(0)
            )))
        );
        assert_eq!(parse_action("Joy_Shoulder_Right:10"), actionvec!());
        assert_eq!(parse_action("Mouse_"), actionvec![]);
        assert_eq!(
            parse_action("Mouse_Left"),
            actionvec!(Input_Action::new(Input_Action_Simple::Mouse(
                Mouse_Button::Left
            )))
        );
        assert_eq!(parse_action(""), actionvec![]);
    }

    #[test]
    fn test_parse_axis() {
        use joystick::Joystick_Axis as J;
        use Input_Action as I;
        use Input_Action_Simple as IS;
        use Virtual_Axis_Mapping as V;

        assert_eq!(
            parse_axis("Stick_Left_H"),
            Some(V::Axis(J::Stick_Left_H, None))
        );
        assert_eq!(
            parse_axis("Stick_Left_H:2"),
            Some(V::Axis(J::Stick_Left_H, Some(1)))
        );
        assert_eq!(parse_axis("Stick_Left_H:0"), None);
        assert_eq!(parse_axis("Stick_Left_H:1i"), None);
        assert_eq!(parse_axis("Stick_Left"), None);
        assert_eq!(
            parse_axis("+Joy_Stick_Left"),
            Some(V::Action_Emulate_Max(I::new(IS::Joystick(
                Joystick_Button::Stick_Left,
                None
            ))))
        );
        assert_eq!(parse_axis("+Stick_Left_H"), None);
        assert_eq!(
            parse_axis("-A"),
            Some(V::Action_Emulate_Min(I::new(IS::Key(Key::A))))
        );
        assert_eq!(parse_axis(""), None);
        assert_eq!(parse_axis("+"), None);
        assert_eq!(
            parse_axis("-Mouse_Right"),
            Some(V::Action_Emulate_Min(I::new(IS::Mouse(
                Mouse_Button::Right
            ))))
        );
    }

    #[test]
    fn test_parse_action_bindings_lines() {
        let lines: Vec<String> = vec![
            "# This is a sample file",
            "action1: Num0",
            "action2: Num1,ctrl+Num2#This is an action",
            "   action3   :   ctrl+SHIFT +  alt +Num3,",
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
            "action11: J, Joy_Stick_Right:1, Mouse_Middle",
        ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
        let parsed = parse_action_bindings_lines(lines.into_iter());

        assert_eq!(parsed.len(), 23);
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Key(Key::Num0))],
            vec![sid!("action1"), sid!("action6")]
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Key(Key::Num1))],
            vec![sid!("action2"), sid!("action9")]
        );
        assert_eq!(
            parsed
                [&Input_Action::new_with_modifiers(Input_Action_Simple::Key(Key::Num2), MOD_LCTRL)],
            vec![sid!("action2")]
        );
        assert_eq!(
            parsed[&Input_Action::new_with_modifiers(
                Input_Action_Simple::Key(Key::Num3),
                MOD_RSHIFT | MOD_LALT | MOD_RCTRL
            )],
            vec![sid!("action3")]
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Key(Key::Num4))],
            vec![sid!("action5")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Key(Key::Num5))],
            vec![sid!("action5")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Key(Key::Num6))],
            vec![sid!("action5")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Mouse(Mouse_Button::Left))],
            vec![sid!("action8")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Mouse(Mouse_Button::Right))],
            vec![sid!("action8")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Joystick(
                Joystick_Button::Face_Bottom,
                None
            ))],
            vec![sid!("action10")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Joystick(
                Joystick_Button::Special_Left,
                None
            ))],
            vec![sid!("action10")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Key(Key::J))],
            vec![sid!("action11")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Joystick(
                Joystick_Button::Stick_Right,
                Some(0)
            ))],
            vec![sid!("action11")],
        );
        assert_eq!(
            parsed[&Input_Action::new(Input_Action_Simple::Mouse(Mouse_Button::Middle))],
            vec![sid!("action11")],
        );
    }

    #[test]
    fn test_parse_axis_bindings_lines() {
        let lines: Vec<String> = vec![
            "# This is a sample file",
            "axis1: Stick_Right_V, Stick_Left_H:1",
            "axis2: ,+D#This is an axis",
            "   axis3   :   Trigger_Right,+Joy_Stick_Right:2,Stick_Left_H,",
            " axis4:",
            "",
            "##############",
            "axis5:-D",
        ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
        let Axis_Bindings {
            real,
            emulated,
            axes_names,
        } = parse_axis_bindings_lines(lines.into_iter());

        use joystick::Joystick_Axis as J;

        assert_eq!(emulated.len(), 2);
        assert_eq!(axes_names.len(), 4);
        assert_eq!(real[J::Stick_Right_V as usize], vec![(sid!("axis1"), !0)]);
        assert_eq!(
            real[J::Stick_Left_H as usize],
            vec![(sid!("axis1"), 1), (sid!("axis3"), !0)]
        );
        assert_eq!(
            emulated[&Input_Action::new(Input_Action_Simple::Key(Key::D))],
            vec![
                (sid!("axis2"), Axis_Emulation_Type::Max),
                (sid!("axis5"), Axis_Emulation_Type::Min)
            ]
        );
        assert_eq!(
            emulated[&Input_Action::new(Input_Action_Simple::Joystick(
                Joystick_Button::Stick_Right,
                Some(1)
            ))],
            vec![(sid!("axis3"), Axis_Emulation_Type::Max)]
        );
        assert_eq!(real[J::Trigger_Right as usize], vec![(sid!("axis3"), !0)]);
    }
}
