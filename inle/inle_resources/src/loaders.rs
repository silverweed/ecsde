use inle_common::stringid::String_Id;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;

pub trait Resource_Loader<'l, R> {
    type Args: ?Sized;

    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}

pub struct Cache<'l, Res, Loader>
where
    Loader: 'l + Resource_Loader<'l, Res, Args = Path>,
{
    loader: &'l Loader,
    pub(super) cache: HashMap<String_Id, Res>,
}

pub(super) type Res_Handle = Option<String_Id>;

impl<'l, Res, Loader> Cache<'l, Res, Loader>
where
    Loader: 'l + Resource_Loader<'l, Res, Args = Path>,
{
    pub fn new_with_loader(loader: &'l Loader) -> Self {
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

        impl<'l> loaders::Resource_Loader<'l, $loaded_res<'l>> for $loader_name {
            type Args = std::path::Path;

            fn load(&'l self, fname: &std::path::Path) -> Result<$loaded_res<'l>, String> {
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

        pub(super) type $cache_name<'l> = loaders::Cache<'l, $loaded_res<'l>, $loader_name>;

        impl $cache_name<'_> {
            pub fn new() -> Self {
                Self::new_with_loader(&$loader_name {})
            }
        }
    };
}
