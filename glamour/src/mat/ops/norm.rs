use crate::ops::norm::Norm;
use num_traits::Float;
use super::super::Mat;
use std::iter::Sum;

impl<'a, R, const I: usize, const J: usize> Norm for &'a Mat<R, I, J> 
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
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn mat_provides_norm_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_ulps_eq!(a.norm(), 5.477225575051661f32);
    }
}
