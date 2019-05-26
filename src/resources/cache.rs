use super::loaders::Resource_Loader;
use crate::audio::sound_loader::Sound_Loader;
use crate::core::common;
use crate::core::common::colors::Color;
use crate::core::common::stringid::String_Id;
use crate::gfx::render::Texture;
use ears::SoundData;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Sound_Buffer = Rc<RefCell<SoundData>>;

pub struct Resource_Manager<'l, Res, Loader>
where
    Loader: 'l + Resource_Loader<'l, Res>,
{
    loader: &'l Loader,
    cache: HashMap<String_Id, Res>,
}

impl<'l, Res, Loader> Resource_Manager<'l, Res, Loader>
where
    Loader: 'l + Resource_Loader<'l, Res>,
{
    pub fn new(loader: &'l Loader) -> Self {
        Resource_Manager {
            cache: HashMap::new(),
            loader,
        }
    }

    pub fn load<'d, D>(&mut self, load_info: &'d D) -> Result<Option<String_Id>, String>
    where
        Loader: Resource_Loader<'l, Res, Args = D>,
        D: ?Sized + std::fmt::Debug + 'd,
        String_Id: std::convert::From<&'d D>,
    {
        let path_id = String_Id::from(load_info);
        if self.cache.get(&path_id).is_none() {
            let resource = self.loader.load(load_info)?;
            eprintln!("Loaded resource {:?}", load_info);
            self.cache.insert(path_id, resource);
        }
        Ok(Some(path_id))
    }

    pub fn n_loaded(&self) -> usize {
        self.cache.len()
    }

    pub fn must_get<'a>(&'a self, res: Option<String_Id>) -> &'a Res
    where
        'l: 'a,
    {
        &self.cache[&res.unwrap()]
    }

    pub fn must_get_mut<'a>(&'a mut self, res: Option<String_Id>) -> &'a mut Res
    where
        'l: 'a,
    {
        self.cache.get_mut(&res.unwrap()).unwrap()
    }
}

pub type Sound_Manager<'l> = Resource_Manager<'l, Sound_Buffer, Sound_Loader>;
