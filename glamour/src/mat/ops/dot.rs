use crate::ops::dot::Dot;
use crate::abop;
use super::super::Mat;
use std::ops::Mul;
use forward_ref::forward_ref_binop;
use crate::float_and_sum::FloatAndSum;

macro_rules! impl_matmul {
    ($dim:literal, $tt:tt) => {
        impl_matmul!($dim, $dim, $dim, $dim, $tt);
    };
    ($nl:literal, $ml:literal, $nr:literal, $mr:literal, $tt:tt) => {
        impl<'a, 'b, R> Mul<&'b Mat<R, $nr, $mr>> for &'a Mat<R, $nl, $ml>
            where
                R: FloatAndSum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn mul(self, rhs: &'b Mat<R, $nr, $mr>) -> Self::Output {
                self.dot(rhs)
            }
        }

        forward_ref_binop!(impl<R: FloatAndSum> Mul, mul for Mat<R, $nl, $ml>, Mat<R, $nr, $mr>, Mat<R, $nl, $mr>);

        impl<'a, 'b, R> Dot<&'b Mat<R, $nr, $mr>> for &'a Mat<R, $nl, $ml>
        where
            R: FloatAndSum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn dot(self, rhs: &'b Mat<R, $nr, $mr>) -> Self::Output {
                let c = abop!(dot, self, rhs, $tt);
                c.into()
            }
        }

        forward_ref_binop!(impl<R: FloatAndSum> Dot, dot for Mat<R, $nl, $ml>, Mat<R, $nr, $mr>, Mat<R, $nl, $mr>);
    };
}

impl_matmul!(2, 1, 1, 2, [((0), (0)), ((0), (1)), ((1), (0)), ((1), (1))]);

impl_matmul!(
    2,
    [((0, 1), (0, 2)), ((0, 1), (1, 3)), ((2, 3), (0, 2)), ((2, 3), (1, 3)),]
);
impl_matmul!(1, 2, 2, 2, [((0, 1), (0, 2)), ((0, 1), (1, 3))]);
impl_matmul!(2, 2, 2, 1, [((0, 1), (0, 1)), ((2, 3), (0, 1))]);

impl_matmul!(
    3,
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
impl_matmul!(
    1, 3, 3, 3,
    [
        ((0, 1, 2), (0, 3, 6)),
        ((0, 1, 2), (1, 4, 7)),
        ((0, 1, 2), (2, 5, 8)),
    ]
);
impl_matmul!(
    3, 3, 3, 1,
    [
        ((0, 1, 2), (0, 1, 2)),
        ((3, 4, 5), (0, 1, 2)),
        ((6, 7, 8), (0, 1, 2)),
    ]
);

impl_matmul!(
    4,
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
impl_matmul!(
    1, 4, 4, 4,
    [
        ((0, 1, 2, 3), (0, 4, 8, 12)),
        ((0, 1, 2, 3), (1, 5, 9, 13)),
        ((0, 1, 2, 3), (2, 6, 10, 14)),
        ((0, 1, 2, 3), (3, 7, 11, 15)),
    ]
);
impl_matmul!(
    4, 4, 4, 1,
    [
        ((0, 1, 2, 3), (0, 1, 2, 3)),
        ((4, 5, 6, 7), (0, 1, 2, 3)),
        ((8, 9, 10, 11), (0, 1, 2, 3)),
        ((12, 13, 14, 15), (0, 1, 2, 3)),
    ]
);

impl<'a, 'b, R> Mul<&'b Mat<R, 2, 1>> for &'a Mat<R, 1, 2>
where
    R: FloatAndSum
{
    type Output = R;

    fn mul(self, rhs: &'b Mat<R, 2, 1>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: FloatAndSum> Mul, mul for Mat<R, 1, 2>, Mat<R, 2, 1>, R);

/// MARK
impl<'a, 'b, R> Dot<&'b Mat<R, 2, 1>> for &'a Mat<R, 1, 2>
where
    R: FloatAndSum
{
    type Output = R;

    fn dot(self, rhs: &'b Mat<R, 2, 1>) -> Self::Output {
        abop!(dot, self, rhs, [((0, 1), (0, 1))])[0]
    }
}

forward_ref_binop!(impl<R: FloatAndSum> Dot, dot for Mat<R, 1, 2>, Mat<R, 2, 1>, R);

