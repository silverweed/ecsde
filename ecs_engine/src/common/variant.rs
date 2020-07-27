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
