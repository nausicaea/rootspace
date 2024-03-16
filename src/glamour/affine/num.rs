use crate::glamour::affine::Affine;
use crate::glamour::mat::Mat4;
use crate::glamour::num::ToMatrix;
use num_traits::{Float, NumAssign};

impl<R> ToMatrix<R> for Affine<R>
where
    R: Float + NumAssign,
{
    fn to_matrix(&self) -> Mat4<R> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affine_provides_to_matrix_method() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(a.to_matrix(), Mat4::<f32>::identity());
    }
}
