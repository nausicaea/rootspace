use num_traits::Float;

use crate::glamour::{mat::Mat4, num::ToMatrix};

use super::Quat;

impl<R> ToMatrix<R> for Quat<R>
where
    R: Float,
{
    fn to_matrix(&self) -> Mat4<R> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::glamour::{mat::Mat4, num::ToMatrix};

    use super::*;

    #[test]
    fn quat_provides_to_matrix_method() {
        let q = Quat::<f32>::identity();
        assert_eq!(q.to_matrix(), Mat4::<f32>::identity());

        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(
            q.to_matrix(),
            Mat4::<f32>::from([0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0])
        );

        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        assert!(q.to_matrix().is_nan());
    }
}
