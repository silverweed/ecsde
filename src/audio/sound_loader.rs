use crate::resources::audio::Sound_Buffer;
use crate::resources::loaders::Resource_Loader;
use ears::SoundData;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Sound_Loader {}

impl<'l> Resource_Loader<'l, Sound_Buffer> for Sound_Loader {
    type Args = str;

    fn load(&'l self, fname: &str) -> Result<Sound_Buffer, String> {
        Ok(Rc::new(RefCell::new(SoundData::new(fname)?)))
    }
}
