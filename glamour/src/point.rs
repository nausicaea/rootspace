use num_traits::One;
use super::mat::{Vec4, Vec3};

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde_support",
    serde(bound(
        serialize = "R: serde::Serialize",
        deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
    ))
)]
#[cfg_attr(feature = "serde_support", serde(transparent))]
#[derive(Debug)]
pub struct Point3<R>(Vec3<R>);

impl<R> Point3<R>
where
    R: Copy,
{
    pub fn x(&self) -> R {
        self.0[(0, 0)]
    }

    pub fn y(&self) -> R {
        self.0[(1, 0)]
    }

    pub fn z(&self) -> R {
        self.0[(2, 0)]
    }
}

impl<R> Point3<R>
where
    R: Copy + One,
{
    pub fn to_vec4(&self) -> Vec4<R> {
        Vec4::new(self.x(), self.y(), self.z(), R::one())
    }
}

impl<R> From<Point3<R>> for Vec3<R> {
    fn from(value: Point3<R>) -> Self {
        value.0
    }
}
