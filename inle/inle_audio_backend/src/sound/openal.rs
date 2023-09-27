use std::path::Path;

struct OpenAL_Error {
    code: al::ALenum,
}

impl OpenAL_Error {
    pub fn new(code: al::ALenum) -> Self {
        Self { code }
    }
}

impl std::fmt::Debug for OpenAL_Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: proper error string
        write!(f, "errcode {:?}", self.code)
    }
}

impl std::fmt::Display for OpenAL_Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Sound;

pub struct Sound_Buffer {
    buf: al::ALuint,
}

pub struct Audio_Context {
    device: *mut al::ALCdevice,
}

impl Drop for Audio_Context {
    fn drop(&mut self) {
        shutdown_audio(self);
    }
}

impl Sound_Buffer {
    pub fn from_file(fname: &str) -> Option<Self> {
        create_sound_buffer(&Path::new(fname)).ok()
    }
}

pub fn play_sound(_sound: &mut Sound) {}

pub fn sound_playing(_sound: &Sound) -> bool {
    false
}

pub fn create_sound_with_buffer(_buf: &Sound_Buffer) -> Sound {
    Sound
}

pub fn create_sound_buffer(file: &Path) -> Result<Sound_Buffer, String> {
    unsafe {
        // reset error state
        al::alGetError();
    }

    let mut buf: al::ALuint = 0;
    let err = unsafe {
        al::alGenBuffers(1, &mut buf);
        al::alGetError()
    };

    if err == al::AL_NO_ERROR {
        Ok(Sound_Buffer { buf })
    } else {
        Err(OpenAL_Error::new(err).to_string())
    }

    // TODO: load audio file, put data into the buffer
}

pub fn init_audio() -> Audio_Context {
    // Request default device
    let device = unsafe { al::alcOpenDevice(std::ptr::null()) };
    if device.is_null() {
        lerr!("Failed to get OpenAL device");
    } else {
        lok!("Successfully opened OpenAL device");
    }

    Audio_Context { device }
}

pub fn shutdown_audio(ctx: &mut Audio_Context) {
    if !ctx.device.is_null() {
        let ok = unsafe { al::alcCloseDevice(ctx.device) };
        if ok == al::ALC_TRUE {
            lok!("Successfully closed OpenAL device.");
        } else {
            lerr!("Failed to close OpenAL device!");
        }
        ctx.device = std::ptr::null_mut();
    }
}

mod al {
    use std::ffi;

    pub const ALC_TRUE: ALCboolean = 1;
    pub const ALC_FALSE: ALCboolean = 0;

    pub const AL_NO_ERROR: ALenum = 0;

    pub type ALCchar = ffi::c_char;
    pub type ALCdevice = ffi::c_void;
    pub type ALCboolean = ffi::c_char;

    pub type ALuint = ffi::c_uint;
    pub type ALsizei = ffi::c_int;
    pub type ALenum = ffi::c_int;

    #[link(name = "openal")]
    extern "C" {
        pub fn alcOpenDevice(devicename: *const ALCchar) -> *mut ALCdevice;
        pub fn alcCloseDevice(device: *mut ALCdevice) -> ALCboolean;

        pub fn alGetError() -> ALenum;
        pub fn alGenBuffers(n: ALsizei, buffers: *mut ALuint);
    }
}

mod libc {
    use std::ffi::{self, c_void};

    #[link(name = "libc")]
    extern "C" {
        pub fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
        pub fn ftell(ptr: *mut c_void) -> i64;
    }
}

mod ov {
    use std::ffi::{self, c_int, c_void};

    #[repr(C)]
    pub struct ov_callbacks {
        pub read_func:
            Option<unsafe extern "C" fn(*mut c_void, usize, usize, *mut c_void) -> usize>,
        pub seek_func: Option<unsafe extern "C" fn(*mut c_void, i64, ffi::c_int) -> ffi::c_int>,
        pub close_func: Option<unsafe extern "C" fn(*mut c_void) -> ffi::c_int>,
        pub tell_func: Option<unsafe extern "C" fn(*mut c_void) -> ffi::c_long>,
    }

    pub fn get_ov_callbacks_default() -> ov_callbacks {
        use super::libc;

        ov_callbacks {
            read_func: Some(libc::fread),
            seek_func: Some(_ov_header_fseek_wrap),
            close_func: None,
            tell_func: Some(libc::ftell),
        }
    }

    pub type OggVorbis_File = c_void;

    #[link(name = "vorbis")]
    extern "C" {
        pub fn ov_open_callbacks(
            datasource: *mut c_void,
            vf: *mut OggVorbis_File,
            ibytes: ffi::c_long,
            callbacks: ov_callbacks,
        );

        fn _ov_header_fseek_wrap(f: *mut c_void, off: i64, whence: ffi::c_int) -> ffi::c_int;
    }
}
