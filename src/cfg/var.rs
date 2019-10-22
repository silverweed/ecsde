use super::config::CFG_VAR_TABLE;
use super::value::Cfg_Value;
use crate::core::common::stringid::String_Id;
use std::convert::{From, Into, TryFrom};
use typename::TypeName;

pub(super) fn from_cfg<T>(var: Cfg_Var<T>) -> T
where
    T: Default + TypeName + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    let table = CFG_VAR_TABLE.read().unwrap();
    let value = table.get(&var.id).unwrap();
    T::try_from(value.clone()).unwrap_or_else(|_| {
        panic!(
            "Error dereferencing Cfg_Var<{}>: incompatible value {:?}",
            T::type_name(),
            value
        )
    })
}

#[derive(Debug, Copy, Clone)]
pub struct Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value>,
{
    id: String_Id,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value>,
{
    fn default() -> Cfg_Var<T> {
        Cfg_Var::empty()
    }
}

impl<T> Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value>,
{
    pub fn new(id: String_Id) -> Cfg_Var<T> {
        Cfg_Var {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn new_from_val(value: T) -> Cfg_Var<T> {
        let id = String_Id::from("empty"); // @Temporary
        let mut table = CFG_VAR_TABLE.write().unwrap();
        table.insert(id, value.into());
        Self::new(id)
    }

    pub fn empty() -> Cfg_Var<T> {
        Self::new_from_val(T::default())
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod tests {
    use crate::cfg;
    use crate::test_common::*;

    #[test]
    fn cfg_var_load() {
        let (_, _, env) = create_test_resources_and_env();
        let config = cfg::Config::new_from_dir(env.get_test_cfg_root());

        let entry_int = config.get_var_int("test/entry_int");
        assert!(entry_int.is_some(), "Failed to load test/entry_int!");
        assert_eq!(*entry_int.unwrap(), 42);

        let entry_bool = config.get_var_bool("test/entry_bool");
        assert!(entry_bool.is_some(), "Failed to load test/entry_bool!");
        assert_eq!(*entry_bool.unwrap(), true);

        let entry_float = config.get_var_float("test/entry_float");
        assert!(entry_float.is_some(), "Failed to load test/entry_float!");
        assert_eq!(*entry_float.unwrap(), 42.0);

        let entry_string = config.get_var_string("test/entry_string");
        assert!(entry_string.is_some(), "Failed to load test/entry_string!");
        assert_eq!(entry_string.unwrap().as_str(), "Fourty Two");

        let entry_nil = config.get_var_string_or("test/entry_nil", "Nil!");
        assert_eq!(entry_nil.as_str(), "Nil!");

        let entry_int_as_float = config.get_var_float_or("test/entry_int", -1.0);
        assert_eq!(*entry_int_as_float, -1.0);
    }
}
