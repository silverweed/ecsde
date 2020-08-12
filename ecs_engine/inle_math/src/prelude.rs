#[macro_export]
macro_rules! v2 {
    ($x: expr, $y: expr $(,)?) => {
        $crate::vector::Vector2::new($x, $y)
    };
}
