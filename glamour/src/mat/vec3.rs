use num_traits::Zero;
use super::{Mat, Vec_};
use super::vec4::Vec4;

/// Vector of 3 dimensions, interpreted as column
pub type Vec3<R> = Vec_<R, 3>;

impl<R> Vec3<R> {
    pub fn new(x: R, y: R, z: R) -> Self {
        Mat([[x], [y], [z]])
    }
}

impl<R> Vec3<R>
where
    R: Copy,
{
    pub fn x(&self) -> R {
        self[(0, 0)]
    }

    pub fn y(&self) -> R {
        self[(1, 0)]
    }

    pub fn z(&self) -> R {
        self[(2, 0)]
    }
}

impl<R> Vec3<R>
where
    R: Copy + Zero,
{
    pub fn to_vec4(&self) -> Vec4<R> {
        Vec4::new(self.x(), self.y(), self.z(), R::zero())
    }
}

