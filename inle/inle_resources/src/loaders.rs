use inle_common::stringid::String_Id;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;

pub trait Resource_Loader<R> {
    type Args: ?Sized;

    fn load(&self, data: &Self::Args) -> Result<R, String>;
}

pub struct Cache<Res, Loader>
where
    Loader: Resource_Loader<Res, Args = Path>,
{
    loader: Loader,
    pub cache: HashMap<String_Id, Res>,
}

pub type Res_Handle = Option<String_Id>;

impl<Res, Loader> Cache<Res, Loader>
where
    Loader: Resource_Loader<Res, Args = Path>,
{
    pub fn new_with_loader(loader: Loader) -> Self {
        Cache {
            cache: HashMap::new(),
            loader,
        }
    }

    pub fn load(&mut self, fname: &Path) -> Res_Handle {
        let id = String_Id::from(fname.to_str().unwrap());
        match self.cache.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                let res = self
                    .loader
                    .load(fname)
                    .unwrap_or_else(|err| fatal!("Error loading {}: {}", fname.display(), err));
                v.insert(res);
                lok!("Loaded resource {}", fname.display());
                Some(id)
            }
        }
    }

    pub fn must_get(&self, handle: Res_Handle) -> &Res {
        &self.cache[&handle.unwrap()]
    }

    pub fn must_get_mut(&mut self, handle: Res_Handle) -> &mut Res {
        self.cache.get_mut(&handle.unwrap()).unwrap()
    }

    pub fn n_loaded(&self) -> usize {
        self.cache.len()
    }
}

#[macro_export]
macro_rules! define_file_loader {
    ($loaded_res: ident, $loader_name: ident, $cache_name: ident, $load_fn: path) => {
        pub(super) struct $loader_name;

        impl loaders::Resource_Loader<$loaded_res> for $loader_name {
            type Args = std::path::Path;

            fn load(&self, fname: &std::path::Path) -> Result<$loaded_res, String> {
                $load_fn(fname).map_err(|err| {
                    format!(
                        concat!(
                            "[ WARNING ] Failed to load ",
                            stringify!($loaded_res),
                            " from {}: {}"
                        ),
                        fname.display(),
                        err.to_string()
                    )
                })
            }
        }

        pub(super) struct $cache_name($crate::loaders::Cache<$loaded_res, $loader_name>);

        impl $cache_name {
            pub fn new() -> Self {
                Self($crate::loaders::Cache::new_with_loader($loader_name {}))
            }
        }

        impl std::ops::Deref for $cache_name {
            type Target = $crate::loaders::Cache<$loaded_res, $loader_name>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $cache_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}
