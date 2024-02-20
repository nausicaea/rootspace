use crate::glamour::num::Zero;
use thiserror::Error;

use super::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum Error {
    #[error("Expected a sequence of length {expected}, got {found} elements instead")]
    LengthMismatch { expected: usize, found: usize },
}

impl<R> AsRef<[[R; 4]; 4]> for Mat4<R> {
    fn as_ref(&self) -> &[[R; 4]; 4] {
        &self.0
    }
}

impl<R> TryFrom<Vec<R>> for Mat4<R>
where
    Self: Zero,
    R: Copy,
{
    type Error = Error;

    fn try_from(v: Vec<R>) -> Result<Self, Self::Error> {
        if v.len() != 16 {
            return Err(Error::LengthMismatch {
                expected: 16,
                found: v.len(),
            });
        }

        let mut mat = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                mat[(i, j)] = v[i * 4 + j];
            }
        }

        Ok(mat)
    }
}

impl<R> FromIterator<R> for Mat4<R>
where
    Self: Zero,
    R: Copy,
{
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let sh = iter.size_hint();
        if sh.0 < 16 {}

        let mut mat: Mat4<R> = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                mat[(i, j)] = iter
                    .next()
                    .unwrap_or_else(|| panic!("Expected an iterator that provides exactly {} elements", 16));
            }
        }
        mat
    }
}

impl<R> From<[[R; 4]; 4]> for Mat4<R> {
    fn from(v: [[R; 4]; 4]) -> Self {
        Mat4(v)
    }
}

macro_rules! impl_from_1d_array {
    ([$([$($i:literal),+ $(,)*]),+ $(,)*] $(,)*) => {
        impl<R> From<[R; 16]> for Mat4<R>
            where
                R: Copy,
        {
            fn from(v: [R; 16]) -> Self {
                Mat4([$(
                    [$(v[$i]),+],
                )+])
            }
        }
    }
}

impl_from_1d_array!([[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_implements_as_ref_for_2d_array() {
        let a: Mat4<f32> = Mat4::identity();
        let _: &[[f32; 4]; 4] = a.as_ref();
    }

    #[test]
    fn mat4_implements_try_from_vec() {
        let a: Result<Mat4<f32>, Error> = Mat4::try_from(vec![0.0; 16]);
        assert_eq!(a, Ok(Mat4([[0.0; 4]; 4])));
    }

    #[test]
    fn mat4_implements_from_2d_array() {
        let _: Mat4<f32> = Mat4::from([[0.0; 4]; 4]);
    }

    #[test]
    fn mat4_implements_from_array() {
        let _: Mat4<f32> = Mat4::from([0.0f32; 16]);
    }
}
