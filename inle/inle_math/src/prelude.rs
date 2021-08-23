#[macro_export]
macro_rules! v2 {
    ($x: expr, $y: expr $(,)?) => {
        $crate::vector::Vector2::new($x, $y)
    };
    ($x: expr) => {
        $crate::vector::Vector2::new($x, $x)
    };
}

#[macro_export]
macro_rules! v3 {
    ($x: expr, $y: expr, $z: expr $(,)?) => {
        $crate::vector::Vector3::new($x, $y, $z)
    };
    ($x: expr) => {
        $crate::vector::Vector3::new($x, $x, $x)
    };
}
