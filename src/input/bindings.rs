use crate::core::common::stringid::String_Id;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::Vec;

#[cfg(feature = "use-sfml")]
mod keymap_sfml;
#[cfg(feature = "use-sfml")]
use keymap_sfml as keymap;
#[cfg(feature = "use-sfml")]
use sfml::window::Key;

pub struct Input_Bindings {
    /// { action_name => [keys] }
    action_bindings: HashMap<String_Id, Vec<Key>>,
}

impl Input_Bindings {
    pub fn create_from_config(bindings_file_path: &Path) -> Result<Input_Bindings, String> {
        Ok(Input_Bindings {
            action_bindings: parse_bindings_file(bindings_file_path)?,
        })
    }
}

pub(super) fn parse_bindings_file(path: &Path) -> Result<HashMap<String_Id, Vec<Key>>, String> {
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
) -> HashMap<String_Id, Vec<Key>> {
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
            let tokens: Vec<_> = line.splitn(2, ':').map(|tok| tok.trim()).collect();
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

        let mut keys: Vec<Key> = action_values_raw
            .split(',')
            .filter_map(|tok| keymap::string_to_key(tok.trim()))
            .collect();
        keys.sort_unstable();
        keys.dedup();
        bindings.insert(String_Id::from(action_name), keys);
    }

    bindings
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
        let parsed = parse_bindings_lines(lines.into_iter());

        assert_eq!(parsed.len(), 6);
        assert_eq!(parsed[&String_Id::from("action1")], vec![Key::Num0]);
        assert_eq!(
            parsed[&String_Id::from("action2")],
            vec![Key::Num1, Key::Num2]
        );
        assert_eq!(parsed[&String_Id::from("action3")], vec![Key::Num3]);
        assert_eq!(parsed[&String_Id::from("action4")], vec![]);
        assert_eq!(
            parsed[&String_Id::from("action5")],
            vec![Key::Num4, Key::Num5, Key::Num6]
        );
        assert_eq!(parsed[&String_Id::from("action6")], vec![Key::Num0]);
    }
}
