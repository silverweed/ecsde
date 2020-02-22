use std::convert::From;

#[cfg(debug_assertions)]
use {std::collections::hash_map::Entry, std::collections::HashMap, std::sync::RwLock};

#[derive(PartialEq, Hash, Copy, Clone, PartialOrd, Eq, Ord)]
pub struct String_Id(u32);

#[cfg(debug_assertions)]
lazy_static! {
    static ref STRING_ID_MAP: RwLock<HashMap<String_Id, String>> = RwLock::new(HashMap::new());
}

impl String_Id {
    pub fn from_u32(x: u32) -> String_Id {
        String_Id(x)
    }
}

impl<'a, T> From<T> for String_Id
where
    &'a str: From<T>,
    T: 'a,
{
    fn from(s: T) -> String_Id {
        let s: &str = s.into();
        let this = String_Id(fnv1a(s.as_bytes()));
        #[cfg(debug_assertions)]
        {
            match STRING_ID_MAP
                .write()
                .expect("[ ERROR ] Failed to lock STRING_ID_MAP")
                .entry(this)
            {
                Entry::Occupied(o) => {
                    let old = o.get().as_str();
                    assert_eq!(
                        old, s,
                        "Two strings map to the same SID: {} and {}!",
                        old, s
                    );
                }
                Entry::Vacant(v) => {
                    v.insert(String::from(s));
                }
            }
        }
        this
    }
}

impl std::fmt::Display for String_Id {
    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }

    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} (orig = \"{}\")",
            self.0,
            STRING_ID_MAP
                .read()
                .expect("[ ERROR ] Failed to lock STRING_ID_MAP")[self]
        )
    }
}

impl std::fmt::Debug for String_Id {
    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "String_Id({})", self.0)
    }

    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "String_Id({}, \"{}\")",
            self.0,
            STRING_ID_MAP
                .read()
                .expect("[ ERROR ] Failed to lock STRING_ID_MAP")[self]
        )
    }
}

const FNV_PRIME32: u32 = 16_777_619;

#[inline]
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

    #[test]
    fn stringid_from_str() {
        assert_eq!(String_Id::from("A test string"), String_Id(943117577));
        assert_eq!(String_Id::from("A test string").0, fnv1a(b"A test string"));
    }
}
