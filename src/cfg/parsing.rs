use super::{Cfg_Entry, Cfg_Section, Cfg_Value};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::vec::Vec;

const HEADER_SEPARATOR: char = '/';
const COMMENT_START: char = '#';

pub(super) fn parse_config_dir(dir_path: &Path) -> Result<Vec<Cfg_Section>, std::io::Error> {
    if dir_path.is_dir() {
        let mut sections = vec![];
        let mut n_parsed = 0;
        for entry in fs::read_dir(dir_path)? {
            match entry {
                Ok(ref e) if e.path().extension() == Some(OsStr::new("cfg")) => {
                    n_parsed += 1;
                    sections.append(&mut parse_config_file(&e.path())?)
                }
                _ => (),
                Err(msg) => eprintln!("{}", msg),
            }
        }
        eprintln!("Parsed {} cfg files.", n_parsed);
        Ok(sections)
    } else {
        eprintln!(
            "Notice: path {:?} given to parse_config_dir is a single file.",
            dir_path
        );
        parse_config_file(dir_path)
    }
}

// @Speed: this function can likely be optimized quite a lot.
fn parse_config_file(path: &Path) -> Result<Vec<Cfg_Section>, std::io::Error> {
    let file = File::open(path)?;
    let lines = BufReader::new(file).lines().filter_map(|l| Some(l.ok()?));
    Ok(parse_lines(lines, path))
}

fn parse_lines(lines: impl std::iter::Iterator<Item = String>, path: &Path) -> Vec<Cfg_Section> {
    let mut sections = vec![];
    let mut cur_section = Cfg_Section {
        header: String::from(""),
        entries: vec![],
    };

    let lines = lines.map(|mut line| {
        if let Some(comment_start) = line.find(COMMENT_START) {
            line.truncate(comment_start);
        }
        line
    });

    let mut lineno = 0;
    for line in lines {
        let line = line.trim();

        lineno += 1;
        if line.len() == 0 {
            continue;
        }

        let first_char = line.chars().next().unwrap();
        if first_char == HEADER_SEPARATOR {
            if cur_section.header.len() > 0 {
                eprintln!("pushed section {:?}", cur_section);
                sections.push(cur_section);
                cur_section = Cfg_Section {
                    header: String::from(""),
                    entries: vec![],
                };
            }
            cur_section.header = String::from(&line[1..]);
        } else {
            let tokens: Vec<_> = line.splitn(2, ' ').collect();
            let (key, val) = match tokens.len() {
                1 => (tokens[0], ""),
                2 => (tokens[0], tokens[1]),
                _ => {
                    // Should never happen due to splitn(2).
                    eprintln!("Line {} in file {:?} is invalid: `{}`", lineno, path, line);
                    continue;
                }
            };
            let entry = Cfg_Entry {
                key: String::from(key),
                value: parse_value(val.trim_left()),
            };
            cur_section.entries.push(entry);
        }
    }
    if cur_section.header.len() > 0 {
        eprintln!("pushed section {:?}", cur_section);
        sections.push(cur_section);
    }

    sections
}

fn parse_value(raw: &str) -> Cfg_Value {
    if raw.len() == 0 {
        Cfg_Value::Nil
    }
    // @Speed: this is easy but inefficient! An actual lexer would be faster, but for now this is ok.
    else if let Ok(v) = raw.parse::<i32>() {
        Cfg_Value::Int(v)
    } else if let Ok(v) = raw.parse::<f32>() {
        Cfg_Value::Float(v)
    } else if let Ok(v) = raw.parse::<bool>() {
        Cfg_Value::Bool(v)
    } else {
        Cfg_Value::String(String::from(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lines() {
        let lines: Vec<String> = vec![
            "# This is a sample file.",
            " /header  ",
            "  ",
            "entry_int 1",
            "entry_bool   false",
            "  entry_nil",
            "entry_string foo  ",
            "entry_multi_string foo bar   baz #this is a comment#",
            "entry_int_neg -2",
            "",
            "   ###",
            "entry_float_neg -1.2 # comment",
            "/other_header",
            "",
        ]
        .iter()
        .map(|&s| String::from(s))
        .collect();
        let parsed = parse_lines(lines.into_iter(), &std::path::PathBuf::new());

        assert_eq!(parsed.len(), 2);

        let sec1 = &parsed[0];
        assert_eq!(sec1.header, "header");
        assert_eq!(sec1.entries.len(), 7);

        assert_eq!(sec1.entries[0].key, "entry_int");
        assert_eq!(sec1.entries[0].value, Cfg_Value::Int(1));
        assert_eq!(sec1.entries[1].key, "entry_bool");
        assert_eq!(sec1.entries[1].value, Cfg_Value::Bool(false));
        assert_eq!(sec1.entries[2].key, "entry_nil");
        assert_eq!(sec1.entries[2].value, Cfg_Value::Nil);
        assert_eq!(sec1.entries[3].key, "entry_string");
        assert_eq!(
            sec1.entries[3].value,
            Cfg_Value::String(String::from("foo"))
        );
        assert_eq!(sec1.entries[4].key, "entry_multi_string");
        assert_eq!(
            sec1.entries[4].value,
            Cfg_Value::String(String::from("foo bar   baz"))
        );
        assert_eq!(sec1.entries[5].key, "entry_int_neg");
        assert_eq!(sec1.entries[5].value, Cfg_Value::Int(-2));
        assert_eq!(sec1.entries[6].key, "entry_float_neg");
        assert_eq!(sec1.entries[6].value, Cfg_Value::Float(-1.2));

        let sec2 = &parsed[1];
        assert_eq!(sec2.header, "other_header");
        assert_eq!(sec2.entries.len(), 0);
    }
}
