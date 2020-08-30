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
    pub const fn from_u32(x: u32) -> String_Id {
        String_Id(x)
    }

    pub const fn val(self) -> u32 {
        self.0
    }
}

impl<'a, T> From<T> for String_Id
where
    &'a str: From<T>,
    T: 'a,
{
    fn from(s: T) -> String_Id {
        trace!("String_Id::from");

        let s: &str = s.into();
        sid_from_str(s)
    }
}

#[cfg(debug_assertions)]
pub fn sid_from_str(s: &str) -> String_Id {
    let this = String_Id(fnv1a(s.as_bytes()));
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

#[cfg(not(debug_assertions))]
pub const fn sid_from_str(s: &str) -> String_Id {
    String_Id(fnv1a(s.as_bytes()))
}

#[macro_export]
macro_rules! sid {
    ($str: expr) => {
        $crate::stringid::sid_from_str($str)
    };
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
            "{}",
            STRING_ID_MAP
                .read()
                .expect("[ ERROR ] Failed to lock STRING_ID_MAP")
                .get(self) // this may fail if we created the String_Id from an integer directly
                .unwrap_or(&format!("{}", self.0))
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
                .expect("[ ERROR ] Failed to lock STRING_ID_MAP")
                .get(self)
                .map_or("??", |s| &s)
        )
    }
}

pub const FNV1A_PRIME32: u32 = 16_777_619;
pub const FNV1A_START32: u32 = 2_166_136_261;

const fn fnv1a(bytes: &[u8]) -> u32 {
    let mut result = FNV1A_START32;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        result ^= b as u32;
        result = result.wrapping_mul(FNV1A_PRIME32);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a() {
        const_assert!(fnv1a(b"A test string") == 943117577);
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
        assert_eq!(sid!("A test string"), String_Id(943117577));
        assert_eq!(sid!("A test string").0, fnv1a(b"A test string"));
    }

    #[test]
    fn stringid_to_str() {
        assert_eq!(
            sid!("Another test string").to_string(),
            String::from("Another test string")
        );
    }
}
