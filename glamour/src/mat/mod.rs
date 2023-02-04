use num_traits::Float;

use crate::{vec::Vec4, One, Unit, Zero};

mod approx;
mod convert;
mod ops;
#[cfg(any(test, feature = "serde_support"))]
mod serde;

/// Generalized matrix type, with data stored in row-major format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4<R>(pub [[R; 4]; 4]);

impl<R> Mat4<R> {
    pub const fn new(v: [[R; 4]; 4]) -> Self {
        Mat4(v)
    }

    /// Given a one-dimensional array index, return the corresponding two-dimensional indices for
    /// this particular matrix' dimensions
    fn as_2d_idx(idx: usize) -> (usize, usize) {
        (idx / 4, idx % 4)
    }
}

impl<R> Mat4<R>
where
    Vec4<R>: Zero,
    R: Copy,
{
    /// Return a copy of the specified matrix column
    pub fn col(&self, j: usize) -> Vec4<R> {
        if j >= 4 {
            panic!("Out of bounds column index (max: {}, actual: {})", 4, j);
        }

        let mut mat = Vec4::zero();
        for i in 0..4 {
            mat[i] = self[(i, j)];
        }
        mat
    }

    /// Return a copy of the specified matrix row
    pub fn row(&self, i: usize) -> Vec4<R> {
        if i >= 4 {
            panic!("Out of bounds row (max: {}, actual: {})", 4, i);
        }
        let mut mat = Vec4::zero();
        for j in 0..4 {
            mat[j] = self[(i, j)];
        }
        mat
    }

    pub fn diag(&self) -> Vec4<R> {
        let mut mat = Vec4::zero();
        for i in 0..4 {
            mat[i] = self[(i, i)];
        }

        mat
    }
}

impl<R> Mat4<R>
where
    Self: Zero,
    R: Copy,
{
    pub fn t(&self) -> Mat4<R> {
        let mut mat = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                mat[(j, i)] = self[(i, j)];
            }
        }
        mat
    }
}

impl<R> Mat4<R>
where
    R: num_traits::Float,
{
    pub fn look_at_lh(fwd: Unit<Vec4<R>>, up: Unit<Vec4<R>>) -> Self {
        use crate::Cross;

        let side: Unit<_> = up.cross(fwd);
        let rotated_up = fwd.cross(side);

        Mat4([
            [side.x, side.y, side.z, R::zero()],
            [rotated_up.x, rotated_up.y, rotated_up.z, R::zero()],
            [fwd.x, fwd.y, fwd.z, R::zero()],
            [R::zero(), R::zero(), R::zero(), R::one()],
        ])
    }
}

impl<R> Mat4<R>
where
    R: Float,
{
    pub fn is_nan(&self) -> bool {
        self.0.iter().flatten().any(|e| e.is_nan())
    }
}

impl<R> Mat4<R>
where
    Self: Zero,
    R: num_traits::One,
{
    pub fn identity() -> Self {
        let mut mat = Mat4::zero();
        for i in 0..4 {
            mat[(i, i)] = R::one();
        }

        mat
    }
}

impl<R> Zero for Mat4<R>
where
    R: Copy + num_traits::Zero,
{
    fn zero() -> Self {
        Mat4([[R::zero(); 4]; 4])
    }
}

impl<R> One for Mat4<R>
where
    R: Copy + num_traits::One,
{
    fn one() -> Self {
        Mat4([[R::one(); 4]; 4])
    }
}

impl<R> std::fmt::Display for Mat4<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..4 {
            write!(f, "[")?;
            for j in 0..4 {
                write!(f, "{}", self[(i, j)])?;
                if j < 4 - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
            if i < 4 - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use proptest::{
        collection::vec,
        num::f32::{Any, INFINITE, NEGATIVE, NORMAL, POSITIVE, QUIET_NAN as NAN, SUBNORMAL, ZERO},
        prop_assert, prop_assert_eq, prop_compose, proptest,
    };

    use super::*;

    prop_compose! {
        pub(crate) fn mat4(s: Any)(v in vec(s, 16)) -> Mat4<f32> {
            Mat4::try_from(v).unwrap()
        }
    }

    proptest! {
        #[test]
        fn mat4_row_returns_a_vec4(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL), b in 0usize..4) {
            prop_assert_eq!(a.row(b), Vec4::new(a[(b, 0)], a[(b, 1)], a[(b, 2)], a[(b, 3)]))
        }

        #[test]
        fn mat4_col_returns_a_vec4(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL), b in 0usize..4) {
            prop_assert_eq!(a.col(b), Vec4::new(a[(0, b)], a[(1, b)], a[(2, b)], a[(3, b)]))
        }

        // NaN Tests
        #[test]
        fn mat4_is_nan_returns_true_for_nan_components(a in mat4(NAN)) {
            prop_assert!(a.is_nan())
        }

        #[test]
        fn mat4_is_nan_returns_false_for_non_nan_components(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            prop_assert!(!a.is_nan())
        }

        #[test]
        fn mat4_transposition_is_its_own_inverse_operation(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO)) {
            let b: Mat4<f32> = a.t();
            prop_assert_eq!(b.t(), a)
        }

        #[test]
        fn mat4_t_flips_non_diagonal_elements(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO)) {
            prop_assert_eq!(a.t(), Mat4::new([
                    [a[(0, 0)], a[(1, 0)], a[(2, 0)], a[(3, 0)]],
                    [a[(0, 1)], a[(1, 1)], a[(2, 1)], a[(3, 1)]],
                    [a[(0, 2)], a[(1, 2)], a[(2, 2)], a[(3, 2)]],
                    [a[(0, 3)], a[(1, 3)], a[(2, 3)], a[(3, 3)]],
            ]))
        }

        #[test]
        fn mat4_diag_returns_vec4_with_diagonal_elements(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            prop_assert_eq!(a.diag(), Vec4::new(a[(0, 0)], a[(1, 1)], a[(2, 2)], a[(3, 3)]));
        }
    }

    #[test]
    fn mat4_provides_zero_constructor() {
        let m: Mat4<f32> = Mat4::zero();
        assert_eq!(m, Mat4([[0.0f32; 4]; 4]));
    }

    #[test]
    fn mat4_supports_one_constructor() {
        let m: Mat4<f32> = Mat4::one();
        assert_eq!(m, Mat4([[1.0f32; 4]; 4]));
    }

    #[test]
    fn mat4_supports_identity_constructor() {
        let m: Mat4<f32> = Mat4::identity();

        let mut expected = Mat4([[0.0f32; 4]; 4]);
        for i in 0..4 {
            expected.0[i][i] = 1.0f32;
        }
        assert_eq!(m, expected);
    }
}
