use num_traits::Float;

use crate::{
    forward_ref_unop,
    glamour::{ops::norm::Norm, quat::Quat},
};

impl<'a, R> Norm for &'a Quat<R>
where
    R: Float,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        self.abssq().sqrt()
    }
}

forward_ref_unop!(impl<R: Float> Norm, norm for Quat<R>, R);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glamour::ops::norm::Norm;

    #[test]
    fn quat_implements_norm() {
        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(q.norm(), 2.0f32);
    }
}
