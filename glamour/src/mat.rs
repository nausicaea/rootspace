use std::{
    marker::PhantomData,
    ops::{Add, Div, Index, IndexMut, Mul, Sub},
    iter::Sum,
};

use num_traits::{Num, One, Zero, Float};

use crate::{
    dim::{Dim, D1, D2, D3, D4},
    dot::Dot,
    abop,
};

pub type Vec2<R> = Mat<R, D1, D2>;
pub type Vec3<R> = Mat<R, D1, D3>;
pub type Vec4<R> = Mat<R, D1, D4>;
pub type Mat2<R> = Mat<R, D2, D2>;
pub type Mat3<R> = Mat<R, D3, D3>;
pub type Mat4<R> = Mat<R, D4, D4>;

fn as_lin_idx<N: Dim, M: Dim>(n: usize, m: usize) -> usize {
    n * M::DIM + m
}

fn as_2d_idx<N: Dim, M: Dim>(idx: usize) -> (usize, usize) {
    (idx / N::DIM, idx % N::DIM)
}

#[derive(Debug, PartialEq)]
pub struct Mat<R, N, M>(pub(crate) Vec<R>, PhantomData<(N, M)>);

impl<R, N, M> Mat<R, N, M> 
where
    R: Copy + Zero,
    N: Dim,
    M: Dim,
{
    pub fn col(&self, m: usize) -> Mat<R, N, D1> {
        if m >= M::DIM {
            panic!("Expected a column index smaller than {}, got {}", M::DIM, m);
        }
        let mut data = vec![R::zero(); N::DIM];
        for n in 0..N::DIM {
            data[n] = self[(n, m)];
        }
        Mat(data, PhantomData::default())
    }

    pub fn row(&self, n: usize) -> Mat<R, D1, M> {
        if n >= N::DIM {
            panic!("Expected a row index smaller than {}, got {}", N::DIM, n);
        }
        let mut data = vec![R::zero(); M::DIM];
        for m in 0..M::DIM {
            data[m] = self[(n, m)];
        }
        Mat(data, PhantomData::default())
    }
}

impl<R, N, M> Mat<R, N, M> 
where
    R: Float + Sum,
{
    pub fn norm(&self) -> R {
        self.0.iter().map(|e| e.powi(2)).sum::<R>().sqrt()
    }
}

impl<R, N, M> Mat<R, N, M>
where
    R: Zero + Copy,
    N: Dim,
    M: Dim,
{
    pub fn t(&self) -> Mat<R, M, N> {
        let mut data = vec![R::zero(); M::DIM * N::DIM];
        for i in 0..(N::DIM * M::DIM) {
            let (n_i, m_i) = as_2d_idx::<N, M>(i);
            let j = as_lin_idx::<M, N>(m_i, n_i);
            data[j] = self[i];
        }
        Mat(data, PhantomData::default())
    }
}

impl<R, N, M> Mat<R, N, M>
where
    R: Float,
    N: Dim,
    M: Dim,
{
    pub fn is_nan(&self) -> bool {
        self.0.iter().any(|e| e.is_nan())
    }
}


impl<R, N, M> Mat<R, N, M>
where
    R: Zero + Copy,
    N: Dim,
    M: Dim,
{
    pub fn zero() -> Self {
        Mat(vec![R::zero(); N::DIM * M::DIM], PhantomData::default())
    }
}

impl<R, N, M> Mat<R, N, M>
where
    R: One + Copy,
    N: Dim,
    M: Dim,
{
    pub fn one() -> Self {
        Mat(vec![R::one(); N::DIM * M::DIM], PhantomData::default())
    }
}

impl<R, N> Mat<R, N, N>
where
    R: Zero + One + Copy,
    N: Dim,
{
    pub fn identity() -> Self {
        let mut data = vec![R::zero(); N::DIM * N::DIM];
        for n in 0..N::DIM {
            let i = as_lin_idx::<N, N>(n, n);
            data[i] = R::one();
        }

        Mat(data, PhantomData::default())
    }
}

impl<R, N, M> std::fmt::Display for Mat<R, N, M>
where
    R: std::fmt::Display,
    N: Dim,
    M: Dim,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for n in 0..N::DIM {
            write!(f, "[")?;
            for m in 0..M::DIM {
                write!(f, "{}", self[(n, m)])?;
                if m < M::DIM - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
            if n < N::DIM - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl<R, N, M> From<Vec<R>> for Mat<R, N, M>
where
    N: Dim,
    M: Dim,
{
    fn from(value: Vec<R>) -> Self {
        if value.len() != N::DIM * M::DIM {
            panic!(
                "Expected a vec of {} elements, got {} elements",
                N::DIM * M::DIM,
                value.len()
            );
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

impl<R, N: Dim, M: Dim> IndexMut<usize> for Mat<R, N, M> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<R, N: Dim, M: Dim> Index<(usize, usize)> for Mat<R, N, M> {
    type Output = R;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let i = as_lin_idx::<N, M>(index.0, index.1);
        self.0.index(i)
    }
}

impl<R, N: Dim, M: Dim> IndexMut<(usize, usize)> for Mat<R, N, M> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let i = as_lin_idx::<N, M>(index.0, index.1);
        self.0.index_mut(i)
    }
}

macro_rules! impl_scalar_matops {
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

impl_scalar_matops!(
    Add, add, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Sub, sub, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Mul, mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div, div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);

macro_rules! impl_elemwise_matops {
    ($($Op:ident, $op:ident);+ $(;)*) => {
        $(
        impl<R, N: Dim, M: Dim> $Op for Mat<R, N, M>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, N, M>;

            fn $op(self, rhs: Self) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'a, R, N: Dim, M: Dim> $Op for &'a Mat<R, N, M>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, N, M>;

            fn $op(self, rhs: Self) -> Self::Output {
                let mut data = vec![R::zero(); N::DIM * M::DIM];
                for i in 0..(N::DIM * M::DIM) {
                    data[i] = self[i].$op(rhs[i]);
                }
                Mat(data, PhantomData::default())
            }
        }
        )+
    };
}

impl_elemwise_matops!(
    Add, add;
    Sub, sub;
);

macro_rules! impl_matmul {
    ($dim:ty, $tt:tt) => {
        impl_matmul!($dim, $dim, $dim, $tt);
    };
    ($nl:ty, $ml:ty, $mr:ty, $tt:tt) => {
        impl<R> Mul<Mat<R, $ml, $mr>> for Mat<R, $nl, $ml>
            where
                R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn mul(self, rhs: Mat<R, $ml, $mr>) -> Self::Output {
                (&self).mul(&rhs)
            }
        }

        impl<'a, R> Mul<&'a Mat<R, $ml, $mr>> for &'a Mat<R, $nl, $ml>
            where
                R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn mul(self, rhs: &'a Mat<R, $ml, $mr>) -> Self::Output {
                self.dot(rhs)
            }
        }

        impl<R> Dot<Mat<R, $ml, $mr>> for Mat<R, $nl, $ml>
            where
                R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn dot(self, rhs: Mat<R, $ml, $mr>) -> Self::Output {
                (&self).dot(&rhs)
            }
        }

        impl<'a, R> Dot<&'a Mat<R, $ml, $mr>> for &'a Mat<R, $nl, $ml>
        where
            R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn dot(self, rhs: &'a Mat<R, $ml, $mr>) -> Self::Output {
                let c = abop!(dot, self, rhs, $tt);
                c.into()
            }
        }
    };
}

impl_matmul!(D2, D1, D2, [((0), (0)), ((0), (1)), ((1), (0)), ((1), (1))]);
impl_matmul!(
    D2,
    [((0, 1), (0, 2)), ((0, 1), (1, 3)), ((2, 3), (0, 2)), ((2, 3), (1, 3)),]
);
impl_matmul!(D1, D2, D2, [((0, 1), (0, 1)), ((0, 1), (2, 3))]);

impl_matmul!(
    D3,
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
    D4,
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

impl<'a, R> Dot<&'a Mat<R, D2, D1>> for &'a Mat<R, D1, D2>
where
    R: Num + Copy + Sum,
{
    type Output = R;

    fn dot(self, rhs: &'a Mat<R, D2, D1>) -> Self::Output {
        abop!(dot, self, rhs, [((0, 1), (0, 1))])[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_lin_idx_works_reasonably_well() {
        assert_eq!(as_lin_idx::<D1, D1>(0, 0), 0);
        assert_eq!(as_lin_idx::<D2, D2>(0, 1), 1);
        assert_eq!(as_lin_idx::<D2, D2>(1, 0), 2);
    }

    #[test]
    fn mat_implements_display() {
        let a: Mat<f32, D2, D3> = Mat::from([1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(format!("{}", a), "[[1, 2, 3], [4, 5, 6]]");

        let a: Mat<f32, D1, D2> = Mat::from([1.0f32, 2.0]);
        assert_eq!(format!("{}", a), "[[1, 2]]");

        let a: Mat<f32, D2, D1> = Mat::from([1.0f32, 2.0]);
        assert_eq!(format!("{}", a), "[[1], [2]]");
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
    fn mat_supports_mut_linear_indexing() {
        let mut m: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[2] = 5.0f32;
        assert_eq!(m[2], 5.0f32);
    }

    #[test]
    fn mat_supports_2d_indexing() {
        let m: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[(1, 1)], 4.0f32);
    }

    #[test]
    fn mat_supports_mut_2d_indexing() {
        let mut m: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[(1, 1)] = 5.0f32;
        assert_eq!(m[(1, 1)], 5.0f32);
    }

    #[test]
    fn mat_supports_transposition() {
        let a: Mat<f32, D2, D3> = Mat::from([1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b: Mat<f32, D3, D2> = a.t();
        assert_eq!(b, Mat::<f32, D3, D2>::from([1.0f32, 4.0, 2.0, 5.0, 3.0, 6.0]));
    }

    #[test]
    fn mat_provides_zero_constructor() {
        let m: Mat<f32, D2, D2> = Mat::zero();
        assert_eq!(m, Mat::<f32, D2, D2>::from([0.0f32; 4]));

        let m: Mat<f32, D2, D3> = Mat::zero();
        assert_eq!(m, Mat::<f32, D2, D3>::from([0.0f32; 6]));
    }

    #[test]
    fn mat_supports_one_constructor() {
        let m: Mat<f32, D2, D2> = Mat::one();
        assert_eq!(m, Mat::<f32, D2, D2>::from([1.0f32; 4]));

        let m: Mat<f32, D2, D3> = Mat::one();
        assert_eq!(m, Mat::<f32, D2, D3>::from([1.0f32; 6]));
    }

    #[test]
    fn mat_supports_identity_constructor() {
        let m: Mat<f32, D2, D2> = Mat::identity();
        assert_eq!(m, Mat::<f32, D2, D2>::from([1.0f32, 0.0, 0.0, 1.0]));
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
    fn mat_supports_dot_product_1x2_2x2() {
        let a: Mat<f32, D1, D2> = Mat::from([2.0, 3.0]);
        let b: Mat<f32, D2, D2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let c: Mat<f32, D1, D2> = Mat::from([8.0, 18.0]);
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
        let a: Mat<f32, D4, D4> = Mat::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat<f32, D4, D4> = Mat::from([
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ]);
        let c: Mat<f32, D4, D4> = Mat::from([
            100., 110., 120., 130., 228., 254., 280., 306., 356., 398., 440., 482., 484., 542., 600., 658.,
        ]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_provides_is_nan() {
        let a: Mat<f32, D2, D2> = Mat::from([f32::NAN, 1.0, 1.0, 1.0]);
        assert!(a.is_nan());

        let a: Mat<f32, D2, D2> = Mat::from([1.0f32, 1.0, 1.0, 1.0]);
        assert!(!a.is_nan());
    }

    #[test]
    fn mat_provides_col_method() {
        let a: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 3.0]);
        let b: Mat<f32, D2, D1> = a.col(0);
        assert_eq!(b, Mat::<f32, D2, D1>::from([1.0f32, 3.0]))
    }

    #[test]
    fn mat_provides_row_method() {
        let a: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        let b: Mat<f32, D1, D2> = a.row(0);
        assert_eq!(b, Mat::<f32, D1, D2>::from([2.0f32, 4.0]))
    }

    #[test]
    fn mat_provides_norm_method() {
        let a: Mat<f32, D2, D2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(a.norm(), 5.477225575051661f32);
    }
}
