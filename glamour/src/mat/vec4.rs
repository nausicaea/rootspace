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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec4_implements_new() {
        let _: Vec4<f32> = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    }

    #[test]
    fn vec4_implements_x_y_z_and_w() {
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
        assert_eq!(v.z(), 3.0f32);
        assert_eq!(v.w(), 4.0f32);
    }
}
