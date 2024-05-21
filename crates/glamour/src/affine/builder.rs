use num_traits::Float;

use crate::{affine::Affine, num::Zero, quat::Quat, unit::Unit, vec::Vec4};

#[derive(Debug, PartialEq, Clone)]
pub struct AffineBuilder<R> {
    t: Option<Vec4<R>>,
    o: Option<Unit<Quat<R>>>,
    s: Option<R>,
}

impl<R> AffineBuilder<R> {
    pub fn with_translation(mut self, v: Vec4<R>) -> Self {
        self.t = Some(v);
        self
    }

    pub fn with_orientation<Q: Into<Unit<Quat<R>>>>(mut self, q: Q) -> Self {
        self.o = Some(q.into());
        self
    }

    pub fn with_scale(mut self, s: R) -> Self {
        self.s = Some(s);
        self
    }
}

impl<R> AffineBuilder<R>
where
    R: Float,
{
    pub fn build(self) -> Affine<R> {
        Affine {
            t: self.t.unwrap_or_else(Vec4::zero),
            o: self.o.unwrap_or_else(|| Unit::from(Quat::identity())),
            s: self.s.unwrap_or_else(|| R::one()),
        }
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
    fn affine_provides_builder() {
        let a: Affine<f32> = Affine::builder().build();
        assert_eq!(a, Affine::<f32>::identity());

        let a: Affine<f32> = Affine::builder().with_scale(2.0).build();

        assert_eq!(a.s, 2.0f32);
    }
}
