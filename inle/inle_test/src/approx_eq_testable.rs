pub trait Approx_Eq_Testable {
    fn cmp_list(&self) -> Vec<f32>;
}

impl Approx_Eq_Testable for f32 {
    fn cmp_list(&self) -> Vec<f32> {
        vec![*self]
    }
}
