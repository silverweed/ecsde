use crate::resources::loaders;
use sfml::audio as sfaud;

pub(super) type Sound<'a> = sfaud::Sound<'a>;
pub(super) type Sound_Buffer = sfaud::SoundBuffer;
pub(super) struct Sound_Loader;

impl<'l> loaders::Resource_Loader<'l, Sound_Buffer> for Sound_Loader {
    type Args = str;

    fn load(&'l self, fname: &str) -> Result<Sound_Buffer, String> {
        let buf = sfaud::SoundBuffer::from_file(fname)
            .ok_or_else(|| format!("[ WARNING ] Failed to load sound from {}", fname))?;
        Ok(buf)
    }
}

pub(super) type Sound_Cache<'l> = loaders::Cache<'l, Sound_Buffer, Sound_Loader>;

impl Sound_Cache<'_> {
    pub fn new() -> Self {
        Self::new_with_loader(&Sound_Loader {})
    }
}
