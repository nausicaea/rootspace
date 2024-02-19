use num_traits::Float;

use super::ops::cross::Cross;

mod approx;
mod convert;
mod num;
mod ops;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4<R> {
    pub x: R,
    pub y: R,
    pub z: R,
    pub w: R,
}

impl<R> Vec4<R> {
    pub const fn new(x: R, y: R, z: R, w: R) -> Self {
        Vec4 { x, y, z, w }
    }
}

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

impl<R> std::fmt::Display for Vec4<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let prettyprint = f.alternate();

        if prettyprint {
            write!(f, "[\n{},\n {},\n {},\n {}\n]", self.x, self.y, self.z, self.w)
        } else {
            write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
        }
    }
}
