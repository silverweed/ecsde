#![allow(clippy::op_ref)]

use std::fmt::{Debug, Formatter};
use std::ops::*;

pub struct Matrix3<T> {
    columns: [[T; 3]; 3],
}

impl<T> Matrix3<T> {
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        r1c1: T, r1c2: T, r1c3: T,
        r2c1: T, r2c2: T, r2c3: T,
        r3c1: T, r3c2: T, r3c3: T,
    ) -> Self {
        Self {
            columns: [
                [r1c1, r2c1, r3c1],
                [r1c2, r2c2, r3c2],
                [r1c3, r2c3, r3c3],
            ],
        }
    }
}

impl<T> Matrix3<T>
where
    T: Copy,
{
    pub fn transposed(&self) -> Self {
        let c = &self.columns;
        Self {
            columns: [
                [c[0][0], c[1][0], c[2][0]],
                [c[0][1], c[1][1], c[2][1]],
                [c[0][2], c[1][2], c[2][2]],
            ],
        }
    }
}

impl<T> Matrix3<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Neg<Output = T>,
{
    pub fn determinant(&self) -> T {
        let c = &self.columns;

        c[0][0] * c[1][1] * c[2][2] + c[1][0] * c[2][1] * c[0][2] + c[2][0] * c[0][1] * c[1][2]
            - c[0][0] * c[2][1] * c[1][2]
            - c[2][0] * c[1][1] * c[0][2]
            - c[1][0] * c[0][1] * c[2][2]
    }

    //pub fn cofactor_matrix(&self) -> Matrix3<T> {
    //Matrix3::new(
    //}

    //pub fn cofactor_at(&self, row: usize, column: usize) -> T {
    //// Note: if matrix is
    ////
    //// |A  B  C|
    //// |D  E  F|
    //// |G  H  I|
    ////                             |E  F|
    //// then cofactor at 0,0 is det(|H  I|)
    ////                         |B  C|
    //// cofactor at 1,0 is -det(|H  I|) and so on.
    //let c = &self.columns;

    //todo!();
    //}
}

//impl Matrix3<f32> {
//pub fn inverse(&self) -> Matrix3<f32> {
//self.cofactor_matrix().transposed() / self.determinant()
//}
//}

impl<T> Copy for Matrix3<T> where T: Copy {}

impl<T> Clone for Matrix3<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            columns: self.columns.clone(),
        }
    }
}

impl<T> Default for Matrix3<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            columns: [
                [T::default(), T::default(), T::default()],
                [T::default(), T::default(), T::default()],
                [T::default(), T::default(), T::default()],
            ],
        }
    }
}

impl<T> Debug for Matrix3<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let c = &self.columns;
        write!(
            f,
            "
{:?} {:?} {:?}
{:?} {:?} {:?}
{:?} {:?} {:?}
",
            c[0][0], c[1][0], c[2][0], c[0][1], c[1][1], c[2][1], c[0][2], c[1][2], c[2][2]
        )
    }
}

impl<T> PartialEq for Matrix3<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let a = &self.columns;
        let b = &other.columns;
        for i in 0..3 {
            for j in 0..3 {
                if a[i][j] != b[i][j] {
                    return false;
                }
            }
        }
        true
    }
}

impl<T> Eq for Matrix3<T> where T: Eq {}

impl<T> Index<usize> for Matrix3<T> {
    type Output = [T; 3];

    fn index(&self, col: usize) -> &Self::Output {
        &self.columns[col]
    }
}

impl<T> Index<(usize, usize)> for Matrix3<T> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.columns[col][row]
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix3<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.columns[col][row]
    }
}

// @Incomplete (maybe @WaitForStable): this should really be impl<T, M> Mul<M>,
// but that gives conflicting implementation
impl<T> Mul<T> for &Matrix3<T>
where
    T: Copy + Mul<T>,
{
    type Output = Matrix3<<T as Mul<T>>::Output>;

    fn mul(self, s: T) -> Self::Output {
        Self::Output {
            columns: [
                [
                    self.columns[0][0] * s,
                    self.columns[0][1] * s,
                    self.columns[0][2] * s,
                ],
                [
                    self.columns[1][0] * s,
                    self.columns[1][1] * s,
                    self.columns[1][2] * s,
                ],
                [
                    self.columns[2][0] * s,
                    self.columns[2][1] * s,
                    self.columns[2][2] * s,
                ],
            ],
        }
    }
}

impl<T> Mul<T> for Matrix3<T>
where
    T: Copy + Mul<T>,
{
    type Output = Matrix3<<T as Mul<T>>::Output>;

    fn mul(self, s: T) -> Self::Output {
        &self * s
    }
}

impl<T> Div<T> for &Matrix3<T>
where
    T: Copy + Div<T>,
{
    type Output = Matrix3<<T as Div<T>>::Output>;

    fn div(self, s: T) -> Self::Output {
        Self::Output {
            columns: [
                [
                    self.columns[0][0] / s,
                    self.columns[0][1] / s,
                    self.columns[0][2] / s,
                ],
                [
                    self.columns[1][0] / s,
                    self.columns[1][1] / s,
                    self.columns[1][2] / s,
                ],
                [
                    self.columns[2][0] / s,
                    self.columns[2][1] / s,
                    self.columns[2][2] / s,
                ],
            ],
        }
    }
}

impl<T> Div<T> for Matrix3<T>
where
    T: Copy + Div<T>,
{
    type Output = Matrix3<<T as Div<T>>::Output>;

    fn div(self, s: T) -> Self::Output {
        &self / s
    }
}
impl<T> Mul<&Matrix3<T>> for &Matrix3<T>
where
    T: Copy + Mul<T, Output = T> + Add<T, Output = T>,
{
    type Output = Matrix3<T>;

    fn mul(self, other: &Matrix3<T>) -> Matrix3<T> {
        let a = &self.columns;
        let b = &other.columns;
        Matrix3 {
            columns: [
                [
                    a[0][0] * b[0][0] + a[1][0] * b[0][1] + a[2][0] * b[0][2],
                    a[0][1] * b[0][0] + a[1][1] * b[0][1] + a[2][1] * b[0][2],
                    a[0][2] * b[0][0] + a[1][2] * b[0][1] + a[2][2] * b[0][2],
                ],
                [
                    a[0][0] * b[1][0] + a[1][0] * b[1][1] + a[2][0] * b[1][2],
                    a[0][1] * b[1][0] + a[1][1] * b[1][1] + a[2][1] * b[1][2],
                    a[0][2] * b[1][0] + a[1][2] * b[1][1] + a[2][2] * b[1][2],
                ],
                [
                    a[0][0] * b[2][0] + a[1][0] * b[2][1] + a[2][0] * b[2][2],
                    a[0][1] * b[2][0] + a[1][1] * b[2][1] + a[2][1] * b[2][2],
                    a[0][2] * b[2][0] + a[1][2] * b[2][1] + a[2][2] * b[2][2],
                ],
            ],
        }
    }
}

impl<T> Mul<Matrix3<T>> for Matrix3<T>
where
    T: Copy + Mul<T, Output = T> + Add<T, Output = T>,
{
    type Output = Matrix3<T>;

    fn mul(self, other: Matrix3<T>) -> Matrix3<T> {
        &self * &other
    }
}

impl<T> Add<&Matrix3<T>> for &Matrix3<T>
where
    T: Copy + Add<T, Output = T>,
{
    type Output = Matrix3<T>;

    fn add(self, other: &Matrix3<T>) -> Self::Output {
        let a = &self.columns;
        let b = &other.columns;
        Self::Output {
            columns: [
                [a[0][0] + b[0][0], a[0][1] + b[0][1], a[0][2] + b[0][2]],
                [a[1][0] + b[1][0], a[1][1] + b[1][1], a[1][2] + b[1][2]],
                [a[2][0] + b[2][0], a[2][1] + b[2][1], a[2][2] + b[2][2]],
            ],
        }
    }
}

impl<T> Add<Matrix3<T>> for Matrix3<T>
where
    T: Copy + Add<T, Output = T>,
{
    type Output = Matrix3<T>;

    fn add(self, other: Matrix3<T>) -> Self::Output {
        &self + &other
    }
}

impl<T> Sub<&Matrix3<T>> for &Matrix3<T>
where
    T: Copy + Sub<T, Output = T>,
{
    type Output = Matrix3<T>;

    fn sub(self, other: &Matrix3<T>) -> Self::Output {
        let a = &self.columns;
        let b = &other.columns;
        Self::Output {
            columns: [
                [a[0][0] - b[0][0], a[0][1] - b[0][1], a[0][2] - b[0][2]],
                [a[1][0] - b[1][0], a[1][1] - b[1][1], a[1][2] - b[1][2]],
                [a[2][0] - b[2][0], a[2][1] - b[2][1], a[2][2] - b[2][2]],
            ],
        }
    }
}

impl<T> Sub<Matrix3<T>> for Matrix3<T>
where
    T: Copy + Sub<T, Output = T>,
{
    type Output = Matrix3<T>;

    fn sub(self, other: Matrix3<T>) -> Self::Output {
        &self - &other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexing() {
        #[rustfmt::skip]
        let mut m = Matrix3::new(
            0, 1, 2,
            3, 4, 5,
            6, 7, 8
        );

        assert_eq!(m[0], [0, 3, 6]);
        assert_eq!(m[0][0], 0);
        assert_eq!(m[2][0], 2);
        assert_eq!(m[0][1], 3);
        assert_eq!(m[(0, 0)], 0);
        assert_eq!(m[(2, 0)], 6);
        assert_eq!(m[(0, 1)], 1);

        m[(1, 2)] = 7;
        assert_eq!(m[(1, 2)], 7);
        assert_eq!(m[2][1], 7);
    }

    #[test]
    #[rustfmt::skip]
    fn add() {
        let a = Matrix3::new(
            0, 0, 1,
            0, 1, 0,
            1, 1, 1
        );
        let b = Matrix3::new(
            3, 3, 0,
            2, 0, 2,
            5, 6, 7
        );
        assert_eq!(a + b, Matrix3::new(
            3, 3, 1,
            2, 1, 2,
            6, 7, 8
        ));
    }

    #[test]
    #[rustfmt::skip]
    fn sub() {
        let a = Matrix3::new(
            0, 0, 1,
            0, 1, 0,
            1, 1, 1
        );
        let b = Matrix3::new(
            3, 3, 0,
            2, 0, 2,
            5, 6, 7
        );
        assert_eq!(a - b, Matrix3::new(
            -3, -3, 1,
            -2, 1, -2,
            -4, -5, -6
        ));
    }

    #[test]
    fn mul_by_scalar() {
        assert_eq!(
            Matrix3::new(1, 2, 3, 4, 4, 4, 0, 9, 0) * 2,
            Matrix3::new(2, 4, 6, 8, 8, 8, 0, 18, 0)
        );
    }

    #[test]
    fn div_by_scalar() {
        assert_eq!(
            Matrix3::new(1, 2, 3, 4, 4, 4, 0, 9, 0) / 2,
            Matrix3::new(0, 1, 1, 2, 2, 2, 0, 4, 0)
        );
    }

    #[test]
    #[rustfmt::skip]
    fn mul_matrix() {
        let a = Matrix3::new(
            0, 0, 1,
            0, 1, 0,
            1, 1, 1
        );
        let b = Matrix3::new(
            3, 3, 0,
            2, 0, 2,
            5, 6, 7
        );

        assert_eq!(a * b, Matrix3::new(
            5, 6, 7,
            2, 0, 2,
            10, 9, 9
        ));
        assert_eq!(b * a, Matrix3::new(
            0, 3, 3,
            2, 2, 4,
            7, 13, 12
        ));
    }

    #[test]
    #[rustfmt::skip]
    fn determinant() {
        let a = Matrix3::new(
            3.2, 4.6, 0.9,
            -3.9, 0.1, -4.8,
            10., 33., -2.1
        );

        assert_approx_eq!(a.determinant(), 131.004);
    }
}
