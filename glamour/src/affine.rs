use num_traits::{Zero, One, Float, NumAssign};
use crate::mat::{Vec3, Mat4};
use crate::quat::Quat;

#[derive(Debug, PartialEq)]
pub struct Affine<R> {
    t: Vec3<R>,
    o: Quat<R>,
    s: Vec3<R>,
}

impl<R> Affine<R> {
    pub fn builder() -> AffineBuilder<R> {
        AffineBuilder::default()
    }
}

impl<R> Affine<R> 
where
    R: Zero + One + Copy,
{
    pub fn identity() -> Self {
        AffineBuilder::default().build()
    }
}

impl<'a, R> From<&'a Affine<R>> for Mat4<R>
where
    R: Float + NumAssign,
{
    fn from(value: &'a Affine<R>) -> Self {
        let mut m: Mat4<R> = (&value.o).into();
        m[(0, 0)] *= value.s[0];
        m[(1, 1)] *= value.s[1];
        m[(2, 2)] *= value.s[2];
        m[(0, 3)] = value.t[0];
        m[(1, 3)] = value.t[1];
        m[(2, 3)] = value.t[2];
        m
    }
}

impl<'a, R> From<&'a Mat4<R>> for Affine<R> 
where
    R: Copy,
{
    fn from(v: &'a Mat4<R>) -> Self {
        //let t = Vec3::from([v[(0, 3)], v[(1, 3)], v[(2, 3)]]);
        todo!()
    }
}

impl<R> From<AffineBuilder<R>> for Affine<R> 
where
    R: One + Zero + Copy,
{
    fn from(v: AffineBuilder<R>) -> Self {
        Affine {
            t: v.t.unwrap_or_else(|| Vec3::zero()),
            o: v.o.unwrap_or_else(|| Quat::identity()),
            s: v.s.unwrap_or_else(|| Vec3::one()),
        }
    }
}

pub struct AffineBuilder<R> {
    t: Option<Vec3<R>>,
    o: Option<Quat<R>>,
    s: Option<Vec3<R>>,
}

impl<R> AffineBuilder<R> {
    pub fn with_translation(mut self, v: Vec3<R>) -> Self {
        self.t = Some(v);
        self
    }

    pub fn with_orientation(mut self, q: Quat<R>) -> Self {
        self.o = Some(q);
        self
    }

    pub fn with_scale(mut self, v: Vec3<R>) -> Self {
        self.s = Some(v);
        self
    }
}

impl<R> AffineBuilder<R> 
where
    R: One + Zero + Copy,
{
    pub fn build(self) -> Affine<R> {
        self.into()
    }
}

impl<R> Default for AffineBuilder<R> {
    fn default() -> Self {
        AffineBuilder {
            t: None,
            o: None,
            s: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affine_provides_identity_constructor() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(a.t, Vec3::<f32>::zero());
        assert_eq!(a.o, Quat::<f32>::identity());
        assert_eq!(a.s, Vec3::<f32>::one());
    }

    #[test]
    fn affine_provides_builder() {
        let a: Affine<f32> = Affine::builder().build();
        assert_eq!(a, Affine::<f32>::identity());

        let a: Affine<f32> = Affine::builder()
            .with_scale(Vec3::from([1.0, 2.0, 3.0]))
            .build();

        assert_eq!(a.s, Vec3::from([1.0, 2.0, 3.0]));
    }

    #[test]
    fn mat4_implements_from_ref_affine() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(Mat4::<f32>::from(&a), Mat4::<f32>::identity());
    }
}
