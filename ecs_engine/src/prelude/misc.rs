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

// Utils to have in default scope
#[macro_export]
macro_rules! v2 {
    ($x: expr, $y: expr $(,)?) => {
        $crate::common::vector::Vector2::new($x, $y)
    };
}
