/// Forward an implementation of a binary operator to combinations of references
#[macro_export]
macro_rules! forward_ref_binop {
    (impl $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl $Op<$Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b> $Op<&'b $Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a> $Op<$Rhs> for &'a $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }
    };
    (impl<$(const $N:ident: usize),+ $(,)*> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$(const $N: usize),+> $Op<$Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, $(const $N: usize),+> $Op<&'b $Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, $(const $N: usize),+> $Op<$Rhs> for &'a $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }
    };
    (impl<$R:ident> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$R> $Op<$Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, $R> $Op<&'b $Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, $R> $Op<$Rhs> for &'a $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }
    };
    (impl<$R:ident: $Bound:ident> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$R> $Op<$Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, $R> $Op<&'b $Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, $R> $Op<$Rhs> for &'a $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }
    };
    (impl<$R:ident: $Bound:ident, $(const $N:ident: usize),+ $(,)*> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$R, $(const $N: usize),+> $Op<$Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, $R, $(const $N: usize),+> $Op<&'b $Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, $R, $(const $N: usize),+> $Op<$Rhs> for &'a $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                self.$op(&rhs)
            }
        }
    };
}

/// Apply a binary operation to arrays (or anything indexable, really)
#[macro_export]
macro_rules! abop {
    // Support right-sided scalar binary operations
    ($op:ident, $lhs:expr, [$($i:literal),+ $(,)*], $rhs:expr) => {
        [$($lhs[$i].$op($rhs)),+]
    };
    // Support left-sided scalar binary operations
    ($op:ident, $lhs:expr, $rhs:expr, [$($j:literal),+ $(,)*]) => {
        [$($lhs.$op($rhs[$j])),+]
    };
    // Support both-sided element-wise matrix binary operations
    ($op:ident, $lhs:expr, $rhs:expr, [$(($i:literal, $j:literal)),+ $(,)*]) => {
        [$($lhs[$i].$op($rhs[$j])),+]
    };
    // Support both-sided Nx1 X 1xM binary operations
    ($op:ident, $lhs:expr, $rhs:expr, [$((($i0:literal), ($j0:literal))),+ $(,)*]) => {
        [$(
            [$lhs[$i0]].$op([$rhs[$j0]])
        ),+]
    };
    // Support both-sided Nx2 X 2xM binary operations
    ($op:ident, $lhs:expr, $rhs:expr, [$((($i0:literal, $i1:literal), ($j0:literal, $j1:literal))),+ $(,)*]) => {
        [$(
            [$lhs[$i0], $lhs[$i1]].$op([$rhs[$j0], $rhs[$j1]])
        ),+]
    };
    // Support both-sided Nx3 X 3xM binary operations
    ($op:ident, $lhs:expr, $rhs:expr, [$((($i0:literal, $i1:literal, $i2:literal), ($j0:literal, $j1:literal, $j2:literal))),+ $(,)*]) => {
        [$(
            [$lhs[$i0], $lhs[$i1], $lhs[$i2]].$op([$rhs[$j0], $rhs[$j1], $rhs[$j2]])
        ),+]
    };
    // Support both-sided Nx4 X 4xM binary operations
    ($op:ident, $lhs:expr, $rhs:expr, [$((($i0:literal, $i1:literal, $i2:literal, $i3:literal), ($j0:literal, $j1:literal, $j2:literal, $j3:literal))),+ $(,)*]) => {
        [$(
            [$lhs[$i0], $lhs[$i1], $lhs[$i2], $lhs[$i3]].$op([$rhs[$j0], $rhs[$j1], $rhs[$j2], $rhs[$j3]])
        ),+]
    };
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use crate::dot::Dot;

    #[test]
    fn abop_operates_correctly_for_two_arrays() {
        let lhs = [1.0; 4];
        let rhs = [2.0; 4];

        let c: [f64; 4] = abop!(add, lhs, rhs, [(0, 0), (1, 1), (2, 2), (3, 3)]);
        assert_eq!(c, [3.0f64; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_an_array_and_a_scalar() {
        let lhs = [1.0; 4];
        let rhs = 2.0f64;

        let c: [f64; 4] = abop!(add, lhs, [0, 1, 2, 3], rhs);
        assert_eq!(c, [3.0f64; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_a_scalar_and_an_array() {
        let lhs = 2.0f64;
        let rhs = [1.0; 4];

        let c: [f64; 4] = abop!(add, lhs, rhs, [0, 1, 2, 3]);
        assert_eq!(c, [3.0f64; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_2x2_binary_operations() {
        let lhs = [1.0; 4];
        let rhs = [2.0; 4];
        let c: [f64; 4] = abop!(
            dot,
            lhs,
            rhs,
            [((0, 1), (0, 2)), ((0, 1), (1, 3)), ((2, 3), (0, 2)), ((2, 3), (1, 3)),]
        );
        assert_eq!(c, [4.0f64; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_2x1_binary_operations() {
        let lhs = [1.0; 4];
        let rhs = [2.0; 2];
        let c: [f64; 2] = abop!(dot, lhs, rhs, [((0, 1), (0, 1)), ((2, 3), (0, 1)),]);
        assert_eq!(c, [4.0f64; 2]);
    }

    #[test]
    fn abop_operates_correctly_for_3x3_binary_operations() {
        let lhs = [1.0; 9];
        let rhs = [2.0; 9];
        let c: [f64; 9] = abop!(
            dot,
            lhs,
            rhs,
            [
                ((0, 1, 2), (0, 3, 6)),
                ((0, 1, 2), (1, 4, 7)),
                ((0, 1, 2), (2, 5, 8)),
                ((3, 4, 5), (0, 3, 6)),
                ((3, 4, 5), (1, 4, 7)),
                ((3, 4, 5), (2, 5, 8)),
                ((6, 7, 8), (0, 3, 6)),
                ((6, 7, 8), (1, 4, 7)),
                ((6, 7, 8), (2, 5, 8)),
            ]
        );
        assert_eq!(c, [6.0f64; 9]);
    }

    #[test]
    fn abop_operates_correctly_for_4x4_binary_operations() {
        let lhs = [1.0; 16];
        let rhs = [2.0; 16];
        let c: [f64; 16] = abop!(
            dot,
            lhs,
            rhs,
            [
                ((0, 1, 2, 3), (0, 4, 8, 12)),
                ((0, 1, 2, 3), (1, 5, 9, 13)),
                ((0, 1, 2, 3), (2, 6, 10, 14)),
                ((0, 1, 2, 3), (3, 7, 11, 15)),
                ((4, 5, 6, 7), (0, 4, 8, 12)),
                ((4, 5, 6, 7), (1, 5, 9, 13)),
                ((4, 5, 6, 7), (2, 6, 10, 14)),
                ((4, 5, 6, 7), (3, 7, 11, 15)),
                ((8, 9, 10, 11), (0, 4, 8, 12)),
                ((8, 9, 10, 11), (1, 5, 9, 13)),
                ((8, 9, 10, 11), (2, 6, 10, 14)),
                ((8, 9, 10, 11), (3, 7, 11, 15)),
                ((12, 13, 14, 15), (0, 4, 8, 12)),
                ((12, 13, 14, 15), (1, 5, 9, 13)),
                ((12, 13, 14, 15), (2, 6, 10, 14)),
                ((12, 13, 14, 15), (3, 7, 11, 15)),
            ]
        );
        assert_eq!(c, [8.0; 16]);
    }
}
