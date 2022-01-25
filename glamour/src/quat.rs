use num_traits::{Zero, One, Float};
use crate::mat::Mat4;

#[derive(Debug, PartialEq)]
pub struct Quat<R> {
    w: R,
    i: R,
    j: R,
    k: R,
}

impl<R> Quat<R> {
    pub fn new(w: R, i: R, j: R, k: R) -> Self {
        Quat { w, i, j, k }
    }
}

impl<R> Quat<R> 
where
    R: Float,
{
    pub fn norm(&self) -> R {
        (self.w.powi(2) + self.i.powi(2) + self.j.powi(2) + self.k.powi(2)).sqrt()
    }
}

impl<R> Quat<R> 
where
    R: Zero + One,
{
    pub fn identity() -> Self {
        Quat {
            w: R::one(),
            i: R::zero(),
            j: R::zero(),
            k: R::zero(),
        }
    }
}

/// Based on information from the [Euclidean Space Blog](https://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToMatrix/index.htm)
impl<'a, R> From<&'a Quat<R>> for Mat4<R> 
where
    R: Float,
{
    fn from(v: &'a Quat<R>) -> Self {
        let v_norm = v.norm();
        let w = v.w / v_norm;
        let i = v.i / v_norm;
        let j = v.j / v_norm;
        let k = v.k / v_norm;

        let z = R::zero();
        let o = R::one();
        let t = o + o;

        Mat4::from([
            o - t*j*j - t*k*k, t*i*j - t*k*w, t*i*k + t*j*w, z,
            t*i*j + t*k*w, o - t*i*i - t*k*k, t*j*k - t*i*w, z,
            t*i*k - t*j*w, t*j*k + t*i*w, o - t*i*i - t*j*j, z,
            z, z, z, o,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quat_provides_identity_constructor() {
        let q: Quat<f32> = Quat::identity();
        assert_eq!(q.w, 1.0f32);
        assert_eq!(q.i, 0.0f32);
        assert_eq!(q.j, 0.0f32);
        assert_eq!(q.k, 0.0f32);
    }

    #[test]
    fn quat_provides_new_constructor() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.w, 1.0f32);
        assert_eq!(q.i, 2.0f32);
        assert_eq!(q.j, 3.0f32);
        assert_eq!(q.k, 4.0f32);
    }

    #[test]
    fn mat4_implements_from_ref_quat() {
        let q = Quat::<f32>::identity();
        assert_eq!(Mat4::<f32>::from(&q), Mat4::<f32>::identity());

        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(Mat4::<f32>::from(&q), Mat4::<f32>::from([0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]));

        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        assert!(Mat4::<f32>::from(&q).is_nan());
    }

    #[test]
    fn mat4_from_quat_results_in_nan_for_zero_norm() {
        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        let m = Mat4::<f32>::from(&q);
        assert!(m.is_nan());
    }

    #[test]
    fn quat_implements_norm_method() {
        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(q.norm(), 2.0f32);
    }
}
