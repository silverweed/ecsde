use inle_common::stringid::String_Id;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub trait Resource_Loader<'l, R> {
    type Args: ?Sized;

    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}

pub struct Cache<'l, Res, Loader>
where
    Loader: 'l + Resource_Loader<'l, Res, Args = str>,
{
    loader: &'l Loader,
    cache: HashMap<String_Id, Res>,
}

pub(super) type Res_Handle = Option<String_Id>;

impl<'l, Res, Loader> Cache<'l, Res, Loader>
where
    Loader: 'l + Resource_Loader<'l, Res, Args = str>,
{
    pub fn new_with_loader(loader: &'l Loader) -> Self {
        Cache {
            cache: HashMap::new(),
            loader,
        }
    }

    pub fn load(&mut self, fname: &str) -> Res_Handle {
        let id = String_Id::from(fname);
        match self.cache.entry(id) {
            Entry::Occupied(_) => Some(id),
            Entry::Vacant(v) => {
                let res = self
                    .loader
                    .load(fname)
                    .unwrap_or_else(|err| fatal!("Error loading {}: {}", fname, err));
                v.insert(res);
                lok!("Loaded resource {}", fname);
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
    ($loaded_res: ident, $loader_name: ident, $cache_name: ident) => {
        pub(super) struct $loader_name;

        impl<'l> loaders::Resource_Loader<'l, $loaded_res<'l>> for $loader_name {
            type Args = str;

            fn load(&'l self, fname: &str) -> Result<$loaded_res<'l>, String> {
                $loaded_res::from_file(fname).ok_or_else(|| {
                    format!(
                        concat!(
                            "[ WARNING ] Failed to load ",
                            stringify!($loaded_res),
                            " from {}"
                        ),
                        fname
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
