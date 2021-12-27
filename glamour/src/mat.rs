use std::marker::PhantomData;
use std::ops::{Add, Div, Index, Mul, Sub};
use num_traits::{Num, Zero, One};
use crate::dot::Dot;

pub trait Dim: 'static + PartialEq + Clone + Copy {
    const DIM: usize;

    fn as_usize(&self) -> usize;
}

macro_rules! impl_dim {
    ($name:ident, $dim:literal) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        struct $name;

        impl Dim for $name {
            const DIM: usize = $dim;

            fn as_usize(&self) -> usize {
                Self::DIM
            }
        }
    };
}

impl_dim!(D1, 1);
impl_dim!(D2, 2);
impl_dim!(D3, 3); impl_dim!(D4, 4);

#[derive(Debug, PartialEq)]
pub struct Mat<R, N, M>(pub(crate) Vec<R>, PhantomData<(N, M)>);

impl<R, N, M> From<Vec<R>> for Mat<R, N, M>
where
    N: Dim,
    M: Dim,
{
    fn from(value: Vec<R>) -> Self {
        if value.len() != N::DIM * M::DIM {
            panic!("Expected a vec of {} elements, got {} elements", N::DIM * M::DIM, value.len());
        }
        Mat(value, PhantomData::default())
    }
}

macro_rules! impl_from_array {
    ($($d1:ty, $d2:ty, $total:literal);+ $(;)*) => {
        $(
        impl<R> From<[R; $total]> for Mat<R, $d1, $d2> {
            fn from(value: [R; $total]) -> Self {
                value.into_iter().collect::<Vec<_>>().into()
            }
        }
        )+
    };
}

impl_from_array!(
    D1, D1, 1;
    D1, D2, 2;
    D1, D3, 3;
    D1, D4, 4;
    D2, D1, 2;
    D2, D2, 4;
    D2, D3, 6;
    D2, D4, 8;
    D3, D1, 3;
    D3, D2, 6;
    D3, D3, 9;
    D3, D4, 12;
    D4, D1, 4;
    D4, D2, 8;
    D4, D3, 12;
    D4, D4, 16;
);

impl<R, N: Dim, M: Dim> Index<usize> for Mat<R, N, M> {
    type Output = R;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<R, N: Dim, M: Dim> Index<(usize, usize)> for Mat<R, N, M> {
    type Output = R;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let lin_idx = M::DIM * index.0 + index.1;
        self.0.index(lin_idx)
    }
}

impl<R, N, M> Zero for Mat<R, N, M>
    where
        R: Num + Zero + Copy,
        N: Dim,
        M: Dim,
{
    fn zero() -> Self {
        Mat(vec![R::zero(); N::DIM * M::DIM], PhantomData::default())
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|r| r.is_zero())
    }
}

impl<R> One for Mat<R, D2, D2>
    where
        R: Num + One + Copy,
{
    fn one() -> Self {
        Mat(vec![R::one(), R::zero(), R::zero(), R::one()], PhantomData::default())
    }
}

macro_rules! impl_binops {
    ($($Op:ident, $op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
        impl<R, N: Dim, M: Dim> $Op<R> for Mat<R, N, M>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, N, M>;

            fn $op(self, rhs: R) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'a, R, N: Dim, M: Dim> $Op<&'a R> for &'a Mat<R, N, M>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, N, M>;

            fn $op(self, rhs: &'a R) -> Self::Output {
                let mut data = vec![R::zero(); N::DIM * M::DIM];
                for i in 0..(N::DIM * M::DIM) {
                    data[i] = self[i].$op(*rhs);
                }
                Mat(data, PhantomData::default())
            }
        }

        $(
        impl<N: Dim, M: Dim> $Op<Mat<$tgt, N, M>> for $tgt {
            type Output = Mat<$tgt, N, M>;

            fn $op(self, rhs: Mat<$tgt, N, M>) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'a, N: Dim, M: Dim> $Op<&'a Mat<$tgt, N, M>> for &'a $tgt {
            type Output = Mat<$tgt, N, M>;

            fn $op(self, rhs: &'a Mat<$tgt, N, M>) -> Self::Output {
                let mut data = vec![$tgt::zero(); N::DIM * M::DIM];
                for i in 0..(N::DIM * M::DIM) {
                    data[i] = self.$op(rhs[i]);
                }
                Mat(data, PhantomData::default())
            }
        }
        )*

        )+
    }
}

impl_binops!(
    Add, add, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Sub, sub, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Mul, mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div, div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);

impl<R, N: Dim, M: Dim> Add for Mat<R, N, M>
    where
        R: Num + Copy,
{
    type Output = Mat<R, N, M>;

    fn add(self, rhs: Self) -> Self::Output {
        (&self).add(&rhs)
    }
}

impl<'a, R, N: Dim, M: Dim> Add for &'a Mat<R, N, M>
    where
        R: Num + Copy,
{
    type Output = Mat<R, N, M>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut data = vec![R::zero(); N::DIM * M::DIM];
        for i in 0..(N::DIM * M::DIM) {
            data[i] = self[i] + rhs[i];
        }
        Mat(data, PhantomData::default())
    }
}

impl<R, N: Dim, M: Dim> Sub for Mat<R, N, M>
    where
        R: Num + Copy,
{
    type Output = Mat<R, N, M>;

    fn sub(self, rhs: Self) -> Self::Output {
        (&self).sub(&rhs)
    }
}

impl<'a, R, N: Dim, M: Dim> Sub for &'a Mat<R, N, M>
    where
        R: Num + Copy,
{
    type Output = Mat<R, N, M>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut data = vec![R::zero(); N::DIM * M::DIM];
        for i in 0..(N::DIM * M::DIM) {
            data[i] = self[i] - rhs[i];
        }
        Mat(data, PhantomData::default())
    }
}

impl<'a, R> Mul<&'a Mat<R, D2, D1>> for &'a Mat<R, D1, D2>
    where
        R: Num + Copy,
{
    type Output = R;

    fn mul(self, rhs: &'a Mat<R, D2, D1>) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot<&'a Mat<R, D2, D1>> for &'a Mat<R, D1, D2>
    where
        R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: &'a Mat<R, D2, D1>) -> Self::Output {
        self[0].mul(rhs[0]) + self[1].mul(rhs[1])
    }
}

impl<'a, R> Mul<&'a Mat<R, D1, D2>> for &'a Mat<R, D2, D1>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D2, D2>;

    fn mul(self, rhs: &'a Mat<R, D1, D2>) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot<&'a Mat<R, D1, D2>> for &'a Mat<R, D2, D1>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D2, D2>;

    fn dot(self, rhs: &'a Mat<R, D1, D2>) -> Self::Output {
        Mat(vec![
            self[0] * rhs[0],
            self[0] * rhs[1],
            self[1] * rhs[0],
            self[1] * rhs[1],
        ], PhantomData::default())
    }
}

impl<'a, R> Mul<&'a Mat<R, D2, D2>> for &'a Mat<R, D2, D2>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D2, D2>;

    fn mul(self, rhs: &'a Mat<R, D2, D2>) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot<Self> for &'a Mat<R, D2, D2>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D2, D2>;

    fn dot(self, rhs: Self) -> Self::Output {
        let c: [R; 4] = abop!(crate::dot::Dot::dot, self, rhs, [
            ((0, 1), (0, 2)),
            ((0, 1), (1, 3)),
            ((2, 3), (0, 2)),
            ((2, 3), (1, 3)),
        ]);
        c.into()
    }
}

impl<'a, R> Mul<&'a Mat<R, D3, D3>> for &'a Mat<R, D3, D3>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D3, D3>;

    fn mul(self, rhs: &'a Mat<R, D3, D3>) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot<Self> for &'a Mat<R, D3, D3>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D3, D3>;

    fn dot(self, rhs: Self) -> Self::Output {
        let c: [R; 9] = abop!(crate::dot::Dot::dot, self, rhs, [
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
        c.into()
    }
}

impl<'a, R> Mul<&'a Mat<R, D4, D4>> for &'a Mat<R, D4, D4>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D4, D4>;

    fn mul(self, rhs: &'a Mat<R, D4, D4>) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot<Self> for &'a Mat<R, D4, D4>
    where
        R: Num + Copy,
{
    type Output = Mat<R, D4, D4>;

    fn dot(self, rhs: Self) -> Self::Output {
        let c: [R; 16] = abop!(crate::dot::Dot::dot, self, rhs, [
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
        c.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dx_implements_dim() {
        assert_eq!(D1::DIM, 1usize);
        assert_eq!(D2::DIM, 2usize);
        assert_eq!(D3::DIM, 3usize);
        assert_eq!(D4::DIM, 4usize);
        assert_eq!(D1.as_usize(), 1usize);
        assert_eq!(D2.as_usize(), 2usize);
        assert_eq!(D3.as_usize(), 3usize);
        assert_eq!(D4.as_usize(), 4usize);
    }

    #[test]
    fn mat_implements_from_array() {
        let _: Mat<f32, D1, D1> = Mat::from([0.0f32; 1]);
        let _: Mat<f32, D1, D2> = Mat::from([0.0f32; 2]);
        let _: Mat<f32, D1, D3> = Mat::from([0.0f32; 3]);
        let _: Mat<f32, D1, D4> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, D2, D1> = Mat::from([0.0f32; 2]);
        let _: Mat<f32, D2, D2> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, D2, D3> = Mat::from([0.0f32; 6]);
        let _: Mat<f32, D2, D4> = Mat::from([0.0f32; 8]);
        let _: Mat<f32, D3, D1> = Mat::from([0.0f32; 3]);
        let _: Mat<f32, D3, D2> = Mat::from([0.0f32; 6]);
        let _: Mat<f32, D3, D3> = Mat::from([0.0f32; 9]);
        let _: Mat<f32, D3, D4> = Mat::from([0.0f32; 12]);
        let _: Mat<f32, D4, D1> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, D4, D2> = Mat::from([0.0f32; 8]);
        let _: Mat<f32, D4, D3> = Mat::from([0.0f32; 12]);
        let _: Mat<f32, D4, D4> = Mat::from([0.0f32; 16]);
    }

    #[test]
    fn mat_supports_linear_indexing() {
        let m: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[2], 3.0f32);
    }

    #[test]
    fn mat_supports_2d_indexing() {
        let m: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[(1, 1)], 4.0f32);
    }

    #[test]
    fn mat_supports_additive_identity() {
        let m: Mat<f32, D2, D2> = Mat::zero();
        assert!(m.is_zero());
        assert_eq!(m, Mat::<f32, D2, D2>::from([0.0f32; 4]));

        let m: Mat<f32, D2, D2> = Mat::from([1.0f32; 4]);
        assert!(!m.is_zero());
    }

    #[test]
    fn mat_supports_scalar_addition() {
        let a: Mat<f32, D1, D1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a + &b, Mat::<f32, D1, D1>::from([3.0]));
        assert_eq!(&b + &a, Mat::<f32, D1, D1>::from([3.0]));
    }

    #[test]
    fn mat_supports_scalar_subtraction() {
        let a: Mat<f32, D1, D1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a - &b, Mat::<f32, D1, D1>::from([-1.0]));
        assert_eq!(&b - &a, Mat::<f32, D1, D1>::from([1.0]));
    }

    #[test]
    fn mat_supports_scalar_multiplication() {
        let a: Mat<f32, D1, D1> = Mat::from([2.0]);
        let b: f32 = 2.0;
        assert_eq!(&a * &b, Mat::<f32, D1, D1>::from([4.0]));
        assert_eq!(&b * &a, Mat::<f32, D1, D1>::from([4.0]));
    }

    #[test]
    fn mat_supports_scalar_division() {
        let a: Mat<f32, D1, D1> = Mat::from([6.0]);
        let b: f32 = 2.0;
        assert_eq!(&a / &b, Mat::<f32, D1, D1>::from([3.0]));
        assert_eq!(&b / &a, Mat::<f32, D1, D1>::from([2.0 / 6.0]));
    }

    #[test]
    fn mat_supports_addition() {
        let a: Mat<f32, D1, D1> = Mat::from([3.0]);
        let b: Mat<f32, D1, D1> = Mat::from([2.0]);
        assert_eq!(&a + &b, Mat::<f32, D1, D1>::from([5.0]));
        assert_eq!(&b + &a, Mat::<f32, D1, D1>::from([5.0]));
    }

    #[test]
    fn mat_supports_subtraction() {
        let a: Mat<f32, D1, D1> = Mat::from([3.0]);
        let b: Mat<f32, D1, D1> = Mat::from([2.0]);
        assert_eq!(&a - &b, Mat::<f32, D1, D1>::from([1.0]));
        assert_eq!(&b - &a, Mat::<f32, D1, D1>::from([-1.0]));
    }

    #[test]
    fn mat_supports_dot_product_2x1_1x2() {
        let a: Mat<f32, D2, D1> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, D1, D2> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), Mat::<f32, D2, D2>::from([6.0, 3.0, 4.0, 2.0]));
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x1() {
        let a: Mat<f32, D1, D2> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, D2, D1> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), 8.0f32);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x2() {
        let a: Mat<f32, D2, D2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, D2, D2> = Mat::from([2.0, 3.0, 4.0, 5.0]);
        let c: Mat<f32, D2, D2> = Mat::from([10.0, 13.0, 22.0, 29.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x3() {
        let a: Mat<f32, D3, D3> = Mat::from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let b: Mat<f32, D3, D3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, D3, D3> = Mat::from([36., 42., 48., 81., 96., 111., 126., 150., 174.]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x4() {
        let a: Mat<f32, D4, D4> = Mat::from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]);
        let b: Mat<f32, D4, D4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let c: Mat<f32, D4, D4> = Mat::from([100., 110., 120., 130., 228., 254., 280., 306., 356., 398., 440., 482., 484., 542., 600., 658.]);
        assert_eq!((&a).dot(&b), c);
    }
}
