use num_traits::Float;

use crate::glamour::{ops::cross::Cross, vec::Vec4};

impl<'a, 'b, R> Cross<&'b Vec4<R>> for &'a Vec4<R>
where
    R: Copy + num_traits::Zero + std::ops::Mul<R, Output = R> + std::ops::Sub<R, Output = R>,
{
    type Output = Vec4<R>;

    fn cross(self, rhs: &'b Vec4<R>) -> Self::Output {
        Vec4 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
            w: R::zero(),
        }
    }
}

crate::forward_ref_binop!(impl<R: Float> Cross, cross for Vec4<R>, Vec4<R>, Vec4<R>);
