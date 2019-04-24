#[derive(PartialEq, Hash, Copy, Clone, Debug)]
pub struct String_Id(u32);

impl Eq for String_Id {}

impl std::convert::From<&str> for String_Id {
    fn from(s: &str) -> String_Id {
        String_Id(fnv1a(s.as_bytes()))
    }
}

const FNV_PRIME32: u32 = 16_777_619;

fn fnv1a(bytes: &[u8]) -> u32 {
    let mut result = 2_166_136_261;
    for &b in bytes {
        result ^= u32::from(b);
        result = result.wrapping_mul(FNV_PRIME32);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a() {
        assert_eq!(fnv1a(b"A test string"), 0x3836d509);
        assert_eq!(fnv1a(b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
                         incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
                         exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure \
                         dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. \
                         Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt \
                         mollit anim id est laborum."), 0x7c0594dd);
    }
}
