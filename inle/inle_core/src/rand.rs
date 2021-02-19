use inle_serialize::{Binary_Serializable, Byte_Stream};
use std::sync::atomic::{AtomicUsize, Ordering};

// Implementation derived from https://github.com/BareRose/ranxoshi256/blob/master/ranxoshi256.h

pub type Default_Rng = Rand_Xoshiro256;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Default_Rng_Seed(pub [u8; 32]);

impl Binary_Serializable for Default_Rng_Seed {
    fn serialize(&self, out: &mut Byte_Stream) -> std::io::Result<()> {
        for byte in self.0.iter().take(32) {
            out.write_u8(*byte)?;
        }
        Ok(())
    }

    fn deserialize(input: &mut Byte_Stream) -> std::io::Result<Self> {
        let mut res = [0; 32];
        for byte in &mut res {
            *byte = input.read_u8()?;
        }
        Ok(Default_Rng_Seed(res))
    }
}

pub struct Rand_Xoshiro256 {
    state: [u64; 4],
}

pub fn new_random_seed() -> std::io::Result<Default_Rng_Seed> {
    let mut seed_buf = [0u8; 32];
    // @Robustness: consider hashing in the system time or something like that.
    get_entropy_from_os(&mut seed_buf)?;
    Ok(Default_Rng_Seed(seed_buf))
}

pub fn new_rng_with_random_seed() -> std::io::Result<Rand_Xoshiro256> {
    Ok(Rand_Xoshiro256::new_with_seed(new_random_seed()?.0))
}

pub fn new_rng_with_seed(seed: Default_Rng_Seed) -> Rand_Xoshiro256 {
    Rand_Xoshiro256::new_with_seed(seed.0)
}

pub fn rand_01(rng: &mut Rand_Xoshiro256) -> f32 {
    (rng.next() >> 32) as f32 / u32::max_value() as f32
}

pub fn rand_range(rng: &mut Rand_Xoshiro256, min: f32, max: f32) -> f32 {
    debug_assert!(min <= max);
    min + rand_01(rng) * (max - min)
}

impl Rand_Xoshiro256 {
    pub fn new_with_seed(s: [u8; 32]) -> Rand_Xoshiro256 {
        let mut state = [0u64; 4];
        for i in 0..state.len() {
            state[i] = (u64::from(s[i * 8]) << 56)
                | (u64::from(s[i * 8 + 1]) << 48)
                | (u64::from(s[i * 8 + 2]) << 40)
                | (u64::from(s[i * 8 + 3]) << 32)
                | (u64::from(s[i * 8 + 4]) << 24)
                | (u64::from(s[i * 8 + 5]) << 16)
                | (u64::from(s[i * 8 + 6]) << 8)
                | (u64::from(s[i * 8 + 7]));
        }

        Rand_Xoshiro256 { state }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let s = &mut self.state;
        let res = rand_xoshi256_rotate(s[1].wrapping_mul(5), 7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = rand_xoshi256_rotate(s[3], 45);
        res
    }
}

#[inline(always)]
fn rand_xoshi256_rotate(x: u64, k: i32) -> u64 {
    (x << k) | (x >> (64 - k))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_entropy_from_os(buf: &mut [u8]) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    let random_device_path = Path::new("/dev/urandom");
    let mut file = File::open(random_device_path)?;
    file.read_exact(buf)
}

#[cfg(target_os = "windows")]
mod win32 {
    use std::os::raw::*;
    pub(super) type PVOID = *mut c_void;
    pub(super) type BOOL = c_int;
    pub(super) type ULONG = c_ulong;

    extern "system" {
        /// @Portability: this function is not strictly standard, but it's just SO MUCH more convenient
        /// to use than the "recommended" BCrypt* API, which is a total mess.
        /// In case Windows 12 or something breaks this, then I'll take the trouble of using the
        /// Official(R) MDSN-Approved(TM) API.
        #[link(name = "Advapi32")]
        #[link_name = "SystemFunction036"]
        pub(super) fn RtlGenRandom(buf: PVOID, buf_len: ULONG) -> BOOL;
    }
}

#[cfg(target_os = "windows")]
fn get_entropy_from_os(buf: &mut [u8]) -> std::io::Result<()> {
    if unsafe { win32::RtlGenRandom(buf.as_mut_ptr() as win32::PVOID, buf.len() as win32::ULONG) }
        != 0
    {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "RtlGenRandom call failed.",
        ))
    }
}

/// Recycles a pool of precomputed random numbers with internal mutability.
/// Used to generate random numbers where immutability is needed.
pub struct Precomputed_Rand_Pool {
    pool: Vec<u64>,
    cur_idx: AtomicUsize,
}

impl Default for Precomputed_Rand_Pool {
    fn default() -> Self {
        Self {
            pool: vec![],
            cur_idx: AtomicUsize::default(),
        }
    }
}

impl Precomputed_Rand_Pool {
    pub fn with_size(rng: &mut Default_Rng, n: usize) -> Self {
        let mut pool = Vec::with_capacity(n);
        for _ in 0..n {
            pool.push(rng.next());
        }
        Self {
            pool,
            cur_idx: AtomicUsize::new(0),
        }
    }

    pub fn next(&self) -> u64 {
        let idx = self.cur_idx.fetch_add(1, Ordering::Relaxed);
        self.pool[idx % self.pool.len()]
    }

    pub fn rand_01(&self) -> f32 {
        (self.next() >> 32) as f32 / u32::max_value() as f32
    }

    pub fn rand_range(&self, min: f32, max: f32) -> f32 {
        assert!(min <= max);
        min + self.rand_01() * (max - min)
    }
}
