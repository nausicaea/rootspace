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
    pub fn look_at_lh(fwd: Unit<Vec3<R>>, up: Unit<Vec3<R>>) -> Self {
        let side: Unit<_> = up.as_ref().cross(fwd.as_ref()).into();
        let rotated_up = fwd.as_ref().cross(side.as_ref());

        Mat([
            [side.x(), side.y(), side.z()],
            [rotated_up.x(), rotated_up.y(), rotated_up.z()],
            [fwd.x(), fwd.y(), fwd.z()],
        ])
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn look_at_lh() {
        let fwd = Unit::from(Vec3::new(0.0f32, 0.0, -1.0));
        let up = Unit::from(Vec3::new(0.0f32, 1.0, 0.0));

        let m = Mat3::look_at_lh(fwd, up);

        assert_ulps_eq!(m, Mat3::new([[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]]));
    }
}
