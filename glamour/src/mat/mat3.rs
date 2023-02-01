use std::iter::Sum;

use num_traits::Float;

use crate::{ops::Cross, Unit, Vec3};

use super::Mat;

/// Matrix of 3x3 dimensions
pub type Mat3<R> = Mat<R, 3, 3>;

impl<R> Mat3<R> {
    pub const fn new(v: [[R; 3]; 3]) -> Self {
        Mat(v)
    }
}

impl<R> Mat3<R>
where
    R: Float + Sum,
{
    pub fn look_at_rh<U: Into<Unit<Vec3<R>>>>(fwd: U, up: U) -> Self {
        let fwd = fwd.into();
        let up = up.into();
        let right: Unit<_> = fwd.as_ref().cross(up.as_ref()).into();
        let rotated_up = right.as_ref().cross(fwd.as_ref());

        Mat([
            [right.x(), rotated_up.x(), fwd.x()],
            [right.y(), rotated_up.y(), fwd.y()],
            [right.z(), rotated_up.z(), fwd.z()],
        ])
    }
}
