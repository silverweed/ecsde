#[cfg(debug_assertions)]
#[macro_export]
macro_rules! mut_in_debug {
    ($x: ident) => {
        mut $x
    }
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! mut_in_debug {
    ($x: ident) => {
        $x
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! pub_in_debug {
    ($($x: tt)*) => {
       pub $($x)*
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! pub_in_debug {
    ($($x: tt)*) => {
        $($x)*
    };
}

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        const _: () = assert!($x);
    };
}

#[derive(Debug)]
struct Generic_Error {
    msg: String,
}

impl std::error::Error for Generic_Error {}
impl std::fmt::Display for Generic_Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub fn generic_error<T: Into<String>>(t: T) -> Box<dyn std::error::Error> {
    Box::new(Generic_Error { msg: t.into() })
}

#[macro_export]
macro_rules! error {
    ($x: expr) => {
        $crate::generic_error($x)
    };
    () => {
        $crate::generic_error("")
    };
}

#[macro_export]
macro_rules! c_str {
    ($literal: expr) => {
        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes()) }
    };
}
