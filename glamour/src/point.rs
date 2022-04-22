use super::mat::{Vec4, Vec3, Vec2};

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
pub struct Point2<R>(Vec2<R>);

impl<R> From<Point2<R>> for Vec2<R> {
    fn from(value: Point2<R>) -> Self {
        value.0
    }
}

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

impl<R> From<Point3<R>> for Vec3<R> {
    fn from(value: Point3<R>) -> Self {
        value.0
    }
}

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
pub struct Point4<R>(Vec4<R>);

impl<R> From<Point4<R>> for Vec4<R> {
    fn from(value: Point4<R>) -> Self {
        value.0
    }
}
