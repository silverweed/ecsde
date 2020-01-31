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

macro_rules! impl_cfg_value {
    ($type: ty => $val: ident) => {
        impl From<$type> for Cfg_Value {
            fn from(v: $type) -> Cfg_Value {
                Cfg_Value::$val(v)
            }
        }

        impl TryFrom<Cfg_Value> for $type {
            type Error = ();

            fn try_from(v: Cfg_Value) -> Result<Self, Self::Error> {
                if let Cfg_Value::$val(b) = v {
                    Ok(b)
                } else {
                    Err(())
                }
            }
        }
    };
}

impl_cfg_value!(bool => Bool);
impl_cfg_value!(u32 => UInt);
impl_cfg_value!(i32 => Int);
impl_cfg_value!(f32 => Float);
impl_cfg_value!(String => String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn val_to_cfg_value() {
        assert_eq!(Cfg_Value::from(true), Cfg_Value::Bool(true));
        assert_eq!(Cfg_Value::from(2), Cfg_Value::Int(2));
        assert_eq!(Cfg_Value::from(2u32), Cfg_Value::UInt(2));
        assert_eq!(Cfg_Value::from(2.0), Cfg_Value::Float(2.0));
        assert_eq!(
            Cfg_Value::from("2".to_string()),
            Cfg_Value::String("2".to_string())
        );
    }

    #[test]
    fn cfg_value_to_val() {
        assert_eq!(bool::try_from(Cfg_Value::Bool(true)), Ok(true));
        assert_eq!(i32::try_from(Cfg_Value::Int(2)), Ok(2));
        assert_eq!(u32::try_from(Cfg_Value::UInt(2)), Ok(2u32));
        assert_eq!(f32::try_from(Cfg_Value::Float(2.0)), Ok(2.0));
        assert_eq!(
            String::try_from(Cfg_Value::String("2".to_string())),
            Ok("2".to_string())
        );

        assert_eq!(i32::try_from(Cfg_Value::UInt(2)), Err(()));
        assert_eq!(String::try_from(Cfg_Value::Int(2)), Err(()));
        assert_eq!(u32::try_from(Cfg_Value::Bool(false)), Err(()));
    }
}
