use super::Cfg_Value;
use std::cmp::PartialEq;
use std::convert::From;
use std::mem::discriminant;
use std::ops::{Add, Deref, Div, Mul, Sub};

pub trait Cfg_Var_Type {
    type Type;

    fn is_type(v: &Cfg_Value) -> bool;
    fn value(v: &Cfg_Value) -> Self::Type;
}

impl Cfg_Var_Type for bool {
    type Type = bool;

    fn is_type(v: &Cfg_Value) -> bool {
        discriminant(v) == discriminant(&Cfg_Value::Bool(false))
    }

    fn value(v: &Cfg_Value) -> Self::Type {
        if let Cfg_Value::Bool(v) = v {
            *v
        } else {
            panic!(
                "Tried to unwrap value of invalid Cfg_Var_Type {:?} (should have been Bool)!",
                v
            );
        }
    }
}

impl Cfg_Var_Type for i32 {
    type Type = i32;

    fn is_type(v: &Cfg_Value) -> bool {
        discriminant(v) == discriminant(&Cfg_Value::Int(0))
    }

    fn value(v: &Cfg_Value) -> Self::Type {
        if let Cfg_Value::Int(v) = v {
            *v
        } else {
            panic!(
                "Tried to unwrap value of invalid Cfg_Var_Type {:?} (should have been Bool)!",
                v
            );
        }
    }
}

impl Cfg_Var_Type for f32 {
    type Type = f32;

    fn is_type(v: &Cfg_Value) -> bool {
        discriminant(v) == discriminant(&Cfg_Value::Float(0.0))
    }

    fn value(v: &Cfg_Value) -> Self::Type {
        if let Cfg_Value::Float(v) = v {
            *v
        } else {
            panic!(
                "Tried to unwrap value of invalid Cfg_Var_Type {:?} (should have been Bool)!",
                v
            );
        }
    }
}

impl Cfg_Var_Type for String {
    type Type = String;

    fn is_type(v: &Cfg_Value) -> bool {
        discriminant(v) == discriminant(&Cfg_Value::String(String::from("")))
    }

    fn value(v: &Cfg_Value) -> Self::Type {
        if let Cfg_Value::String(v) = v {
            String::from(v.as_str())
        } else {
            panic!(
                "Tried to unwrap value of invalid Cfg_Var_Type {:?} (should have been String)!",
                v
            );
        }
    }
}

#[derive(Debug)]
pub struct Cfg_Var<T> {
    value: T,
}

impl<T> PartialEq<T> for Cfg_Var<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: Add<Output = T>> Add<T> for Cfg_Var<T>
where
    T: Add,
{
    type Output = T;

    fn add(self, other: T) -> T {
        self.value + other
    }
}

impl<T: Sub<Output = T>> Sub<T> for Cfg_Var<T>
where
    T: Sub,
{
    type Output = T;

    fn sub(self, other: T) -> T {
        self.value - other
    }
}

impl<T: Mul<Output = T>> Mul<T> for Cfg_Var<T>
where
    T: Mul,
{
    type Output = T;

    fn mul(self, other: T) -> T {
        self.value * other
    }
}

impl<T: Div<Output = T>> Div<T> for Cfg_Var<T>
where
    T: Div,
{
    type Output = T;

    fn div(self, other: T) -> T {
        self.value / other
    }
}

impl<T> Cfg_Var<T> {
    pub fn new(value: T) -> Self {
        Cfg_Var { value }
    }
}

impl<T> Deref for Cfg_Var<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl From<Cfg_Var<bool>> for bool {
    fn from(v: Cfg_Var<bool>) -> bool {
        v.value
    }
}

impl From<Cfg_Var<i32>> for i32 {
    fn from(v: Cfg_Var<i32>) -> i32 {
        v.value
    }
}

impl From<Cfg_Var<f32>> for f32 {
    fn from(v: Cfg_Var<f32>) -> f32 {
        v.value
    }
}

impl From<Cfg_Var<String>> for String {
    fn from(v: Cfg_Var<String>) -> String {
        v.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg;
    use crate::test_common::*;

    #[test]
    fn cfg_var_load() {
        let (loaders, _, _) = create_resource_loaders();
        let (_, env) = create_test_resources_and_env(&loaders);
        let config = cfg::Config::new(env.get_test_cfg_root());

        let entry_int = config.get_var::<i32>("test/entry_int");
        assert!(entry_int.is_some(), "Failed to load test/entry_int!");
        assert_eq!(entry_int.unwrap(), 42);

        let entry_bool = config.get_var::<bool>("test/entry_bool");
        assert!(entry_bool.is_some(), "Failed to load test/entry_bool!");
        assert_eq!(entry_bool.unwrap(), true);

        let entry_float = config.get_var::<f32>("test/entry_float");
        assert!(entry_float.is_some(), "Failed to load test/entry_float!");
        assert_eq!(entry_float.unwrap(), 42.0);

        let entry_string = config.get_var::<String>("test/entry_string");
        assert!(entry_string.is_some(), "Failed to load test/entry_string!");
        assert_eq!(entry_string.unwrap().as_str(), "Fourty Two");

        let entry_nil = config.get_var_or::<String, _>("test/entry_nil", "Nil!");
        assert_eq!(entry_nil.as_str(), "Nil!");

        let entry_int_as_float = config.get_var_or::<f32, _>("test/entry_int", -1.0);
        assert_eq!(entry_int_as_float, -1.0);
    }
}
