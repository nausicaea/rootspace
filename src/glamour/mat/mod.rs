use num_traits::Float;

use crate::glamour::{num::Zero, ops::cross::Cross, unit::Unit, vec::Vec4};

#[cfg(test)]
mod approx;
#[cfg(test)]
mod cmp;
mod convert;
mod num;
mod ops;
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
    R: Copy,
{
    /// Return a transposed copy
    pub fn t(&self) -> Mat4<R> {
        Mat4::new([
            [self[(0, 0)], self[(1, 0)], self[(2, 0)], self[(3, 0)]],
            [self[(0, 1)], self[(1, 1)], self[(2, 1)], self[(3, 1)]],
            [self[(0, 2)], self[(1, 2)], self[(2, 2)], self[(3, 2)]],
            [self[(0, 3)], self[(1, 3)], self[(2, 3)], self[(3, 3)]],
        ])
    }
}

impl<R> Mat4<R>
where
    R: Float,
{
    pub fn look_at_lh(fwd: Unit<Vec4<R>>, up: Unit<Vec4<R>>) -> Self {
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

impl<R> std::fmt::Display for Mat4<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let prettyprint = f.alternate();

        write!(f, "[")?;
        if prettyprint {
            writeln!(f)?;
        }
        for i in 0..4 {
            write!(f, "[")?;
            for j in 0..4 {
                write!(f, "{}", self[(i, j)])?;
                if j < 4 - 1 {
                    write!(f, ", ")?;
                    if prettyprint {
                        write!(f, "\t")?;
                    }
                }
            }
            write!(f, "]")?;
            if i < 4 - 1 {
                write!(f, ", ")?;
            }
            if prettyprint {
                writeln!(f)?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use proptest::{
        num::f32::{INFINITE, NEGATIVE, NORMAL, POSITIVE, QUIET_NAN as NAN, SUBNORMAL, ZERO},
        prop_assert, prop_assert_eq, proptest,
    };

    use super::*;
    use crate::glamour::test_helpers::mat4;

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
        fn mat4_diag_returns_vec4_with_diagonal_elements(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            prop_assert_eq!(a.diag(), Vec4::new(a[(0, 0)], a[(1, 1)], a[(2, 2)], a[(3, 3)]));
        }

        #[test]
        fn mat4_t_returns_the_transpose(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            let mut mat = Mat4::zero();
            for i in 0..4 {
                for j in 0..4 {
                    mat[(i, j)] = a[(j, i)];
                }
            }
            prop_assert_eq!(a.t(), mat);
        }
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
