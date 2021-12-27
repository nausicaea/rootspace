#[macro_export]
macro_rules! impl_vector {
    ($name:ident, $dims:literal, [$($field:ident),+ $(,)*]) => {
        #[derive(Debug, PartialEq)]
        pub struct $name<N>(pub(crate) [N; $dims]);

        impl<N> $name<N> {
            pub fn new($($field: N),+) -> Self {
                $name([$($field),+])
            }
        }

        impl<N> From<[N; $dims]> for $name<N> {
            fn from(value: [N; $dims]) -> Self {
                $name(value)
            }
        }

        impl<N> std::ops::Index<usize> for $name<N> {
            type Output = N;

            fn index(&self, index: usize) -> &Self::Output {
                self.0.index(index)
            }
        }

        impl<'a, N> std::ops::Add for &'a $name<N>
        where
            N: num_traits::Num + Copy,
        {
            type Output = $name<N>;

            fn add(self, rhs: Self) -> Self::Output {
                let mut data = [N::zero(); $dims];
                for i in 0..$dims {
                    data[i] = self.0[i].add(rhs.0[i]);
                }
                $name(data)
            }
        }

        impl<'a, N> std::ops::Sub for &'a $name<N>
        where
            N: num_traits::Num + Copy,
        {
            type Output = $name<N>;

            fn sub(self, rhs: Self) -> Self::Output {
                let mut data = [N::zero(); $dims];
                for i in 0..$dims {
                    data[i] = self.0[i].sub(rhs.0[i]);
                }
                $name(data)
            }
        }

        impl<'a, N> std::ops::Add<N> for &'a $name<N>
        where
            N: num_traits::Num + Copy,
        {
            type Output = $name<N>;

            fn add(self, rhs: N) -> Self::Output {
                let mut data = [N::zero(); $dims];
                for i in 0..$dims {
                    data[i] = self.0[i].add(rhs);
                }
                $name(data)
            }
        }

        impl<'a, N> std::ops::Sub<N> for &'a $name<N>
        where
            N: num_traits::Num + Copy,
        {
            type Output = $name<N>;

            fn sub(self, rhs: N) -> Self::Output {
                let mut data = [N::zero(); $dims];
                for i in 0..$dims {
                    data[i] = self.0[i].sub(rhs);
                }
                $name(data)
            }
        }

        impl<'a, N> std::ops::Mul<N> for &'a $name<N>
        where
            N: num_traits::Num + Copy,
        {
            type Output = $name<N>;

            fn mul(self, rhs: N) -> Self::Output {
                let mut data = [N::zero(); $dims];
                for i in 0..$dims {
                    data[i] = self.0[i].mul(rhs);
                }
                $name(data)
            }
        }

        impl<'a, N> std::ops::Div<N> for &'a $name<N>
        where
            N: num_traits::Num + Copy,
        {
            type Output = $name<N>;

            fn div(self, rhs: N) -> Self::Output {
                let mut data = [N::zero(); $dims];
                for i in 0..$dims {
                    data[i] = self.0[i].div(rhs);
                }
                $name(data)
            }
        }

        impl<'a, N> $crate::dot::Dot for &'a $name<N>
        where
            N: num_traits::Num + Copy + std::iter::Sum,
        {
            type Output = N;

            fn dot(self, rhs: Self) -> Self::Output {
                self.0.iter()
                    .zip(rhs.0.iter())
                    .map(|(l, r)| *l * *r)
                    .sum()
            }
        }

        impl<N> $crate::dot::Dot for $name<N>
        where
            N: num_traits::Num + Copy + std::iter::Sum,
        {
            type Output = N;

            fn dot(self, rhs: Self) -> Self::Output {
                $crate::dot::Dot::dot(&self, &rhs)
            }
        }
    };
}

/// Apply a binary operation to arrays (or anything indexable, really)
#[macro_export]
macro_rules! abop {
    // Support right-sided scalar binary operations
    ($op:path, $lhs:expr, [$($i:literal),+ $(,)*], $rhs:expr) => {
        [$($op($lhs[$i], $rhs)),+]
    };
    // Support left-sided scalar binary operations
    ($op:path, $lhs:expr, $rhs:expr, [$($j:literal),+ $(,)*]) => {
        [$($op($lhs, $rhs[$j])),+]
    };
    // Support both-sided element-wise matrix binary operations
    ($op:path, $lhs:expr, $rhs:expr, [$(($i:literal, $j:literal)),+ $(,)*]) => {
        [$($op($lhs[$i], $rhs[$j])),+]
    };
    // Support both-sided Nx2 X 2xM binary operations
    ($op:path, $lhs:expr, $rhs:expr, [$((($i0:literal, $i1:literal), ($j0:literal, $j1:literal))),+ $(,)*]) => {
        [$(
            $op(
                [$lhs[$i0], $lhs[$i1]],
                [$rhs[$j0], $rhs[$j1]],
            )
        ),+]
    };
    // Support both-sided Nx3 X 3xM binary operations
    ($op:path, $lhs:expr, $rhs:expr, [$((($i0:literal, $i1:literal, $i2:literal), ($j0:literal, $j1:literal, $j2:literal))),+ $(,)*]) => {
        [$(
            $op(
                [$lhs[$i0], $lhs[$i1], $lhs[$i2]],
                [$rhs[$j0], $rhs[$j1], $rhs[$j2]],
            )
        ),+]
    };
    // Support both-sided Nx4 X 4xM binary operations
    ($op:path, $lhs:expr, $rhs:expr, [$((($i0:literal, $i1:literal, $i2:literal, $i3:literal), ($j0:literal, $j1:literal, $j2:literal, $j3:literal))),+ $(,)*]) => {
        [$(
            $op(
                [$lhs[$i0], $lhs[$i1], $lhs[$i2], $lhs[$i3]],
                [$rhs[$j0], $rhs[$j1], $rhs[$j2], $rhs[$j3]],
            )
        ),+]
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn abop_operates_correctly_for_two_arrays() {
        let lhs = [1.0; 4];
        let rhs = [2.0; 4];

        let c: [f32; 4] = abop!(std::ops::Add::add, lhs, rhs, [(0, 0), (1, 1), (2, 2), (3, 3)]);
        assert_eq!(c, [3.0f32; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_an_array_and_a_scalar() {
        let lhs = [1.0; 4];
        let rhs = 2.0f32;

        let c: [f32; 4] = abop!(std::ops::Add::add, lhs, [0, 1, 2, 3], rhs);
        assert_eq!(c, [3.0f32; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_a_scalar_and_an_array() {
        let lhs = 2.0f32;
        let rhs = [1.0; 4];

        let c: [f32; 4] = abop!(std::ops::Add::add, lhs, rhs, [0, 1, 2, 3]);
        assert_eq!(c, [3.0f32; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_2x2_binary_operations() {
        let lhs = [1.0; 4];
        let rhs = [2.0; 4];
        let c: [f32; 4] = abop!(crate::dot::Dot::dot, lhs, rhs, [
            ((0, 1), (0, 2)),
            ((0, 1), (1, 3)),
            ((2, 3), (0, 2)),
            ((2, 3), (1, 3)),
        ]);
        assert_eq!(c, [4.0f32; 4]);
    }

    #[test]
    fn abop_operates_correctly_for_2x1_binary_operations() {
        let lhs = [1.0; 4];
        let rhs = [2.0; 2];
        let c: [f32; 2] = abop!(crate::dot::Dot::dot, lhs, rhs, [
            ((0, 1), (0, 1)),
            ((2, 3), (0, 1)),
        ]);
        assert_eq!(c, [4.0f32; 2]);
    }


    #[test]
    fn abop_operates_correctly_for_3x3_binary_operations() {
        let lhs = [1.0; 9];
        let rhs = [2.0; 9];
        let c: [f32; 9] = abop!(crate::dot::Dot::dot, lhs, rhs, [
            ((0, 1, 2), (0, 3, 6)),
            ((0, 1, 2), (1, 4, 7)),
            ((0, 1, 2), (2, 5, 8)),
            ((3, 4, 5), (0, 3, 6)),
            ((3, 4, 5), (1, 4, 7)),
            ((3, 4, 5), (2, 5, 8)),
            ((6, 7, 8), (0, 3, 6)),
            ((6, 7, 8), (1, 4, 7)),
            ((6, 7, 8), (2, 5, 8)),
        ]);
        assert_eq!(c, [6.0f32; 9]);
    }

    #[test]
    fn abop_operates_correctly_for_4x4_binary_operations() {
        let lhs = [1.0; 16];
        let rhs = [2.0; 16];
        let c: [f32; 16] = abop!(crate::dot::Dot::dot, lhs, rhs, [
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
        ]);
        assert_eq!(c, [8.0f32; 16]);
    }
}
