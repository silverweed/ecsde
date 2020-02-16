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
