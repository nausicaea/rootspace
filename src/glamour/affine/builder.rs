use crate::glamour::affine::Affine;
use crate::glamour::num::Zero;
use crate::glamour::quat::Quat;
use crate::glamour::unit::Unit;
use crate::Vec4;
use num_traits::Float;

#[derive(Debug, PartialEq, Clone)]
pub struct AffineBuilder<R> {
    t: Option<Vec4<R>>,
    o: Option<Quat<R>>,
    s: Option<Vec4<R>>,
}

impl<R> AffineBuilder<R> {
    pub fn with_translation(mut self, v: Vec4<R>) -> Self {
        self.t = Some(v);
        self
    }

    pub fn with_orientation(mut self, q: Quat<R>) -> Self {
        self.o = Some(q);
        self
    }

    pub fn with_scale(mut self, v: Vec4<R>) -> Self {
        self.s = Some(v);
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
            o: self.o.map(Unit::from).unwrap_or_else(|| Unit::from(Quat::identity())),
            s: self
                .s
                .unwrap_or_else(|| Vec4::new(R::one(), R::one(), R::one(), R::zero())),
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

        let a: Affine<f32> = Affine::builder().with_scale(Vec4::from([1.0, 2.0, 3.0, 0.0])).build();

        assert_eq!(a.s, Vec4::from([1.0, 2.0, 3.0, 0.0]));
    }
}
