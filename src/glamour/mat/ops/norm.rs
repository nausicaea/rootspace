use std::iter::Sum;

use crate::glamour::ops::norm::Norm;
use num_traits::real::Real;
use num_traits::Float;

use super::super::Mat4;

impl<'a, R> Norm for &'a Mat4<R>
where
    R: Float + Sum,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        self.0.iter().flatten().map(|e| e.powi(2)).sum::<R>().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn mat4_provides_norm_method() {
        let a: Mat4<f32> = (0..16).into_iter().map(|n| n as f32).collect();
        assert_ulps_eq!(a.norm(), 35.21363372331802);
    }
}
