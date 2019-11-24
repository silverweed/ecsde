use std::convert::{From, TryFrom};

#[derive(Debug, PartialEq, Clone)]
pub enum Cfg_Value {
    Nil, // @Redundant: do we really need this value?
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    String(String),
}

impl From<bool> for Cfg_Value {
    fn from(v: bool) -> Cfg_Value {
        Cfg_Value::Bool(v)
    }
}

impl From<i32> for Cfg_Value {
    fn from(v: i32) -> Cfg_Value {
        Cfg_Value::Int(v)
    }
}

impl From<u32> for Cfg_Value {
    fn from(v: u32) -> Cfg_Value {
        Cfg_Value::UInt(v)
    }
}

impl From<f32> for Cfg_Value {
    fn from(v: f32) -> Cfg_Value {
        Cfg_Value::Float(v)
    }
}

impl From<String> for Cfg_Value {
    fn from(v: String) -> Cfg_Value {
        Cfg_Value::String(v)
    }
}

impl TryFrom<Cfg_Value> for bool {
    type Error = ();

    fn try_from(v: Cfg_Value) -> Result<Self, Self::Error> {
        if let Cfg_Value::Bool(b) = v {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Cfg_Value> for i32 {
    type Error = ();

    fn try_from(v: Cfg_Value) -> Result<Self, Self::Error> {
        if let Cfg_Value::Int(b) = v {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Cfg_Value> for u32 {
    type Error = ();

    fn try_from(v: Cfg_Value) -> Result<Self, Self::Error> {
        if let Cfg_Value::UInt(b) = v {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Cfg_Value> for f32 {
    type Error = ();

    fn try_from(v: Cfg_Value) -> Result<Self, Self::Error> {
        if let Cfg_Value::Float(b) = v {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Cfg_Value> for String {
    type Error = ();

    fn try_from(v: Cfg_Value) -> Result<Self, Self::Error> {
        if let Cfg_Value::String(b) = v {
            Ok(b)
        } else {
            Err(())
        }
    }
}
