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

// Taken from `static_assertions` crate
#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}

// Utils to have in default scope
#[macro_export]
macro_rules! v2 {
    ($x: expr, $y: expr $(,)?) => {
        $crate::common::vector::Vector2::new($x, $y)
    };
}
