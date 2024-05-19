use super::Mat4;
use crate::num::{One, Zero};

impl<R> Zero for Mat4<R>
where
    R: Copy + num_traits::Zero,
{
    fn zero() -> Self {
        Mat4([[R::zero(); 4]; 4])
    }
}

impl<R> One for Mat4<R>
where
    R: Copy + num_traits::One,
{
    fn one() -> Self {
        Mat4([[R::one(); 4]; 4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_provides_zero_constructor() {
        let m: Mat4<f32> = Mat4::zero();
        assert_eq!(m, Mat4([[0.0f32; 4]; 4]));
    }

    #[test]
    fn mat4_supports_one_constructor() {
        let m: Mat4<f32> = Mat4::one();
        assert_eq!(m, Mat4([[1.0f32; 4]; 4]));
    }
}
