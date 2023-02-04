use std::iter::Sum;

use num_traits::Float;

use crate::{Norm, Vec4};

impl<'a, R> Norm for &'a Vec4<R>
where
    R: Float + Sum,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        self.0.iter().map(|e| e.powi(2)).sum::<R>().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn vec4_provides_norm_method() {
        let a: Vec4<f32> = Vec4::new(0.0, 1.0, 2.0, 3.0);
        assert_ulps_eq!(a.norm(), 3.7416573868);
    }
}
