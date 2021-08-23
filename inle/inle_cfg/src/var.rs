use super::config::Config;
use super::value::Cfg_Value;
use inle_common::stringid::String_Id;
use std::any::type_name;
use std::convert::{From, Into, TryFrom};
use std::fmt::{Debug, Display};

fn read_cfg<T>(path_id: String_Id, cfg: &Config) -> T
where
    T: Default + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    let value = cfg
        .read_cfg(path_id)
        .unwrap_or_else(|| fatal!("Tried to read inexistent Cfg_Var \"{}\"", path_id));

    T::try_from(value.clone()).unwrap_or_else(|_| {
        fatal!(
            "Error dereferencing Cfg_Var<{}>({}): incompatible value {:?}",
            type_name::<T>(),
            path_id,
            value
        )
    })
}

#[cfg(debug_assertions)]
fn read_cfg_str(path_id: String_Id, cfg: &Config) -> &String {
    let value = cfg
        .read_cfg(path_id)
        .unwrap_or_else(|| fatal!(r#"Tried to read inexistent Cfg_Var "{}""#, path_id));

    if let Cfg_Value::String(s) = value {
        s
    } else {
        fatal!(
            "Error dereferencing Cfg_Var<String>({}): incompatible value {:?}",
            path_id,
            value
        );
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
enum Cfg_Var_Content<T> {
    Fixed(T),
    Hot_Reloadable(String_Id),
}

#[cfg(debug_assertions)]
impl<T> Copy for Cfg_Var_Content<T> where T: Copy {}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct Cfg_Var<T>
where
    T: Default + Into<Cfg_Value>,
{
    content: Cfg_Var_Content<T>,
}

impl<T> Default for Cfg_Var<T>
where
    T: Default + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    #[cfg(debug_assertions)]
    fn default() -> Self {
        Cfg_Var {
            content: Cfg_Var_Content::Fixed(T::default()),
        }
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Cfg_Var(T::default())
    }
}

#[cfg(not(debug_assertions))]
#[derive(Debug, Clone)]
pub struct Cfg_Var<T>(T)
where
    T: Default + Into<Cfg_Value>;

impl<T> Copy for Cfg_Var<T> where T: Copy + Default + Into<Cfg_Value> {}

#[cfg(debug_assertions)]
impl<T> Cfg_Var<T>
where
    T: Default + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    pub fn new(path: &str, _cfg: &Config) -> Self {
        Self {
            content: Cfg_Var_Content::Hot_Reloadable(String_Id::from(path)),
        }
    }

    fn new_from_sid(id: String_Id) -> Self {
        Self {
            content: Cfg_Var_Content::Hot_Reloadable(id),
        }
    }

    pub fn new_from_val(value: T) -> Self
    where
        T: Debug,
    {
        Self {
            content: Cfg_Var_Content::Fixed(value),
        }
    }

    pub fn has_changed(&self, cfg: &Config) -> bool {
        match self.content {
            Cfg_Var_Content::Hot_Reloadable(id) => cfg.has_changed(id),
            _ => unreachable!(),
        }
    }
}

#[cfg(not(debug_assertions))]
impl<T> Cfg_Var<T>
where
    T: Default + Into<Cfg_Value> + TryFrom<Cfg_Value>,
{
    pub fn new(path: &str, cfg: &Config) -> Self {
        let id = String_Id::from(path);
        Self(read_cfg(id, cfg))
    }

    pub fn new_from_val(value: T) -> Self {
        Self(value)
    }
}

macro_rules! impl_cfg_vars {
    (copy: $($type: ty),*) => {
        $(
            impl Cfg_Var<$type> {
                #[cfg(debug_assertions)]
                pub fn read(self, cfg: &Config) -> $type {
                    match self.content {
                        Cfg_Var_Content::Fixed(x) => x,
                        Cfg_Var_Content::Hot_Reloadable(id) => read_cfg(id, cfg),
                    }
                }

                #[cfg(not(debug_assertions))]
                #[inline(always)]
                pub fn read(self, _: &Config) -> $type {
                    self.0
                }
            }
        )*
    };
    (noncopy: $($type: ty),*) => {
        $(
            impl Cfg_Var<$type> {
                #[cfg(debug_assertions)]
                pub fn read<'s, 'c>(&'s self, cfg: &'c Config) -> &'s $type
                where 'c: 's {
                    match &self.content {
                        Cfg_Var_Content::Fixed(x) => &x,
                        Cfg_Var_Content::Hot_Reloadable(id) => read_cfg_str(*id, cfg),
                    }
                }

                #[cfg(not(debug_assertions))]
                #[inline(always)]
                pub fn read(&self, _: &Config) -> &$type {
                    &self.0
                }
            }
        )*
    }
}

// @WaitForStable: if specialization lands, only have impls for T: Copy / NonCopy.
impl_cfg_vars!(copy: bool, i32, u32, f32);
impl_cfg_vars!(noncopy: String);

impl<T: Display> Display for Cfg_Var<T>
where
    T: Default + Into<Cfg_Value>,
{
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.content {
            Cfg_Var_Content::Fixed(x) => write!(f, "{}", x),
            Cfg_Var_Content::Hot_Reloadable(id) => write!(f, "(REF {})", id),
        }
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use inle_core::env::Env_Info;
    use inle_test::env as test_env;

    #[test]
    fn cfg_var_load() {
        let env = Env_Info::gather().expect("Failed to gather env info!");
        let config = Config::new_from_dir(&test_env::get_test_cfg_root(&env));

        let entry_int = Cfg_Var::<i32>::new("test/entry_int", &config);
        assert_eq!(entry_int.read(&config), 42);

        let entry_uint_hex = Cfg_Var::<u32>::new("test/entry_uint", &config);
        assert_eq!(entry_uint_hex.read(&config), 0x42);

        let entry_uint_color = Cfg_Var::<u32>::new("test/entry_color", &config);
        assert_eq!(entry_uint_color.read(&config), 0xFFFF0000);

        let entry_bool = Cfg_Var::<bool>::new("test/entry_bool", &config);
        assert_eq!(entry_bool.read(&config), true);

        let entry_float = Cfg_Var::<f32>::new("test/entry_float", &config);
        assert_eq!(entry_float.read(&config), 42.0);

        let entry_string = Cfg_Var::<String>::new("test/entry_string", &config);
        assert_eq!(entry_string.read(&config).as_str(), "Fourty Two");
    }

    #[test]
    #[should_panic]
    fn cfg_read_invalid() {
        let env = Env_Info::gather().expect("Failed to gather env info!");
        let config = Config::new_from_dir(&test_env::get_test_cfg_root(&env));

        let entry_nonexisting = Cfg_Var::<i32>::new("entry non existing", &config);
        let _ = entry_nonexisting.read(&config);
    }

    #[test]
    fn cfg_new_from_val() {
        let env = Env_Info::gather().expect("Failed to gather env info!");
        let config = Config::new_from_dir(&test_env::get_test_cfg_root(&env));

        let var: Cfg_Var<i32> = Cfg_Var::new_from_val(42);
        assert_eq!(var.read(&config), 42);

        let var = Cfg_Var::new_from_val(String::from("foo"));
        assert_eq!(var.read(&config), "foo");
    }

    #[test]
    #[should_panic]
    fn cfg_incompatible_type() {
        let env = Env_Info::gather().expect("Failed to gather env info!");
        let config = Config::new_from_dir(&test_env::get_test_cfg_root(&env));

        let entry_float_mistyped = Cfg_Var::<i32>::new("test/entry_float", &config);
        let _ = entry_float_mistyped.read(&config);
    }
}
