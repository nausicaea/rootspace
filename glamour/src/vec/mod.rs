mod approx;
mod convert;
mod num;
mod ops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4<R>(pub [R; 4]);

impl<R> Vec4<R> {
    pub const fn new(x: R, y: R, z: R, w: R) -> Self {
        Vec4([x, y, z, w])
    }
}
