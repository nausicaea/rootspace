use crate::glamour::ops::norm::Norm;
use crate::glamour::vec::Vec4;
use num_traits::Float;

impl<'a, R> Norm for &'a Vec4<R>
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
    use crate::glamour::vec::Vec4;
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn vec4_provides_norm_method() {
        let a: Vec4<f32> = Vec4::new(0.0, 1.0, 2.0, 3.0);
        assert_ulps_eq!(a.norm(), 3.7416573868);
    }
}
