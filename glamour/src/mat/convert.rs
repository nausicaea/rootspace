use super::Mat;
use thiserror::Error;
use num_traits::Zero;

impl<R, const I: usize, const J: usize> AsRef<[[R; J]; I]> for Mat<R, I, J> {
    fn as_ref(&self) -> &[[R; J]; I] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum Error {
    #[error("Expected a sequence of length {expected}, got {found} elements instead")]
    LengthMismatch { expected: usize, found: usize },
}

impl<R, const I: usize, const J: usize> TryFrom<Vec<R>> for Mat<R, I, J> 
where
    R: Copy + Zero,
{
    type Error = Error;

    fn try_from(v: Vec<R>) -> Result<Self, Self::Error> {
        if v.len() != I * J {
            return Err(Error::LengthMismatch{ expected: I * J, found: v.len() });
        }

        let mut mat = Mat::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(i, j)] = v[i * J + j];
            }
        }

        Ok(mat)
    }
}

impl<R, const I: usize, const J: usize> FromIterator<R> for Mat<R, I, J> 
where
    R: Copy + Zero,
{
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let sh = iter.size_hint();
        if sh.0 < I * J {
        }


        let mut mat: Mat<R, I, J> = Mat::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(i, j)] = iter.next()
                    .unwrap_or_else(|| panic!("Expected an iterator that provides exactly {} elements", I * J));
            }
        }
        mat
    }
}

impl<R, const I: usize, const J: usize> From<[[R; J]; I]> for Mat<R, I, J> {
    fn from(v: [[R; J]; I]) -> Self {
        Mat(v)
    }
}

impl<R> From<R> for Mat<R, 1, 1> {
    fn from(v: R) -> Self {
        Mat([[v]])
    }
}

macro_rules! impl_from_1d_array {
    ($I:literal, $J:literal, [$([$($i:literal),+ $(,)*]),+ $(,)*] $(,)*) => {
        impl<R> From<[R; $I * $J]> for Mat<R, $I, $J>
            where
                R: Copy,
        {
            fn from(v: [R; $I * $J]) -> Self {
                Mat([$(
                    [$(v[$i]),+],
                )+])
            }
        }
    }
}

impl_from_1d_array!(1, 1, [[0]]);
impl_from_1d_array!(1, 2, [[0, 1]]);
impl_from_1d_array!(1, 3, [[0, 1, 2]]);
impl_from_1d_array!(1, 4, [[0, 1, 2, 3]]);
impl_from_1d_array!(2, 1, [[0], [1]]);
impl_from_1d_array!(2, 2, [[0, 1], [2, 3]]);
impl_from_1d_array!(2, 3, [[0, 1, 2], [3, 4, 5]]);
impl_from_1d_array!(2, 4, [[0, 1, 2, 3], [4, 5, 6, 7]]);
impl_from_1d_array!(3, 1, [[0], [1], [2]]);
impl_from_1d_array!(3, 2, [[0, 1], [2, 3], [4, 5]]);
impl_from_1d_array!(3, 3, [[0, 1, 2], [3, 4, 5], [6, 7, 8]]);
impl_from_1d_array!(3, 4, [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11]]);
impl_from_1d_array!(4, 1, [[0], [1], [2], [3]]);
impl_from_1d_array!(4, 2, [[0, 1], [2, 3], [4, 5], [6, 7]]);
impl_from_1d_array!(4, 3, [[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11]]);
impl_from_1d_array!(4, 4, [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat_implements_as_ref_for_2d_array() {
        let a: Mat<f32, 2, 2> = Mat::identity();
        let _: &[[f32; 2]; 2] = a.as_ref();
    }

    #[test]
    fn mat_implements_try_from_vec() {
        let a: Result<Mat<f32, 2, 2>, Error> = Mat::try_from(vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(a, Ok(Mat([[0.0, 1.0], [2.0, 3.0]])));
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


}
