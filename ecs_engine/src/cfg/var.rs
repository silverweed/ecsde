use super::config::Config;
use super::value::Cfg_Value;
use crate::core::common::stringid::String_Id;
use std::convert::{From, Into, TryFrom};
use std::fmt::Debug;
use typename::TypeName;

fn read_cfg<T>(path_id: String_Id, cfg: &Config) -> T
where
    T: Default + TypeName + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    let value = cfg
        .read_cfg(path_id)
        .unwrap_or_else(|| panic!("[ FATAL ] Tried to read inexistent Cfg_Var \"{}\"", path_id));

    T::try_from(value.clone()).unwrap_or_else(|_| {
        panic!(
            "[ FATAL ] Error dereferencing Cfg_Var<{}>: incompatible value {:?}",
            T::type_name(),
            value
        )
    })
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value>,
{
    id: String_Id,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(not(debug_assertions))]
#[derive(Debug, Clone)]
pub struct Cfg_Var<T>(T)
where
    T: Default + TypeName + Into<Cfg_Value>;

impl<T> Copy for Cfg_Var<T> where T: Copy + TypeName + Default + Into<Cfg_Value> {}

impl<T> Cfg_Var<T>
where
    T: Default + TypeName + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    #[cfg(debug_assertions)]
    pub fn new(path: &str) -> Cfg_Var<T> {
        Cfg_Var {
            id: String_Id::from(path),
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(path: &str) -> Cfg_Var<T> {
        let id = String_Id::from(path);
        Cfg_Var(read_cfg(id))
    }

    #[cfg(debug_assertions)]
    fn new_from_sid(id: String_Id) -> Cfg_Var<T> {
        Cfg_Var {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(debug_assertions)]
    pub fn new_from_val(value: T, cfg: &mut Config) -> Cfg_Var<T>
    where
        T: Debug,
    {
        let id = String_Id::from(format!("{:?}", value).as_str());
        cfg.write_cfg(id, value.into());
        Self::new_from_sid(id)
    }

    #[cfg(not(debug_assertions))]
    pub fn new_from_val(value: T, _: &mut Config) -> Cfg_Var<T> {
        Cfg_Var(value)
    }
}

impl Cfg_Var<bool> {
    #[cfg(debug_assertions)]
    pub fn read(self, cfg: &Config) -> bool {
        read_cfg(self.id, cfg)
    }

    #[cfg(not(debug_assertions))]
    pub fn read(self, _: &Config) -> bool {
        self.0
    }
}

impl Cfg_Var<i32> {
    #[cfg(debug_assertions)]
    pub fn read(self, cfg: &Config) -> i32 {
        read_cfg(self.id, cfg)
    }

    #[cfg(not(debug_assertions))]
    pub fn read(self, _: &Config) -> i32 {
        self.0
    }
}

impl Cfg_Var<f32> {
    #[cfg(debug_assertions)]
    pub fn read(self, cfg: &Config) -> f32 {
        read_cfg(self.id, cfg)
    }

    #[cfg(not(debug_assertions))]
    pub fn read(self, _: &Config) -> f32 {
        self.0
    }
}

impl Cfg_Var<String> {
    #[cfg(debug_assertions)]
    pub fn read(&self, cfg: &Config) -> String {
        read_cfg(self.id, cfg)
    }

    #[cfg(not(debug_assertions))]
    pub fn read(&self, _: &Config) -> String {
        self.0.clone()
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
    use super::*;
    use crate::cfg;
    use crate::test_common::*;

    #[test]
    fn cfg_var_load() {
        let (_, _, env) = create_test_resources_and_env();
        let _config = cfg::Config::new_from_dir(env.get_test_cfg_root());

        let entry_int = Cfg_Var::<i32>::new("test/entry_int");
        assert_eq!(entry_int.read(), 42);

        let entry_bool = Cfg_Var::<bool>::new("test/entry_bool");
        assert_eq!(entry_bool.read(), true);

        let entry_float = Cfg_Var::<f32>::new("test/entry_float");
        assert_eq!(entry_float.read(), 42.0);

        let entry_string = Cfg_Var::<String>::new("test/entry_string");
        assert_eq!(entry_string.read().as_str(), "Fourty Two");
    }

    #[test]
    #[should_panic]
    fn cfg_read_invalid() {
        let (_, _, env) = create_test_resources_and_env();
        let _config = cfg::Config::new_from_dir(env.get_test_cfg_root());

        let entry_nonexisting = Cfg_Var::<i32>::new("entry non existing");
        let _ = entry_nonexisting.read();
    }

    #[test]
    fn cfg_new_from_val() {
        let (_, _, env) = create_test_resources_and_env();
        let _config = cfg::Config::new_from_dir(env.get_test_cfg_root());

        let var = Cfg_Var::new_from_val(42);
        assert_eq!(var.read(), 42);

        let var = Cfg_Var::new_from_val(String::from("foo"));
        assert_eq!(var.read(), String::from("foo"));
    }

    #[test]
    #[should_panic]
    fn cfg_incompatible_type() {
        let (_, _, env) = create_test_resources_and_env();
        let _config = cfg::Config::new_from_dir(env.get_test_cfg_root());

        let entry_float_mistyped = Cfg_Var::<i32>::new("test/entry_float");
        let _ = entry_float_mistyped.read();
    }
}
