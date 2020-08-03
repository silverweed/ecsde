use std::convert::From;

pub type Cfg_Value = crate::common::variant::Variant;

impl From<&str> for Cfg_Value {
    fn from(raw: &str) -> Cfg_Value {
        if raw.is_empty() {
            // @Redundant: can this ever happen? Do we need a Nil value at all?
            return Cfg_Value::Nil;
        }

        // @Speed: this is easy but inefficient! An actual lexer would be faster, but for now this is ok.
        if raw.starts_with("0x") {
            if let Ok(v) = u32::from_str_radix(raw.trim_start_matches("0x"), 16) {
                Cfg_Value::UInt(v)
            } else {
                eprintln!("[ NOTICE ] Config {} parsed as string.", raw);
                Cfg_Value::String(String::from(raw))
            }
        } else if let Ok(v) = raw.parse::<i32>() {
            Cfg_Value::Int(v)
        } else if let Ok(v) = raw.parse::<f32>() {
            Cfg_Value::Float(v)
        } else if let Ok(v) = raw.parse::<bool>() {
            Cfg_Value::Bool(v)
        } else {
            Cfg_Value::String(String::from(raw))
        }
    }
}
