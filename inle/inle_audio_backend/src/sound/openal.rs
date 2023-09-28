use std::io;
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

#[derive(Default)]
pub struct Sound {
    source: al::ALuint,
}

impl Drop for Sound {
    fn drop(&mut self) {
        if self.source > 0 {
            unsafe {
                al::alDeleteSources(1, &mut self.source);
            }
        }
    }
}

pub struct Sound_Buffer {
    buf: al::ALuint,
}

impl Drop for Sound_Buffer {
    fn drop(&mut self) {
        if self.buf > 0 {
            unsafe {
                al::alDeleteBuffers(1, &mut self.buf);
            }
        }
    }
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

pub fn create_sound_with_buffer(buf: &Sound_Buffer) -> Sound {
    let sound = new_sound().unwrap_or_default();
    let err = unsafe {
        al::alSourcei(sound.source, al::AL_BUFFER, buf.buf);
        al::alGetError()
    };
    sound
}

fn new_sound() -> Result<Sound, String> {
    unsafe {
        // reset error state
        al::alGetError();
    }

    let mut source: al::ALuint = 0;
    let err = unsafe {
        al::alGenSources(1, &mut source);
        al::alGetError()
    };

    if err == al::AL_NO_ERROR {
        Ok(Sound { source })
    } else {
        Err(OpenAL_Error::new(err).to_string())
    }
}

pub fn create_sound_buffer(file: &Path) -> Result<Sound_Buffer, String> {
    let buf = new_sound_buf()?;

    // load audio file
    let mut decoded_file = read_and_decode_ogg_file(file).map_err(|err| err.to_string())?;

    // put data into the buffer
    let err = unsafe {
        al::alBufferData(
            buf.buf,
            decoded_file.format,
            decoded_file.samples.as_mut_ptr() as *mut _,
            std::mem::size_of_val(&decoded_file.samples) as _,
            decoded_file.freq,
        );
        al::alGetError()
    };

    if err == al::AL_NO_ERROR {
        Ok(buf)
    } else {
        Err(OpenAL_Error::new(err).to_string())
    }
}

fn new_sound_buf() -> Result<Sound_Buffer, String> {
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

struct Decoded_Ogg {
    pub samples: Vec<i16>,
    pub format: al::ALenum,
    pub freq: al::ALsizei,
}

fn read_and_decode_ogg_file(path: &Path) -> io::Result<Decoded_Ogg> {
    use io::Read;
    use lewton::inside_ogg as ogg;
    use std::fs;

    let file_reader = fs::File::open(path)?;
    let mut ogg_reader = ogg::OggStreamReader::new(file_reader)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let mut samples: Vec<i16> = vec![];
    while let Some(packets) = ogg_reader
        .read_dec_packet()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    {
        for packet in packets {
            samples.extend(packet);
        }
    }

    let format = match ogg_reader.ident_hdr.audio_channels {
        1 => al::AL_FORMAT_MONO16,
        2 => al::AL_FORMAT_STEREO16,
        n => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unsupported number of channels {}", n),
            ))
        }
    };
    let freq = ogg_reader.ident_hdr.audio_sample_rate as _;

    Ok(Decoded_Ogg {
        samples,
        format,
        freq,
    })
}

mod al {
    use std::ffi;

    pub const ALC_TRUE: ALCboolean = 1;
    pub const ALC_FALSE: ALCboolean = 0;

    pub const AL_NO_ERROR: ALenum = 0;

    pub const AL_FORMAT_MONO8: ALenum = 0x1100;
    pub const AL_FORMAT_MONO16: ALenum = 0x1101;
    pub const AL_FORMAT_STEREO8: ALenum = 0x1102;
    pub const AL_FORMAT_STEREO16: ALenum = 0x1103;

    // Note: al.h comment says it should be an ALint, but then it doesn't typecheck correctly.
    pub const AL_BUFFER: ALenum = 0x1009;

    pub type ALCchar = ffi::c_char;
    pub type ALCdevice = ffi::c_void;
    pub type ALCboolean = ffi::c_char;

    pub type ALuint = ffi::c_uint;
    pub type ALint = ffi::c_int;
    pub type ALsizei = ffi::c_int;
    pub type ALenum = ffi::c_int;
    pub type ALvoid = ffi::c_void;

    #[link(name = "openal")]
    extern "C" {
        pub fn alcOpenDevice(devicename: *const ALCchar) -> *mut ALCdevice;
        pub fn alcCloseDevice(device: *mut ALCdevice) -> ALCboolean;

        pub fn alGetError() -> ALenum;
        pub fn alGenBuffers(n: ALsizei, buffers: *mut ALuint);
        pub fn alDeleteBuffers(n: ALsizei, buffers: *mut ALuint);
        pub fn alBufferData(
            buffer: ALuint,
            format: ALenum,
            data: *mut ALvoid,
            size: ALsizei,
            freq: ALsizei,
        );

        pub fn alGenSources(n: ALsizei, sources: *mut ALuint);
        pub fn alDeleteSources(n: ALsizei, sources: *mut ALuint);
        pub fn alSourcei(source: ALuint, pname: ALenum, value: ALint);
    }
}


