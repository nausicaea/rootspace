use num_traits::{Float, One, Zero};

mod approx;
mod convert;
pub mod mat2;
pub mod mat3;
pub mod mat4;
mod ops;
mod serde;
pub mod vec2;
pub mod vec3;
pub mod vec4;

pub use self::{mat2::Mat2, mat3::Mat3, mat4::Mat4};
pub use self::{vec2::Vec2, vec3::Vec3, vec4::Vec4};

// Generalized vector, interpreted as column
type Vec_<R, const I: usize> = Mat<R, I, 1>;

/// Generalized matrix type, with data stored in row-major format.
#[derive(Debug, Clone, PartialEq)]
pub struct Mat<R, const I: usize, const J: usize>([[R; J]; I]);

impl<R, const I: usize, const J: usize> Mat<R, I, J> {
    /// Given a one-dimensional array index, return the corresponding two-dimensional indices for
    /// this particular matrix' dimensions
    fn as_2d_idx(idx: usize) -> (usize, usize) {
        (idx / J, idx % J)
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J> {
    pub fn as_slice(&self) -> &[[R; J]; I] {
        &self.0
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
pub(crate) mod tests {
    use proptest::{
        collection::vec,
        num::f32::{Any, INFINITE, NEGATIVE, NORMAL, POSITIVE, QUIET_NAN as NAN, SUBNORMAL, ZERO},
        prop_assert, prop_assert_eq, prop_compose, proptest,
    };

    use super::*;

    macro_rules! impl_mat_strategy {
        ($name:ident, $I:literal, $J:literal) => {
            prop_compose! {
                pub(crate) fn $name(s: Any)(v in vec(s, $I * $J)) -> Mat<f32, $I, $J> {
                    Mat::try_from(v).unwrap()
                }
            }
        };
    }

    impl_mat_strategy!(mat1, 1, 1);
    impl_mat_strategy!(mat1x2, 1, 2);
    impl_mat_strategy!(mat1x3, 1, 3);
    impl_mat_strategy!(mat1x4, 1, 4);
    impl_mat_strategy!(mat2x1, 2, 1);
    impl_mat_strategy!(mat2, 2, 2);
    impl_mat_strategy!(mat2x3, 2, 3);
    impl_mat_strategy!(mat2x4, 2, 4);
    impl_mat_strategy!(mat3x1, 3, 1);
    impl_mat_strategy!(mat3x2, 3, 2);
    impl_mat_strategy!(mat3, 3, 3);
    impl_mat_strategy!(mat3x4, 3, 4);
    impl_mat_strategy!(mat4x1, 4, 1);
    impl_mat_strategy!(mat4x2, 4, 2);
    impl_mat_strategy!(mat4x3, 4, 3);
    impl_mat_strategy!(mat4, 4, 4);

    /// [Row](crate::mat::Mat::row) Tests
    mod row {
        use super::*;

        macro_rules! impl_row_test {
            ($name:ident, $strat:expr, $dims:literal, [$($i:literal),+ $(,)*]) => {
                proptest! {
                    #[test]
                    fn $name(a in $strat, b in 0usize..$dims) {
                        prop_assert_eq!(a.row(b), Mat([[$(a[(b, $i)]),+]]));
                    }
                }
            };
        }

        impl_row_test!(
            mat1_row_returns_a_1x1_matrix,
            mat1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0]
        );
        impl_row_test!(
            mat1x2_row_returns_a_1x2_matrix,
            mat1x2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0, 1]
        );
        impl_row_test!(
            mat1x3_row_returns_a_1x3_matrix,
            mat1x3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0, 1, 2]
        );
        impl_row_test!(
            mat1x4_row_returns_a_1x4_matrix,
            mat1x4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0, 1, 2, 3]
        );
        impl_row_test!(
            mat2x1_row_returns_a_1x1_matrix,
            mat2x1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0]
        );
        impl_row_test!(
            mat2_row_returns_a_1x2_matrix,
            mat2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0, 1]
        );
        impl_row_test!(
            mat2x3_row_returns_a_1x3_matrix,
            mat2x3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0, 1, 2]
        );
        impl_row_test!(
            mat2x4_row_returns_a_1x4_matrix,
            mat2x4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0, 1, 2, 3]
        );
        impl_row_test!(
            mat3x1_row_returns_a_1x1_matrix,
            mat3x1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0]
        );
        impl_row_test!(
            mat3x2_row_returns_a_1x2_matrix,
            mat3x2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0, 1]
        );
        impl_row_test!(
            mat3_row_returns_a_1x3_matrix,
            mat3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0, 1, 2]
        );
        impl_row_test!(
            mat3x4_row_returns_a_1x4_matrix,
            mat3x4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0, 1, 2, 3]
        );
        impl_row_test!(
            mat4x1_row_returns_a_1x1_matrix,
            mat4x1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0]
        );
        impl_row_test!(
            mat4x2_row_returns_a_1x2_matrix,
            mat4x2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0, 1]
        );
        impl_row_test!(
            mat4x3_row_returns_a_1x3_matrix,
            mat4x3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0, 1, 2]
        );
        impl_row_test!(
            mat4_row_returns_a_1x4_matrix,
            mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0, 1, 2, 3]
        );
    }

    /// [Column](crate::mat::Mat::col) Tests
    mod col {
        use super::*;

        macro_rules! impl_col_test {
            ($name:ident, $strat:expr, $dims:literal, [$($i:literal),+ $(,)*]) => {
                proptest! {
                    #[test]
                    fn $name(a in $strat, b in 0usize..$dims) {
                        prop_assert_eq!(a.col(b), Mat([$([a[($i, b)]]),+]));
                    }
                }
            };
        }

        impl_col_test!(
            mat1_col_returns_a_1x1_matrix,
            mat1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0]
        );
        impl_col_test!(
            mat1x2_col_returns_a_1x1_matrix,
            mat1x2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0]
        );
        impl_col_test!(
            mat1x3_col_returns_a_1x1_matrix,
            mat1x3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0]
        );
        impl_col_test!(
            mat1x4_col_returns_a_1x1_matrix,
            mat1x4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0]
        );
        impl_col_test!(
            mat2x1_col_returns_a_2x1_matrix,
            mat2x1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0, 1]
        );
        impl_col_test!(
            mat2_col_returns_a_2x1_matrix,
            mat2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0, 1]
        );
        impl_col_test!(
            mat2x3_col_returns_a_2x1_matrix,
            mat2x3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0, 1]
        );
        impl_col_test!(
            mat2x4_col_returns_a_2x1_matrix,
            mat2x4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0, 1]
        );
        impl_col_test!(
            mat3x1_col_returns_a_3x1_matrix,
            mat3x1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0, 1, 2]
        );
        impl_col_test!(
            mat3x2_col_returns_a_3x1_matrix,
            mat3x2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0, 1, 2]
        );
        impl_col_test!(
            mat3_col_returns_a_3x1_matrix,
            mat3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0, 1, 2]
        );
        impl_col_test!(
            mat3x4_col_returns_a_3x1_matrix,
            mat3x4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0, 1, 2]
        );
        impl_col_test!(
            mat4x1_col_returns_a_4x1_matrix,
            mat4x1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            1,
            [0, 1, 2, 3]
        );
        impl_col_test!(
            mat4x2_col_returns_a_4x1_matrix,
            mat4x2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            2,
            [0, 1, 2, 3]
        );
        impl_col_test!(
            mat4x3_col_returns_a_4x1_matrix,
            mat4x3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            3,
            [0, 1, 2, 3]
        );
        impl_col_test!(
            mat4_col_returns_a_4x1_matrix,
            mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL),
            4,
            [0, 1, 2, 3]
        );
    }

    proptest! {
        // NaN Tests
        #[test]
        fn mat2_is_nan_returns_true_for_nan_components(a in mat2(NAN)) {
            prop_assert!(a.is_nan());
        }

        #[test]
        fn mat2_is_nan_returns_false_for_non_nan_components(a in mat2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            prop_assert!(!a.is_nan());
        }

        // Subset Tests
        #[test]
        fn mat4_provides_subset_method_that_selects_a_submatrix(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            prop_assert_eq!(a.subset::<2, 2>(0, 1), Mat([[a[(0, 1)], a[(0, 2)]], [a[(1, 1)], a[(1, 2)]]]));
        }
    }

    // Transposition Tests
    mod t {
        use super::*;

        macro_rules! impl_t_inverse_test {
            ($name:ident, $strat:expr, $I:literal, $J:literal) => {
                proptest! {
                    #[test]
                    fn $name(a in $strat) {
                        let b: Mat<f32, $J, $I> = a.t();
                        prop_assert_eq!(b.t(), a);
                    }
                }
            };
        }

        impl_t_inverse_test!(
            mat1_transposition_is_its_own_inverse_operation,
            mat1(NORMAL | POSITIVE | NEGATIVE | ZERO),
            1,
            1
        );
        impl_t_inverse_test!(
            mat1x2_transposition_is_its_own_inverse_operation,
            mat1x2(NORMAL | POSITIVE | NEGATIVE | ZERO),
            1,
            2
        );
        impl_t_inverse_test!(
            mat1x3_transposition_is_its_own_inverse_operation,
            mat1x3(NORMAL | POSITIVE | NEGATIVE | ZERO),
            1,
            3
        );
        impl_t_inverse_test!(
            mat1x4_transposition_is_its_own_inverse_operation,
            mat1x4(NORMAL | POSITIVE | NEGATIVE | ZERO),
            1,
            4
        );
        impl_t_inverse_test!(
            mat2x1_transposition_is_its_own_inverse_operation,
            mat2x1(NORMAL | POSITIVE | NEGATIVE | ZERO),
            2,
            1
        );
        impl_t_inverse_test!(
            mat2_transposition_is_its_own_inverse_operation,
            mat2(NORMAL | POSITIVE | NEGATIVE | ZERO),
            2,
            2
        );
        impl_t_inverse_test!(
            mat2x3_transposition_is_its_own_inverse_operation,
            mat2x3(NORMAL | POSITIVE | NEGATIVE | ZERO),
            2,
            3
        );
        impl_t_inverse_test!(
            mat2x4_transposition_is_its_own_inverse_operation,
            mat2x4(NORMAL | POSITIVE | NEGATIVE | ZERO),
            2,
            4
        );
        impl_t_inverse_test!(
            mat3x1_transposition_is_its_own_inverse_operation,
            mat3x1(NORMAL | POSITIVE | NEGATIVE | ZERO),
            3,
            1
        );
        impl_t_inverse_test!(
            mat3x2_transposition_is_its_own_inverse_operation,
            mat3x2(NORMAL | POSITIVE | NEGATIVE | ZERO),
            3,
            2
        );
        impl_t_inverse_test!(
            mat3_transposition_is_its_own_inverse_operation,
            mat3(NORMAL | POSITIVE | NEGATIVE | ZERO),
            3,
            3
        );
        impl_t_inverse_test!(
            mat3x4_transposition_is_its_own_inverse_operation,
            mat3x4(NORMAL | POSITIVE | NEGATIVE | ZERO),
            3,
            4
        );
        impl_t_inverse_test!(
            mat4x1_transposition_is_its_own_inverse_operation,
            mat4x1(NORMAL | POSITIVE | NEGATIVE | ZERO),
            4,
            1
        );
        impl_t_inverse_test!(
            mat4x2_transposition_is_its_own_inverse_operation,
            mat4x2(NORMAL | POSITIVE | NEGATIVE | ZERO),
            4,
            2
        );
        impl_t_inverse_test!(
            mat4x3_transposition_is_its_own_inverse_operation,
            mat4x3(NORMAL | POSITIVE | NEGATIVE | ZERO),
            4,
            3
        );
        impl_t_inverse_test!(
            mat4_transposition_is_its_own_inverse_operation,
            mat4(NORMAL | POSITIVE | NEGATIVE | ZERO),
            4,
            4
        );

        proptest! {
            #[test]
            fn mat2_t_flips_non_diagonal_elements(a in mat2(NORMAL | POSITIVE | NEGATIVE | ZERO)) {
                prop_assert_eq!(a.t(), Mat([[a[(0, 0)], a[(1, 0)]], [a[(0, 1)], a[(1, 1)]]]));
            }
        }
    }

    // Diagonal Tests (for square matrices)
    mod diag {
        use super::*;

        proptest! {
            #[test]
            fn mat1_diag_returns_a_1x1_matrix_with_diagonal_elements(a in mat1(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
                prop_assert_eq!(a.diag(), Mat([[a[(0, 0)]]]));
            }

            #[test]
            fn mat2_diag_returns_a_2x1_matrix_with_diagonal_elements(a in mat2(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
                prop_assert_eq!(a.diag(), Mat([[a[(0, 0)]], [a[(1, 1)]]]));
            }

            #[test]
            fn mat3_diag_returns_a_3x1_matrix_with_diagonal_elements(a in mat3(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
                prop_assert_eq!(a.diag(), Mat([[a[(0, 0)]], [a[(1, 1)]], [a[(2, 2)]]]));
            }

            #[test]
            fn mat4_diag_returns_a_4x1_matrix_with_diagonal_elements(a in mat4(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
                prop_assert_eq!(a.diag(), Mat([[a[(0, 0)]], [a[(1, 1)]], [a[(2, 2)]], [a[(3, 3)]]]));
            }
        }
    }

    mod as_2d_idx {
        use super::*;

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
    fn mat_provides_zero_constructor_for_all_matrices() {
        fn test<const I: usize, const J: usize>() {
            let m: Mat<f32, I, J> = Mat::zero();
            assert_eq!(m, Mat([[0.0f32; J]; I]));
        }

        test::<1, 1>();
        test::<1, 2>();
        test::<1, 3>();
        test::<1, 4>();
        test::<2, 1>();
        test::<2, 2>();
        test::<2, 3>();
        test::<2, 4>();
        test::<3, 1>();
        test::<3, 2>();
        test::<3, 3>();
        test::<3, 4>();
        test::<4, 1>();
        test::<4, 2>();
        test::<4, 3>();
        test::<4, 4>();
    }

    #[test]
    fn mat_supports_one_constructor_for_all_matrices() {
        fn test<const I: usize, const J: usize>() {
            let m: Mat<f32, I, J> = Mat::one();
            assert_eq!(m, Mat([[1.0f32; J]; I]));
        }

        test::<1, 1>();
        test::<1, 2>();
        test::<1, 3>();
        test::<1, 4>();
        test::<2, 1>();
        test::<2, 2>();
        test::<2, 3>();
        test::<2, 4>();
        test::<3, 1>();
        test::<3, 2>();
        test::<3, 3>();
        test::<3, 4>();
        test::<4, 1>();
        test::<4, 2>();
        test::<4, 3>();
        test::<4, 4>();
    }

    #[test]
    fn mat_supports_identity_constructor_for_square_matrices() {
        fn test<const I: usize>() {
            let m: Mat<f32, I, I> = Mat::identity();
            let mut e = Mat([[0.0f32; I]; I]);
            for i in 0..I {
                e.0[i][i] = 1.0f32;
            }
            assert_eq!(m, e);
        }

        test::<1>();
        test::<2>();
        test::<3>();
        test::<4>();
    }
}
