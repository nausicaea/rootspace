use numenor::{ConstOne, ConstZero};

use crate::unit::Unit;

mod approx;
mod convert;
mod num;
mod ops;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec4<R> {
    pub x: R,
    pub y: R,
    pub z: R,
    pub w: R,
}

impl<R> Vec4<R> {
    pub const fn new(x: R, y: R, z: R, w: R) -> Self {
        Self { x, y, z, w }
    }
}

impl<R: ConstOne + ConstZero> Vec4<R> {
    pub const fn new_point(x: R, y: R, z: R) -> Self {
        Self { x, y, z, w: R::ONE }
    }

    pub const fn new_vector(x: R, y: R, z: R) -> Self {
        Self { x, y, z, w: R::ZERO }
    }

    #[must_use] 
    pub const fn x() -> Unit<Self> {
        Unit(Self {
            x: R::ONE,
            y: R::ZERO,
            z: R::ZERO,
            w: R::ZERO,
        })
    }

    #[must_use] 
    pub const fn y() -> Unit<Self> {
        Unit(Self {
            x: R::ZERO,
            y: R::ONE,
            z: R::ZERO,
            w: R::ZERO,
        })
    }

    #[must_use] 
    pub const fn z() -> Unit<Self> {
        Unit(Self {
            x: R::ZERO,
            y: R::ZERO,
            z: R::ONE,
            w: R::ZERO,
        })
    }
}

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
