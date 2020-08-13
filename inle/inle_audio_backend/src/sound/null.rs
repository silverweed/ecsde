use std::marker::PhantomData;

pub struct Sound<'a>(PhantomData<&'a ()>);
pub struct Sound_Buffer<'a>(PhantomData<&'a ()>);

static mut INSTANCE: () = ();

impl Sound_Buffer<'_> {
    pub fn from_file(fname: &str) -> Option<Self> {
        Some(Self(PhantomData))
    }
}

impl std::ops::Deref for Sound_Buffer<'_> {
    type Target = ();

    fn deref(&self) -> &Self::Target {
        unsafe { &INSTANCE }
    }
}

impl std::ops::DerefMut for Sound_Buffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut INSTANCE }
    }
}

pub fn play_sound(_sound: &mut Sound) {}

pub fn sound_playing(_sound: &Sound) -> bool {
    false
}

pub fn create_sound_with_buffer<'a>(buf: &'a Sound_Buffer) -> Sound<'a> {
    Sound(PhantomData)
}
