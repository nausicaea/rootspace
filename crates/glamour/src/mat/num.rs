use numenor::{ConstOne, ConstZero};

use super::Mat4;

impl<R> ConstZero for Mat4<R>
where
    R: Copy + ConstZero,
{
    const ZERO: Self = Self([[R::ZERO; 4]; 4]);
}

impl<R> ConstOne for Mat4<R>
where
    R: Copy + ConstOne,
{
    const ONE: Self = Self([[R::ONE; 4]; 4]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_provides_zero_constructor() {
        let m: Mat4<f32> = Mat4::ZERO;
        assert_eq!(m, Mat4([[0.0f32; 4]; 4]));
    }

    #[test]
    fn mat4_supports_one_constructor() {
        let m: Mat4<f32> = Mat4::ONE;
        assert_eq!(m, Mat4([[1.0f32; 4]; 4]));
    }
}
