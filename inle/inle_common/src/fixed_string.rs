use std::fmt;

pub type Fixed_String_64 = Fixed_String<64>;

/// A copyable mutable string containing at most N characters.
/// Only works for ASCII.
#[derive(Copy, Clone)]
pub struct Fixed_String<const LEN: usize> {
    bytes: [u8; LEN],
    byte_count: u8,
}

impl<const LEN: usize> Fixed_String<LEN> {
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

impl<const LEN: usize> fmt::Debug for Fixed_String<LEN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.bytes.iter().take(self.byte_count as usize) {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

impl<const LEN: usize> fmt::Display for Fixed_String<LEN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl<const LEN: usize> Default for Fixed_String<LEN> {
    fn default() -> Self {
        Self {
            bytes: [0; LEN],
            byte_count: 0,
        }
    }
}

impl<const LEN: usize> PartialEq for Fixed_String<LEN> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.bytes == other.bytes
    }
}

impl<const LEN: usize> Eq for Fixed_String<LEN> {}

impl<const LEN: usize> PartialEq<str> for Fixed_String<LEN> {
    fn eq(&self, other: &str) -> bool {
        self == &Self::from(other)
    }
}

impl<const LEN: usize> From<&str> for Fixed_String<LEN> {
    fn from(s: &str) -> Self {
        // @WaitForStable: this can probably become a const_assert in the future
        assert!(s.len() <= LEN);

        let mut fs = Self {
            bytes: [0; LEN],
            byte_count: s.len() as u8,
        };

        for (dst, src) in fs
            .bytes
            .iter_mut()
            .zip(s.bytes().take(fs.byte_count as usize))
        {
            *dst = src;
        }

        //for (i, c) in s.bytes().take(fs.byte_count as usize).enumerate() {
        //fs.bytes[i] = c;
        //}

        fs
    }
}

impl<const LEN: usize> From<&Fixed_String<LEN>> for String {
    fn from(s: &Fixed_String<LEN>) -> String {
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

impl<const LEN: usize> AsRef<str> for Fixed_String<LEN> {
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
