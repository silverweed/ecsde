pub struct Sound;
pub struct Sound_Buffer;
pub struct Audio_Context;

static mut INSTANCE: () = ();

impl Sound_Buffer {
    pub fn from_file(_fname: &str) -> Option<Self> {
        Some(Self)
    }
}

impl std::ops::Deref for Sound_Buffer {
    type Target = ();

    fn deref(&self) -> &Self::Target {
        unsafe { &INSTANCE }
    }
}

impl std::ops::DerefMut for Sound_Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut INSTANCE }
    }
}

pub fn play_sound(_sound: &mut Sound) {}

pub fn sound_playing(_sound: &Sound) -> bool {
    false
}

pub fn create_sound_with_buffer(_buf: &Sound_Buffer) -> Sound {
    Sound
}

pub fn init_audio() -> Audio_Context { Audio_Context }
