use num_traits::{Zero, One};
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
        self[0]
    }

    pub fn y(&self) -> R {
        self[1]
    }

    pub fn z(&self) -> R {
        self[2]
    }
}

impl<R> Vec3<R>
where
    R: Copy + One,
{
    pub fn to_point4(&self) -> Vec4<R> {
        Vec4::new(self.x(), self.y(), self.z(), R::one())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec3_provides_a_constructor() {
        let _: Vec3<f32> = Vec3::new(0.0, 1.0, 2.0);
    }
}
