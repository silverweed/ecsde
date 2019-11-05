use crate::resources::loaders;
use ears::SoundData;
use std::cell::RefCell;
use std::rc::Rc;

pub(super) type Sound_Buffer = Rc<RefCell<SoundData>>;
pub(super) struct Sound_Loader;

impl<'l> loaders::Resource_Loader<'l, Sound_Buffer> for Sound_Loader {
    type Args = str;

    fn load(&'l self, fname: &str) -> Result<Sound_Buffer, String> {
        Ok(Rc::new(RefCell::new(SoundData::new(fname)?)))
    }
}

pub(super) type Sound_Cache<'l> = loaders::Cache<'l, Sound_Buffer, Sound_Loader>;

impl Sound_Cache<'_> {
    pub fn new() -> Self {
        Self::new_with_loader(&Sound_Loader {})
    }
}
