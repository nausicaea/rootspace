#[cfg(test)]
mod approx;

use ::approx::{relative_eq, RelativeEq};
use num_traits::Float;

use super::mat::Mat4;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "R: serde::Serialize",
    deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Persp<R>(Mat4<R>);

impl<R> Persp<R> {
    pub fn as_matrix(&self) -> &Mat4<R> {
        self.as_ref()
    }
}

impl<R> Persp<R>
where
    R: Float + RelativeEq,
{
    pub fn new(aspect: R, fov_y: R, near_z: R, far_z: R) -> Self {
        let z = R::zero();
        let o = R::one();
        let t = o + o;

        if aspect < z || relative_eq!(aspect, z) {
            panic!("the 'aspect' parameter must positive and non-zero");
        }

        if relative_eq!(fov_y, z) {
            panic!("the 'fov_y' parameter must positive and non-zero");
        }

        if relative_eq!(near_z, far_z) {
            panic!("the 'near_z' and 'far_z' parameters must not be similar");
        } else if near_z > far_z {
            panic!("the 'near_z' parameter must not be larger than 'far_z'");
        }

        let tan = o / (fov_y / t).tan();
        let f_rel = -far_z / (far_z - near_z);

        let m00 = aspect * tan;
        let m11 = tan;
        let m22 = f_rel;
        let m23 = near_z * f_rel;
        let m32 = -o;

        Persp(Mat4::new([
            [m00, z, z, z],
            [z, m11, z, z],
            [z, z, m22, m23],
            [z, z, m32, z],
        ]))
    }
}

impl<R> Persp<R>
where
    R: Float + RelativeEq,
{
    pub fn set_aspect(&mut self, aspect: R) {
        let z = R::zero();

        if aspect < z || relative_eq!(aspect, z) {
            panic!("the 'aspect' parameter must positive and non-zero");
        }

        self.0[(0, 0)] = aspect * self.0[(1, 1)];
    }
}

impl<R> AsRef<Mat4<R>> for Persp<R> {
    fn as_ref(&self) -> &Mat4<R> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testing_persp() -> Persp<f32> {
        Persp::new(1.5, std::f32::consts::PI / 4.0, 0.1, 1000.0)
    }

    #[test]
    fn persp_implements_as_ref_for_mat4_and_provides_as_matrix() {
        let p = testing_persp();

        let _: &Mat4<f32> = AsRef::as_ref(&p);
        let _: &Mat4<f32> = p.as_matrix();
    }
}
