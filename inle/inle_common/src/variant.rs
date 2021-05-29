use std::convert::{From, TryFrom};

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Variant {
    Nil, // @Redundant: do we really need this value?
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    String(String),
    ILong(i64),
    ULong(u64),
    Double(f64),
}

macro_rules! impl_variant {
    ($type: ty => $val: ident) => {
        impl From<$type> for Variant {
            fn from(v: $type) -> Variant {
                Variant::$val(v)
            }
        }

        impl TryFrom<Variant> for $type {
            type Error = ();

            fn try_from(v: Variant) -> Result<Self, Self::Error> {
                if let Variant::$val(b) = v {
                    Ok(b)
                } else {
                    Err(())
                }
            }
        }
    };
}

impl_variant!(bool => Bool);
impl_variant!(u32 => UInt);
impl_variant!(i32 => Int);
impl_variant!(f32 => Float);
impl_variant!(i64 => ILong);
impl_variant!(u64 => ULong);
impl_variant!(f64 => Double);
impl_variant!(String => String);

impl From<&str> for Variant {
    fn from(raw: &str) -> Self {
        if raw.is_empty() {
            // @Redundant: can this ever happen? Do we need a Nil value at all?
            return Self::Nil;
        }

        // @Speed: this is easy but inefficient! An actual lexer would be faster, but for now this is ok.
        if raw.starts_with("0x") {
            if let Ok(v) = u32::from_str_radix(&raw[2..], 16) {
                Self::UInt(v)
            } else {
                eprintln!("[ NOTICE ] Variant {} parsed as string.", raw);
                Self::String(String::from(raw))
            }
        } else if raw.starts_with(|c| ('0'..='9').contains(&c)) && raw.ends_with('u') {
            if let Ok(v) = u32::from_str_radix(&raw[..raw.len() - 1], 10) {
                Self::UInt(v)
            } else {
                eprintln!("[ NOTICE ] Variant {} parsed as string.", raw);
                Self::String(String::from(raw))
            }
        } else if let Ok(v) = raw.parse::<i32>() {
            Self::Int(v)
        } else if let Ok(v) = raw.parse::<f32>() {
            Self::Float(v)
        } else if let Ok(v) = raw.parse::<bool>() {
            Self::Bool(v)
        } else {
            Self::String(String::from(raw))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn val_to_variant() {
        assert_eq!(Variant::from(true), Variant::Bool(true));
        assert_eq!(Variant::from(2), Variant::Int(2));
        assert_eq!(Variant::from(2u32), Variant::UInt(2));
        assert_eq!(Variant::from(2.0f32), Variant::Float(2.0));
        assert_eq!(Variant::from(2.0), Variant::Double(2.0));
        assert_eq!(
            Variant::from("2".to_string()),
            Variant::String("2".to_string())
        );
        assert_eq!(Variant::from(2i64), Variant::ILong(2));
        assert_eq!(Variant::from(2u64), Variant::ULong(2));
        assert_eq!(Variant::from("22"), Variant::Int(22));
        assert_eq!(Variant::from("0x22"), Variant::UInt(0x22));
        assert_eq!(Variant::from("0xx22"), Variant::String("0xx22".to_string()));
        assert_eq!(Variant::from("22u"), Variant::UInt(22));
        assert_eq!(Variant::from("9u"), Variant::UInt(9));
        assert_eq!(Variant::from("0x22u"), Variant::String("0x22u".to_string()));
        assert_eq!(Variant::from("22uu"), Variant::String("22uu".to_string()));
    }

    #[test]
    fn variant_to_val() {
        assert_eq!(bool::try_from(Variant::Bool(true)), Ok(true));
        assert_eq!(i32::try_from(Variant::Int(2)), Ok(2));
        assert_eq!(u32::try_from(Variant::UInt(2)), Ok(2u32));
        assert_eq!(f32::try_from(Variant::Float(2.0)), Ok(2.0));
        assert_eq!(
            String::try_from(Variant::String("2".to_string())),
            Ok("2".to_string())
        );
        assert_eq!(i64::try_from(Variant::ILong(2)), Ok(2i64));
        assert_eq!(u64::try_from(Variant::ULong(2)), Ok(2u64));

        assert_eq!(i32::try_from(Variant::UInt(2)), Err(()));
        assert_eq!(String::try_from(Variant::Int(2)), Err(()));
        assert_eq!(u32::try_from(Variant::Bool(false)), Err(()));
    }
}
