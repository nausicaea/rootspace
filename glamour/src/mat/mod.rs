use std::{
    iter::Sum,
};

use num_traits::{One, Zero, Float};
use crate::ops::norm::Norm;

pub mod vec2;
pub mod vec3;
pub mod vec4;
mod ops;
mod convert;
mod serde;
mod approx;

pub use self::vec2::Vec2;
pub use self::vec3::Vec3;
pub use self::vec4::Vec4;

// Generalized vector, interpreted as column
type Vec_<R, const I: usize> = Mat<R, I, 1>;

/// Matrix of 2x2 dimensions
pub type Mat2<R> = Mat<R, 2, 2>;

/// Matrix of 3x3 dimensions
pub type Mat3<R> = Mat<R, 3, 3>;

/// Matrix of 4x4 dimensions
pub type Mat4<R> = Mat<R, 4, 4>;

/// Generalized matrix type, with data stored in row-major format.
#[derive(Debug, PartialEq, Clone)]
pub struct Mat<R, const I: usize, const J: usize>([[R; J]; I]);

impl<R, const I: usize, const J: usize> Mat<R, I, J> {
    /// Given a one-dimensional array index, return the corresponding two-dimensional indices for
    /// this particular matrix' dimensions
    fn as_2d_idx(idx: usize) -> (usize, usize) {
        (idx / J, idx % J)
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J> 
where
    R: Copy + Zero,
{
    /// Return a copy of the specified matrix column
    pub fn col(&self, j: usize) -> Mat<R, I, 1> {
        if j >= J {
            panic!("Index j is out of bounds (max: {}, actual: {})", J, j);
        }
        let mut mat = Mat::<R, I, 1>::zero();
        for i in 0..I {
            mat[(i, 0)] = self[(i, j)];
        }
        mat
    }

    /// Return a copy of the specified matrix row
    pub fn row(&self, i: usize) -> Mat<R, 1, J> {
        if i >= I {
            panic!("Index i is out of bounds (max: {}, actual: {})", I, i);
        }
        let mut mat = Mat::<R, 1, J>::zero();
        for j in 0..J {
            mat[(0, j)] = self[(i, j)];
        }
        mat
    }

    /// Return a sub-matrix of the given size with the given starting index
    pub fn subset<const O: usize, const P: usize>(&self, i: usize, j: usize) -> Mat<R, O, P> {
        debug_assert!(O <= I && P <= J);
        debug_assert!(i + O <= I && j + P <= J);

        let mut mat = Mat::<R, O, P>::zero();
        for o in 0..O {
            for p in 0..P {
                mat[(o, p)] = self[(i + o, j + p)];
            }
        }
        mat
    }
}

impl<'a, R, const I: usize, const J: usize> Norm for &'a Mat<R, I, J> 
where
    R: Float + Sum,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        self.0.iter().flatten().map(|e| e.powi(2)).sum::<R>().sqrt()
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: Zero + Copy,
{
    pub fn t(&self) -> Mat<R, J, I> {
        let mut mat = Mat::<R, J, I>::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(j, i)] = self[(i, j)];
            }
        }
        mat
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: Float,
{
    pub fn is_nan(&self) -> bool {
        self.0.iter().flatten().any(|e| e.is_nan())
    }
}


impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: Zero + Copy,
{
    pub fn zero() -> Self {
        Mat([[R::zero(); J]; I])
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: One + Copy,
{
    pub fn one() -> Self {
        Mat([[R::one(); J]; I])
    }
}

impl<R, const I: usize> Mat<R, I, I>
where
    R: Zero + One + Copy,
{
    pub fn identity() -> Self {
        let mut mat = Mat::<R, I, I>::zero();
        for i in 0..I {
            mat[(i, i)] = R::one();
        }

        mat
    }
}

impl<R, const I: usize> Mat<R, I, I> 
where
    R: Zero + Copy,
{
    pub fn diag(&self) -> Vec_<R, I> {
        let mut mat = Vec_::<R, I>::zero();
        for i in 0..I {
            mat[i] = self[(i, i)];
        }

        mat
    }
}

impl<R, const I: usize, const J: usize> std::fmt::Display for Mat<R, I, J>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..I {
            write!(f, "[")?;
            for j in 0..J {
                write!(f, "{}", self[(i, j)])?;
                if j < J - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
            if i < I - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::dot::Dot;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn a2i_1x1() {
        assert_eq!(Mat::<f32, 1, 1>::as_2d_idx(0), (0, 0), "i=0");
    }

    #[test]
    fn a2i_1x2() {
        assert_eq!(Mat::<f32, 1, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 1, 2>::as_2d_idx(1), (0, 1), "i=1");
    }

    #[test]
    fn a2i_1x3() {
        assert_eq!(Mat::<f32, 1, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 1, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 1, 3>::as_2d_idx(2), (0, 2), "i=2");
    }

    #[test]
    fn a2i_1x4() {
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(3), (0, 3), "i=3");
    }

    #[test]
    fn a2i_2x1() {
        assert_eq!(Mat::<f32, 2, 1>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 1>::as_2d_idx(1), (1, 0), "i=1");
    }

    #[test]
    fn a2i_2x2() {
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(2), (1, 0), "i=2");
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(3), (1, 1), "i=3");
    }

    #[test]
    fn a2i_2x3() {
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(3), (1, 0), "i=3");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(4), (1, 1), "i=4");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(5), (1, 2), "i=5");
    }

    #[test]
    fn a2i_2x4() {
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(3), (0, 3), "i=3");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(4), (1, 0), "i=4");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(5), (1, 1), "i=5");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(6), (1, 2), "i=6");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(7), (1, 3), "i=7");
    }

    #[test]
    fn a2i_3x1() {
        assert_eq!(Mat::<f32, 3, 1>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 1>::as_2d_idx(1), (1, 0), "i=1");
        assert_eq!(Mat::<f32, 3, 1>::as_2d_idx(2), (2, 0), "i=2");
    }

    #[test]
    fn a2i_3x2() {
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(2), (1, 0), "i=2");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(3), (1, 1), "i=3");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(4), (2, 0), "i=4");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(5), (2, 1), "i=5");
    }

    #[test]
    fn a2i_3x3() {
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(3), (1, 0), "i=3");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(4), (1, 1), "i=4");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(5), (1, 2), "i=5");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(6), (2, 0), "i=6");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(7), (2, 1), "i=7");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(8), (2, 2), "i=8");
    }

    #[test]
    fn a2i_3x4() {
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(3), (0, 3), "i=3");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(4), (1, 0), "i=4");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(5), (1, 1), "i=5");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(6), (1, 2), "i=6");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(7), (1, 3), "i=7");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(8), (2, 0), "i=8");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(9), (2, 1), "i=9");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(10), (2, 2), "i=10");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(11), (2, 3), "i=11");
    }

    #[test]
    fn a2i_4x1() {
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(1), (1, 0), "i=1");
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(2), (2, 0), "i=2");
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(3), (3, 0), "i=3");
    }

    #[test]
    fn a2i_4x2() {
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(2), (1, 0), "i=2");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(3), (1, 1), "i=3");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(4), (2, 0), "i=4");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(5), (2, 1), "i=5");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(6), (3, 0), "i=6");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(7), (3, 1), "i=7");
    }

    #[test]
    fn a2i_4x3() {
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(3), (1, 0), "i=3");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(4), (1, 1), "i=4");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(5), (1, 2), "i=5");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(6), (2, 0), "i=6");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(7), (2, 1), "i=7");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(8), (2, 2), "i=8");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(9), (3, 0), "i=9");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(10), (3, 1), "i=10");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(11), (3, 2), "i=11");
    }

    #[test]
    fn a2i_4x4() {
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(3), (0, 3), "i=3");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(4), (1, 0), "i=4");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(5), (1, 1), "i=5");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(6), (1, 2), "i=6");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(7), (1, 3), "i=7");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(8), (2, 0), "i=8");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(9), (2, 1), "i=9");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(10), (2, 2), "i=10");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(11), (2, 3), "i=11");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(12), (3, 0), "i=12");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(13), (3, 1), "i=13");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(14), (3, 2), "i=14");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(15), (3, 3), "i=15");
    }

    #[test]
    fn mat_implements_display() {
        let a: Mat<f32, 2, 3> = Mat::from([1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(format!("{}", a), "[[1, 2, 3], [4, 5, 6]]");

        let a: Mat<f32, 1, 2> = Mat::from([1.0f32, 2.0]);
        assert_eq!(format!("{}", a), "[[1, 2]]");

        let a: Mat<f32, 2, 1> = Mat::from([1.0f32, 2.0]);
        assert_eq!(format!("{}", a), "[[1], [2]]");
    }

    #[test]
    fn mat_implements_from_2d_array() {
        let _: Mat<f32, 2, 2> = Mat::from([[0.0, 1.0], [2.0, 3.0]]);
    }

    #[test]
    fn mat_1x1_implements_from_scalar_value() {
        let _: Mat<f32, 1, 1> = (1.0f32).into();
    }

    #[test]
    fn mat_implements_from_array() {
        let _: Mat<f32, 1, 1> = Mat::from([0.0f32; 1]);
        let _: Mat<f32, 1, 2> = Mat::from([0.0f32; 2]);
        let _: Mat<f32, 1, 3> = Mat::from([0.0f32; 3]);
        let _: Mat<f32, 1, 4> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, 2, 1> = Mat::from([0.0f32; 2]);
        let _: Mat<f32, 2, 2> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, 2, 3> = Mat::from([0.0f32; 6]);
        let _: Mat<f32, 2, 4> = Mat::from([0.0f32; 8]);
        let _: Mat<f32, 3, 1> = Mat::from([0.0f32; 3]);
        let _: Mat<f32, 3, 2> = Mat::from([0.0f32; 6]);
        let _: Mat<f32, 3, 3> = Mat::from([0.0f32; 9]);
        let _: Mat<f32, 3, 4> = Mat::from([0.0f32; 12]);
        let _: Mat<f32, 4, 1> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, 4, 2> = Mat::from([0.0f32; 8]);
        let _: Mat<f32, 4, 3> = Mat::from([0.0f32; 12]);
        let _: Mat<f32, 4, 4> = Mat::from([0.0f32; 16]);
    }

    #[test]
    fn mat_supports_1d_indexing() {
        let m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[2], 3.0f32);
    }

    #[test]
    fn mat_supports_mut_1d_indexing() {
        let mut m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[2] = 5.0f32;
        assert_eq!(m[2], 5.0f32);
    }

    #[test]
    fn mat_supports_2d_indexing() {
        let m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[(1, 1)], 4.0f32);
    }

    #[test]
    fn mat_supports_mut_2d_indexing() {
        let mut m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[(1, 1)] = 5.0f32;
        assert_eq!(m[(1, 1)], 5.0f32);
    }

    #[test]
    fn mat_supports_transposition() {
        let a: Mat<f32, 2, 3> = Mat::from([1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b: Mat<f32, 3, 2> = a.t();
        assert_eq!(b, Mat::<f32, 3, 2>::from([1.0f32, 4.0, 2.0, 5.0, 3.0, 6.0]));
    }

    #[test]
    fn mat_provides_zero_constructor() {
        let m: Mat<f32, 2, 2> = Mat::zero();
        assert_eq!(m, Mat::<f32, 2, 2>::from([0.0f32; 4]));

        let m: Mat<f32, 2, 3> = Mat::zero();
        assert_eq!(m, Mat::<f32, 2, 3>::from([0.0f32; 6]));
    }

    #[test]
    fn mat_supports_one_constructor() {
        let m: Mat<f32, 2, 2> = Mat::one();
        assert_eq!(m, Mat::<f32, 2, 2>::from([1.0f32; 4]));

        let m: Mat<f32, 2, 3> = Mat::one();
        assert_eq!(m, Mat::<f32, 2, 3>::from([1.0f32; 6]));
    }

    #[test]
    fn mat_supports_identity_constructor() {
        let m: Mat<f32, 2, 2> = Mat::identity();
        assert_eq!(m, Mat::<f32, 2, 2>::from([1.0f32, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn mat_supports_scalar_addition() {
        let a: Mat<f32, 1, 1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a + &b, Mat::<f32, 1, 1>::from([3.0]));
        assert_eq!(&b + &a, Mat::<f32, 1, 1>::from([3.0]));
    }

    #[test]
    fn mat_supports_scalar_subtraction() {
        let a: Mat<f32, 1, 1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a - &b, Mat::<f32, 1, 1>::from([-1.0]));
        assert_eq!(&b - &a, Mat::<f32, 1, 1>::from([1.0]));
    }

    #[test]
    fn mat_supports_scalar_multiplication() {
        let a: Mat<f32, 1, 1> = Mat::from([2.0]);
        let b: f32 = 2.0;
        assert_eq!(&a * &b, Mat::<f32, 1, 1>::from([4.0]));
        assert_eq!(&b * &a, Mat::<f32, 1, 1>::from([4.0]));
    }

    #[test]
    fn mat_supports_scalar_division() {
        let a: Mat<f32, 1, 1> = Mat::from([6.0]);
        let b: f32 = 2.0;
        assert_eq!(&a / &b, Mat::<f32, 1, 1>::from([3.0]));
        assert_eq!(&b / &a, Mat::<f32, 1, 1>::from([2.0 / 6.0]));
    }

    #[test]
    fn mat_supports_addition() {
        let a: Mat<f32, 1, 1> = Mat::from([3.0]);
        let b: Mat<f32, 1, 1> = Mat::from([2.0]);
        assert_eq!(&a + &b, Mat::<f32, 1, 1>::from([5.0]));
        assert_eq!(&b + &a, Mat::<f32, 1, 1>::from([5.0]));
    }

    #[test]
    fn mat_supports_subtraction() {
        let a: Mat<f32, 1, 1> = Mat::from([3.0]);
        let b: Mat<f32, 1, 1> = Mat::from([2.0]);
        assert_eq!(&a - &b, Mat::<f32, 1, 1>::from([1.0]));
        assert_eq!(&b - &a, Mat::<f32, 1, 1>::from([-1.0]));
    }

    #[test]
    fn mat_supports_dot_product_2x1_1x2() {
        let a: Mat<f32, 2, 1> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, 1, 2> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), Mat::<f32, 2, 2>::from([6.0, 3.0, 4.0, 2.0]));
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x1() {
        let a: Mat<f32, 1, 2> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, 2, 1> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), 8.0f32);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x2() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 2, 2> = Mat::from([2.0, 3.0, 4.0, 5.0]);
        let c: Mat<f32, 2, 2> = Mat::from([10.0, 13.0, 22.0, 29.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x2() {
        let a: Mat<f32, 1, 2> = Mat::from([2.0, 3.0]);
        let b: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let c: Mat<f32, 1, 2> = Mat::from([11.0, 16.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x1() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 2, 1> = Mat::from([2.0, 3.0]);
        let c: Mat<f32, 2, 1> = Mat::from([8.0, 18.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x3() {
        let a: Mat<f32, 3, 3> = Mat::from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let b: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, 3, 3> = Mat::from([36., 42., 48., 81., 96., 111., 126., 150., 174.]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x3_3x3() {
        let a: Mat<f32, 1, 3> = Mat::from([1.0, 2.0, 3.0]);
        let b: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, 1, 3> = Mat::from([36.0, 42.0, 48.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x1() {
        let a: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let b: Mat<f32, 3, 1> = Mat::from([1.0, 2.0, 3.0]);
        let c: Mat<f32, 3, 1> = Mat::from([20.0, 38.0, 56.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x4() {
        let a: Mat<f32, 4, 4> = Mat::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat<f32, 4, 4> = Mat::from([
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ]);
        let c: Mat<f32, 4, 4> = Mat::from([
            100., 110., 120., 130., 228., 254., 280., 306., 356., 398., 440., 482., 484., 542., 600., 658.,
        ]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x4_4x4() {
        let a: Mat<f32, 1, 4> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 4, 4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let c: Mat<f32, 1, 4> = Mat::from([100.0, 110.0, 120.0, 130.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x1() {
        let a: Mat<f32, 4, 4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let b: Mat<f32, 4, 1> = Mat::from([1.0, 2.0, 3.0, 3.0]);
        let c: Mat<f32, 4, 1> = Mat::from([35.0, 71.0, 107.0, 143.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat2_x_vec2_works_as_premultiplication_of_the_matrix() {
        let m: Mat2<f32> = Mat2::identity();
        let v: Vec2<f32> = Vec2::one();

        assert_eq!(m * v, Vec2::one());
    }

    #[test]
    fn mat3_x_vec3_works_as_premultiplication_of_the_matrix() {
        let m: Mat3<f32> = Mat3::identity();
        let v: Vec3<f32> = Vec3::one();

        assert_eq!(m * v, Vec3::one());
    }

    #[test]
    fn mat4_x_vec4_works_as_premultiplication_of_the_matrix() {
        let m: Mat4<f32> = Mat4::identity();
        let v: Vec4<f32> = Vec4::one();

        assert_eq!(m * v, Vec4::one());
    }

    #[test]
    fn mat_provides_is_nan() {
        let a: Mat<f32, 2, 2> = Mat::from([f32::NAN, 1.0, 1.0, 1.0]);
        assert!(a.is_nan());

        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 1.0, 1.0, 1.0]);
        assert!(!a.is_nan());
    }

    #[test]
    fn mat_provides_col_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 3.0]);
        let b: Mat<f32, 2, 1> = a.col(0);
        assert_eq!(b, Mat::<f32, 2, 1>::from([1.0f32, 3.0]));
    }

    #[test]
    fn mat_provides_row_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 1, 2> = a.row(0);
        assert_eq!(b, Mat::<f32, 1, 2>::from([1.0f32, 2.0]));
    }

    #[test]
    fn mat_provides_subset_method() {
        let a: Mat<f32, 4, 4> = Mat::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat<f32, 2, 2> = a.subset::<2, 2>(0, 1);
        assert_eq!(b, Mat::<f32, 2, 2>::from([2.0f32, 3.0, 6.0, 7.0]));
    }

    #[test]
    fn mat_provides_norm_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(a.norm(), 5.477225575051661f32);
    }

    #[test]
    fn mat_probides_diag_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(a.diag(), Vec2::new(1.0f32, 4.0f32));
    }

    #[test]
    fn vec2_implements_new() {
        let _: Vec2<f32> = Vec2::new(1.0f32, 2.0f32);
    }

    #[test]
    fn vec3_implements_new() {
        let _: Vec3<f32> = Vec3::new(1.0f32, 2.0f32, 3.0f32);
    }

    #[test]
    fn vec4_implements_new() {
        let _: Vec4<f32> = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    }

    #[test]
    fn vec2_implements_x_and_y() {
        let v: Vec2<f32> = Vec2::new(1.0, 2.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
    }

    #[test]
    fn vec3_implements_x_y_and_z() {
        let v: Vec3<f32> = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
        assert_eq!(v.z(), 3.0f32);
    }

    #[test]
    fn vec4_implements_x_y_z_and_w() {
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
        assert_eq!(v.z(), 3.0f32);
        assert_eq!(v.w(), 4.0f32);
    }

    #[test]
    fn mat_implements_serde() {
        let a: Mat<f32, 2, 2> = Mat::identity();

        assert_tokens(
            &a,
            &[
                Token::Seq { len: Some(4) },
                Token::F32(1.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::SeqEnd,
            ],
        );
    }
}
