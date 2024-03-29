use super::value::Cfg_Value;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::vec::Vec;

const HEADER_SEPARATOR: char = '/';
const COMMENT_START: char = '#';

/// Contains all configurations from all cfg files.
/// Conceptually, it's as all cfg sections were in the same file: they're just split
/// into multiple files for convenience.
// @Convenience: this means all headers must be unique across files; maybe splitting
// files logically may become convenient in the long run...we'll see.
pub(super) struct Raw_Config {
    pub sections: Vec<Cfg_Section>,
}

/// A Cfg_Section is a section in a cfg file delimited by /header and
/// consisting of multiple lines of the format:
/// [#] key [value] [# ...]
#[derive(Debug)]
pub(super) struct Cfg_Section {
    pub header: String,
    pub entries: Vec<Cfg_Entry>,
}

#[derive(Debug, Clone)]
pub(super) struct Cfg_Entry {
    pub key: String,
    pub value: Cfg_Value,
}

impl Raw_Config {
    pub fn empty() -> Raw_Config {
        Raw_Config { sections: vec![] }
    }

    pub fn new_from_dir(dir_path: &Path) -> Raw_Config {
        if let Ok(sections_list) = parse_config_dir(dir_path) {
            let mut sections = vec![];
            for section in sections_list.into_iter() {
                sections.push(section);
            }
            Raw_Config { sections }
        } else {
            Raw_Config::empty()
        }
    }
}

fn parse_config_dir(dir_path: &Path) -> Result<Vec<Cfg_Section>, std::io::Error> {
    if dir_path.is_dir() {
        let mut sections = vec![];
        let mut n_parsed = 0;
        for entry in fs::read_dir(dir_path)? {
            match entry {
                Ok(e) if e.path().extension() == Some(OsStr::new("cfg")) => {
                    n_parsed += 1;
                    sections.append(&mut parse_config_file(&e.path())?)
                }
                Err(msg) => eprintln!("{}", msg),
                _ => (),
            }
        }
        lok!("Parsed {} cfg files.", n_parsed);
        Ok(sections)
    } else {
        eprintln!(
            "Notice: path {:?} given to parse_config_dir is a single file.",
            dir_path
        );
        parse_config_file(dir_path)
    }
}

pub(super) fn parse_config_file(path: &Path) -> Result<Vec<Cfg_Section>, std::io::Error> {
    let file = File::open(path)?;
    let lines = BufReader::new(file).lines().filter_map(|l| l.ok());
    Ok(parse_lines(lines))
}

// @Speed: this function can likely be optimized quite a lot.
fn parse_lines(lines: impl std::iter::Iterator<Item = String>) -> Vec<Cfg_Section> {
    let mut sections = vec![];
    let mut cur_section = Cfg_Section {
        header: String::from(""),
        entries: vec![],
    };

    // Strip comments
    let lines = lines.map(|mut line| {
        if let Some(comment_start) = line.find(COMMENT_START) {
            line.truncate(comment_start);
        }
        line
    });

    for line in lines {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let first_char = line.chars().next().unwrap();
        if first_char == HEADER_SEPARATOR {
            if !cur_section.header.is_empty() {
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
                _ => unreachable!(),
            };
            let entry = Cfg_Entry {
                key: String::from(key),
                value: parse_value(val.trim_start()),
            };
            cur_section.entries.push(entry);
        }
    }
    if !cur_section.header.is_empty() {
        sections.push(cur_section);
    }

    sections
}

fn parse_value(raw: &str) -> Cfg_Value {
    Cfg_Value::from(raw)
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
        let parsed = parse_lines(lines.into_iter());

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
