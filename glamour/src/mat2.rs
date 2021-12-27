use crate::dot::Dot;
use crate::vec2::Vec2;
use num_traits::Num;
use std::ops::{Add, Div, Index, Mul, Sub};

#[derive(Debug, PartialEq)]
pub struct Mat2<N>([N; 4]);

impl<N> Mat2<N> {
    pub fn new(m00: N, m01: N, m10: N, m11: N) -> Self {
        Mat2([m00, m01, m10, m11])
    }
}

impl<N> From<[N; 4]> for Mat2<N> {
    fn from(value: [N; 4]) -> Self {
        Mat2(value)
    }
}

impl<N> From<[[N; 2]; 2]> for Mat2<N>
where
    N: Copy,
{
    fn from(value: [[N; 2]; 2]) -> Self {
        Mat2([value[0][0], value[0][1], value[1][0], value[1][1]])
    }
}

impl<N> Index<usize> for Mat2<N> {
    type Output = N;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<N> Index<(usize, usize)> for Mat2<N> {
    type Output = N;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let lin_idx = 2 * index.0 + index.1;
        self.0.index(lin_idx)
    }
}

impl<'a, N> Add for &'a Mat2<N>
where
    N: Num + Copy,
{
    type Output = Mat2<N>;

    fn add(self, rhs: Self) -> Self::Output {
        Mat2(abop!(std::ops::Add::add, self.0, rhs.0, [(0, 0), (1, 1), (2, 2), (3, 3)]))
    }
}

impl<'a, N> Sub for &'a Mat2<N>
where
    N: Num + Copy,
{
    type Output = Mat2<N>;

    fn sub(self, rhs: Self) -> Self::Output {
        Mat2(abop!(std::ops::Sub::sub, self.0, rhs.0, [(0, 0), (1, 1), (2, 2), (3, 3)]))
    }
}

impl<'a, N> Dot for &'a Mat2<N>
where
    N: Num + std::iter::Sum + Copy,
{
    type Output = Mat2<N>;

    fn dot(self, rhs: Self) -> Self::Output {
        Mat2(abop!(crate::dot::Dot::dot, self.0, rhs.0, [
            ((0, 1), (0, 2)),
            ((0, 1), (1, 3)),
            ((2, 3), (0, 2)),
            ((2, 3), (1, 3)),
        ]))
    }
}

impl<'a, N> Dot<&'a Vec2<N>> for &'a Mat2<N>
where
    N: Num + std::iter::Sum + Copy,
{
    type Output = Vec2<N>;

    fn dot(self, rhs: &'a Vec2<N>) -> Self::Output {
        Vec2(abop!(crate::dot::Dot::dot, self.0, rhs.0, [((0, 1), (0, 1)), ((2, 3), (0, 1))]))
    }
}

impl<'a, N> Add<N> for &'a Mat2<N>
where
    N: Num + Copy,
{
    type Output = Mat2<N>;

    fn add(self, rhs: N) -> Self::Output {
        Mat2(abop!(std::ops::Add::add, self.0, [0, 1, 2, 3], rhs))
    }
}

impl<'a, N> Sub<N> for &'a Mat2<N>
where
    N: Num + Copy,
{
    type Output = Mat2<N>;

    fn sub(self, rhs: N) -> Self::Output {
        Mat2(abop!(std::ops::Sub::sub, self.0, [0, 1, 2, 3], rhs))
    }
}

impl<'a, N> Mul<N> for &'a Mat2<N>
where
    N: Num + Copy,
{
    type Output = Mat2<N>;

    fn mul(self, rhs: N) -> Self::Output {
        Mat2(abop!(std::ops::Mul::mul, self.0, [0, 1, 2, 3], rhs))
    }
}

impl<'a, N> Div<N> for &'a Mat2<N>
where
    N: Num + Copy,
{
    type Output = Mat2<N>;

    fn div(self, rhs: N) -> Self::Output {
        Mat2(abop!(std::ops::Div::div, self.0, [0, 1, 2, 3], rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn mat2_has_four_constructor_arguments() {
        let _: Mat2<f32> = Mat2::new(0.0f32, 1.0f32, 3.0f32, 4.0f32);
    }

    #[test]
    fn mat2_supports_indexing() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);

        assert_ulps_eq!(a[(0, 1)], 1.0f32);
    }

    #[test]
    fn mat2_supports_dot_product() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = Mat2::new(0.5f32, 1.5f32, 2.5f32, 3.5f32);

        let c: Mat2<f32> = a.dot(&b);
        assert_ulps_eq!(c[(0, 0)], 2.5f32);
        assert_ulps_eq!(c[(0, 1)], 3.5f32);
        assert_ulps_eq!(c[(1, 0)], 8.5f32);
        assert_ulps_eq!(c[(1, 1)], 13.5f32);
    }

    #[test]
    fn mat2_supports_addition() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = Mat2::new(0.5f32, 1.5f32, 2.5f32, 3.5f32);

        let c: Mat2<f32> = a.add(&b);
        assert_ulps_eq!(c[(0, 0)], 0.5f32);
        assert_ulps_eq!(c[(0, 1)], 2.5f32);
        assert_ulps_eq!(c[(1, 0)], 4.5f32);
        assert_ulps_eq!(c[(1, 1)], 6.5f32);
    }

    #[test]
    fn mat2_supports_subtraction() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = Mat2::new(0.5f32, 1.5f32, 2.5f32, 3.5f32);

        let c: Mat2<f32> = a.sub(&b);
        assert_ulps_eq!(c[(0, 0)], -0.5f32);
        assert_ulps_eq!(c[(0, 1)], -0.5f32);
        assert_ulps_eq!(c[(1, 0)], -0.5f32);
        assert_ulps_eq!(c[(1, 1)], -0.5f32);
    }

    #[test]
    fn mat2_supports_scalar_addition() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Mat2<f32> = a.add(b);
        assert_ulps_eq!(c[(0, 0)], 2.0f32);
        assert_ulps_eq!(c[(0, 1)], 3.0f32);
        assert_ulps_eq!(c[(1, 0)], 4.0f32);
        assert_ulps_eq!(c[(1, 1)], 5.0f32);
    }

    #[test]
    fn mat2_supports_scalar_subtraction() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Mat2<f32> = a.sub(b);
        assert_ulps_eq!(c[(0, 0)], -2.0f32);
        assert_ulps_eq!(c[(0, 1)], -1.0f32);
        assert_ulps_eq!(c[(1, 0)], 0.0f32);
        assert_ulps_eq!(c[(1, 1)], 1.0f32);
    }

    #[test]
    fn mat2_supports_scalar_multiplication() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Mat2<f32> = a.mul(b);
        assert_ulps_eq!(c[(0, 0)], 0.0f32);
        assert_ulps_eq!(c[(0, 1)], 2.0f32);
        assert_ulps_eq!(c[(1, 0)], 4.0f32);
        assert_ulps_eq!(c[(1, 1)], 6.0f32);
    }

    #[test]
    fn mat2_supports_scalar_division() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Mat2<f32> = a.div(b);
        assert_ulps_eq!(c[(0, 0)], 0.0f32);
        assert_ulps_eq!(c[(0, 1)], 0.5f32);
        assert_ulps_eq!(c[(1, 0)], 1.0f32);
        assert_ulps_eq!(c[(1, 1)], 1.5f32);
    }

    #[test]
    fn mat2_supports_f64() {
        let _: Mat2<f64> = Mat2::new(1.0f64, 2.0f64, 3.0f64, 4.0f64);
    }

    #[test]
    fn mat2_supports_vec2_dot_product() {
        let a = Mat2::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);
        let b = Vec2::new(1.0f32, 2.0f32);

        let c: Vec2<f32> = a.dot(&b);
        assert_ulps_eq!(c[0], 2.0f32);
        assert_ulps_eq!(c[1], 8.0f32);
    }

}
