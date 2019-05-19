use super::parsing::Cfg_Value;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use typename::TypeName;

#[derive(Debug, Clone)]
pub struct Cfg_Var<T>
where
    T: Default,
{
    value: Rc<RefCell<T>>,
}

impl<T> Default for Cfg_Var<T>
where
    T: Default,
{
    fn default() -> Cfg_Var<T> {
        Cfg_Var::empty()
    }
}

impl<T> Cfg_Var<T>
where
    T: Default,
{
    pub fn new(val: &Rc<RefCell<T>>) -> Cfg_Var<T> {
        Cfg_Var { value: val.clone() }
    }

    pub fn new_from_val(value: T) -> Cfg_Var<T> {
        Cfg_Var {
            value: Rc::new(RefCell::new(value)),
        }
    }

    pub fn empty() -> Cfg_Var<T> {
        Cfg_Var {
            value: Rc::new(RefCell::new(T::default())),
        }
    }
}

impl<T> Deref for Cfg_Var<T>
where
    T: Default,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Note: the only place that can mutate this pointer's value is the Config struct.
        unsafe { &*self.value.as_ptr() }
        //&*self.value
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Cfg_Var<T>
where
    T: Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value.borrow())
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
