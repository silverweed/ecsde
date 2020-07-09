use std::fmt;

/// A copyable mutable string containing at most N characters.
/// Only works for ASCII.
// @WaitForStable: allow using a generic length via template
#[derive(Copy, Clone)]
pub struct Fixed_String_64 {
    bytes: [u8; Self::MAX_BYTES],
    byte_count: u8,
}

impl Fixed_String_64 {
    const MAX_BYTES: usize = 64;

    pub fn len(&self) -> usize {
        self.byte_count as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn set(&mut self, s: &str) {
        *self = Self::from(s);
    }
}

impl fmt::Debug for Fixed_String_64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.bytes.iter().take(self.byte_count as usize) {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

impl Default for Fixed_String_64 {
    fn default() -> Self {
        Self {
            bytes: [0; Self::MAX_BYTES],
            byte_count: 0,
        }
    }
}

impl PartialEq for Fixed_String_64 {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for i in 0..self.len() {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }

        true
    }
}

impl Eq for Fixed_String_64 {}

impl PartialEq<str> for Fixed_String_64 {
    fn eq(&self, other: &str) -> bool {
        self == &Self::from(other)
    }
}

impl From<&str> for Fixed_String_64 {
    fn from(s: &str) -> Self {
        // @WaitForStable: this can probably become a const_assert in the future
        assert!(s.len() <= Fixed_String_64::MAX_BYTES);

        let mut fs = Self {
            bytes: [0; Self::MAX_BYTES],
            byte_count: s.len() as u8,
        };

        for (i, c) in s.bytes().take(fs.byte_count as usize).enumerate() {
            fs.bytes[i] = c;
        }

        fs
    }
}

impl From<&Fixed_String_64> for String {
    fn from(s: &Fixed_String_64) -> String {
        String::from_utf8(
            s.bytes
                .iter()
                .copied()
                .take(s.byte_count as usize)
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }
}

impl AsRef<str> for Fixed_String_64 {
    // @Robustness: this only works if the Fixed_String is ASCII-only!
    fn as_ref(&self) -> &str {
        let ptr = self.bytes.as_ptr();
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, self.byte_count as usize) })
            .unwrap()
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn fixed_string_create_eq() {
        let fs = Fixed_String_64::from("foo_bar  Baz!!");
        let fs2 = fs;
        assert_eq!(fs, fs2);
        assert_eq!(fs, "foo_bar  Baz!!");
    }
}
