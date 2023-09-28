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
        write!(f, "OpenAL errcode {:?}", self.code)
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
    bufs: Vec<al::ALuint>,
}

impl Drop for Sound_Buffer {
    fn drop(&mut self) {
        unsafe {
            al::alDeleteBuffers(self.bufs.len() as _, self.bufs.as_mut_ptr());
        }
    }
}

pub struct Audio_Context {
    device: *mut al::ALCdevice,
    context: *mut al::ALCcontext,
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

pub fn init_audio() -> Audio_Context {
    let mut context = std::ptr::null_mut();

    // Request default device
    let device = unsafe { al::alcOpenDevice(std::ptr::null()) };
    if device.is_null() {
        lerr!("Failed to get OpenAL device");
    } else {
        lok!("Successfully opened OpenAL device");

        context = unsafe { al::alcCreateContext(device, std::ptr::null_mut()) };
        if context.is_null() {
            lerr!("Failed to create OpenAL context");
        } else {
            let ok = unsafe { al::alcMakeContextCurrent(context) };
            if ok != al::ALC_TRUE {
                lerr!("Failed to make OpenAL context current");
            }
        }
    }

    let err = unsafe { al::alcGetError(device) };
    if err != al::ALC_NO_ERROR {
        lerr!("Error initting audio: ALC errcode {}", err);
    }

    Audio_Context { device, context }
}

pub fn shutdown_audio(ctx: &mut Audio_Context) {
    if !ctx.context.is_null() {
        unsafe { al::alcDestroyContext(ctx.context) };
        ctx.context = std::ptr::null_mut();
    }
    if !ctx.device.is_null() {
        let ok = unsafe { al::alcCloseDevice(ctx.device) };
        if ok != al::ALC_TRUE {
            lerr!("Failed to close OpenAL device!");
        }
        ctx.device = std::ptr::null_mut();
    }
}

pub fn create_sound_buffer(file: &Path) -> Result<Sound_Buffer, String> {
    // load audio file
    let mut decoded_file = read_and_decode_ogg_file(file).map_err(|err| err.to_string())?;

    let sound_buf = new_sound_buf(1)?;

    // put data into the buffers
    let err = unsafe {
        let size = std::mem::size_of::<i16>() * decoded_file.buffers.len();
        al::alBufferData(
            sound_buf.bufs[0],
            decoded_file.format,
            decoded_file.buffers.as_mut_ptr() as *mut _,
            size as _,
            decoded_file.freq,
        );
        al::alGetError()
    };

    if err == al::AL_NO_ERROR {
        Ok(sound_buf)
    } else {
        Err(OpenAL_Error::new(err).to_string())
    }
}

fn new_sound_buf(n_internal_buffers: usize) -> Result<Sound_Buffer, String> {
    unsafe {
        // reset error state
        al::alGetError();
    }

    let mut bufs = vec![0; n_internal_buffers];
    let err = unsafe {
        al::alGenBuffers(n_internal_buffers as _, bufs.as_mut_ptr());
        al::alGetError()
    };

    if err == al::AL_NO_ERROR {
        Ok(Sound_Buffer { bufs })
    } else {
        Err(OpenAL_Error::new(err).to_string())
    }
}

pub fn create_sound_with_buffer(buf: &Sound_Buffer) -> Sound {
    let sound = new_sound().unwrap_or_default();
    // @Speed: we clone the buffer handles so we don't have to pass `buf` as mutable.
    // alSourceQueueBuffers probably doesn't really need a mutable pointer, but that's
    // what the signature asks for, so we comply.
    let mut bufs = buf.bufs.clone();
    debug_assert!(unsafe { al::alIsBuffer(bufs[0]) == al::AL_TRUE });
    let err = unsafe {
        al::alSourcei(sound.source, al::AL_BUFFER, bufs[0] as _);
        al::alGetError()
    };
    if err != al::AL_NO_ERROR {
        lerr!("Error creating sound: OpenAL errcode {}", err);
    }
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
        ldebug!("Created sound with OpenAL source {}.", source);
        debug_assert!(unsafe { al::alIsSource(source) == al::AL_TRUE });
        Ok(Sound { source })
    } else {
        Err(OpenAL_Error::new(err).to_string())
    }
}

pub fn play_sound(sound: &mut Sound) {
    if sound.source == 0 {
        lwarn!("Sound wasn't played because source is invalid.");
        return;
    }

    let mut gain: al::ALfloat = 0.0;
    unsafe {
        al::alGetSourcef(sound.source, al::AL_GAIN, &mut gain);
    }
    ldebug!("Gain: {}", gain);

    let err = unsafe {
        al::alGetError();
        al::alSourcePlay(sound.source);
        al::alGetError()
    };

    if err != al::AL_NO_ERROR {
        lerr!("Error playing sound: OpenAL errcode {}", err);
    }
}

pub fn sound_playing(sound: &Sound) -> bool {
    let mut state: al::ALint = 0;
    let err = unsafe {
        al::alGetError();
        al::alGetSourcei(sound.source, al::AL_SOURCE_STATE, &mut state);
        al::alGetError()
    };
    if err != al::AL_NO_ERROR {
        lerr!(
            "Error retrieving sound source state: OpenAL errcode {}",
            err
        );
        false
    } else {
        dbg!(state) == al::AL_PLAYING
    }
}

struct Decoded_Ogg {
    pub buffers: Vec<i16>,
    pub format: al::ALenum,
    pub freq: al::ALsizei,
}

fn read_and_decode_ogg_file(path: &Path) -> io::Result<Decoded_Ogg> {
    use lewton::inside_ogg as ogg;
    use std::fs;

    let file_reader = fs::File::open(path)?;
    let mut ogg_reader = ogg::OggStreamReader::new(file_reader)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let mut buffers = vec![];
    let mut play_length = 0.;
    let header = &ogg_reader.ident_hdr;
    let sample_channels = header.audio_channels as f32 * header.audio_sample_rate as f32;
    let freq = header.audio_sample_rate as _;
    while let Some(packets) = ogg_reader
        .read_dec_packet_itl()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    {
        play_length += packets.len() as f32 / sample_channels;
        buffers.extend(packets);
    }

    let format = match ogg_reader.ident_hdr.audio_channels {
        // @Incomplete: how to differentiate 8/16 bits?
        1 => al::AL_FORMAT_MONO16,
        2 => al::AL_FORMAT_STEREO16,
        n => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unsupported number of channels {}", n),
            ))
        }
    };

    ldebug!("Decoded OGG file of length {play_length} s, sample rate {freq} Hz, format {format:?}");

    Ok(Decoded_Ogg {
        buffers,
        format,
        freq,
    })
}

mod al {
    use std::ffi;

    pub const ALC_TRUE: ALCboolean = 1;
    pub const ALC_FALSE: ALCboolean = 0;

    pub const AL_TRUE: ALboolean = 1;
    pub const AL_FALSE: ALboolean = 0;

    pub const ALC_NO_ERROR: ALCenum = ALC_FALSE as _;
    pub const AL_NO_ERROR: ALenum = 0;

    pub const AL_FORMAT_MONO8: ALenum = 0x1100;
    pub const AL_FORMAT_MONO16: ALenum = 0x1101;
    pub const AL_FORMAT_STEREO8: ALenum = 0x1102;
    pub const AL_FORMAT_STEREO16: ALenum = 0x1103;

    // Note: al.h comment says it should be an ALint, but then it doesn't typecheck correctly.
    pub const AL_BUFFER: ALenum = 0x1009;
    pub const AL_GAIN: ALenum = 0x100A;
    pub const AL_SOURCE_STATE: ALenum = 0x1010;
    pub const AL_PLAYING: ALenum = 0x1012;

    pub type ALCdevice = ffi::c_void;
    pub type ALCcontext = ffi::c_void;
    pub type ALCchar = ffi::c_char;
    pub type ALCboolean = ffi::c_char;
    pub type ALCint = ffi::c_int;
    pub type ALCenum = ffi::c_int;

    pub type ALuint = ffi::c_uint;
    pub type ALint = ffi::c_int;
    pub type ALsizei = ffi::c_int;
    pub type ALenum = ffi::c_int;
    pub type ALvoid = ffi::c_void;
    pub type ALboolean = ffi::c_char;
    pub type ALfloat = ffi::c_float;

    #[link(name = "openal")]
    extern "C" {
        pub fn alcGetError(device: *mut ALCdevice) -> ALCenum;
        pub fn alcOpenDevice(devicename: *const ALCchar) -> *mut ALCdevice;
        pub fn alcCloseDevice(device: *mut ALCdevice) -> ALCboolean;
        pub fn alcCreateContext(device: *mut ALCdevice, attrlist: *mut ALCint) -> *mut ALCcontext;
        pub fn alcDestroyContext(context: *mut ALCcontext);
        pub fn alcMakeContextCurrent(context: *mut ALCcontext) -> ALCboolean;

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
        pub fn alIsBuffer(buf: ALuint) -> ALboolean;

        pub fn alGenSources(n: ALsizei, sources: *mut ALuint);
        pub fn alDeleteSources(n: ALsizei, sources: *mut ALuint);
        pub fn alSourcei(source: ALuint, pname: ALenum, value: ALint);
        pub fn alSourcef(source: ALuint, pname: ALenum, value: ALfloat);
        pub fn alGetSourcei(source: ALuint, pname: ALenum, value: *mut ALint);
        pub fn alGetSourcef(source: ALuint, pname: ALenum, value: *mut ALfloat);
        pub fn alSourcePlay(source: ALuint);
        pub fn alSourceQueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);
        pub fn alIsSource(source: ALuint) -> ALboolean;
    }
}
