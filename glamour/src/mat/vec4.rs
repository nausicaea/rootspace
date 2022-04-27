use super::{Mat, Vec_};

/// Vector of 4 dimensions, interpreted as column
pub type Vec4<R> = Vec_<R, 4>;

impl<R> Vec4<R> {
    pub fn new(x: R, y: R, z: R, w: R) -> Self {
        Mat([[x], [y], [z], [w]])
    }
}

impl<R> Vec4<R>
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

    pub fn w(&self) -> R {
        self[(3, 0)]
    }
}

