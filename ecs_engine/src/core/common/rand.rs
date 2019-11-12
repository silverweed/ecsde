// Implementation derived from https://github.com/BareRose/ranxoshi256/blob/master/ranxoshi256.h

pub type Default_Rng = Rand_Xoshiro256;

pub struct Rand_Xoshiro256 {
    pub state: [u64; 4],
}

pub fn new_rng() -> std::io::Result<Rand_Xoshiro256> {
    let mut seed_buf = [0u8; 32];
    get_entropy_from_os(&mut seed_buf)?;
    // @Robustness: consider hashing in the system time
    // or something like that.
    Ok(Rand_Xoshiro256::new_with_seed(seed_buf))
}

pub fn rand_01(rng: &mut Rand_Xoshiro256) -> f32 {
    (rng.next() >> 32) as f32 / u32::max_value() as f32
}

pub fn rand_range(rng: &mut Rand_Xoshiro256, min: f32, max: f32) -> f32 {
    assert!(min <= max);
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

#[cfg(target_os = "linux")]
fn get_entropy_from_os(buf: &mut [u8]) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    let random_device_path = Path::new("/dev/urandom");
    let mut file = File::open(random_device_path)?;
    file.read_exact(buf)
}
