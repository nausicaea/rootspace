use num_traits::Float;

use crate::{ops::norm::Norm, vec::Vec4};

impl<R> Norm for &Vec4<R>
where
    R: Float,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }
}

impl<R> Norm for Vec4<R>
where
    R: Float,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        Norm::norm(&self)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;
    use crate::vec::Vec4;

    #[test]
    fn vec4_provides_norm_method() {
        let a: Vec4<f32> = Vec4::new(0.0, 1.0, 2.0, 3.0);
        assert_ulps_eq!(a.norm(), 3.741_657_5);
    }
}
