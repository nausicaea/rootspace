use super::{Mat, Vec_};

/// Vector of 2 dimensions, interpreted as column
pub type Vec2<R> = Vec_<R, 2>;

impl<R> Vec2<R> {
    pub fn new(x: R, y: R) -> Self {
        Mat([[x], [y]])
    }
}

impl<R> Vec2<R>
where
    R: Copy,
{
    pub fn x(&self) -> R {
        self[(0, 0)]
    }

    pub fn y(&self) -> R {
        self[(1, 0)]
    }
}

