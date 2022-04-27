use ecs::{Component, VecStorage};
use glamour::{Affine, Mat4, Vec3, Quat, AffineBuilder, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Model(Affine<f32>);

impl Model {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::default()
    }

    pub fn transform_point(&self, point: &Vec3<f32>) -> Vec3<f32> {
        let v4 = Vec4::new(point.x(), point.y(), point.z(), 1.0);
        let v4t = self.0 * v4;
        Vec3::new(v4t.x(), v4t.y(), v4t.z())
    }

    pub fn inverse_transform_point(&self, point: &Vec3<f32>) -> Vec3<f32> {
        let v4 = Vec4::new(point.x(), point.y(), point.z(), 1.0);
        let v4t = self.0.inv() * v4;
        Vec3::new(v4t.x(), v4t.y(), v4t.z())
    }

    pub fn set_translation(&mut self, value: Vec3<f32>) {
        self.0.t = value;
    }

    pub fn set_orientation(&mut self, value: Quat<f32>) {
        self.0.o = value;
    }

    pub fn set_scale(&mut self, value: Vec3<f32>) {
        self.0.s = value;
    }

    pub fn as_affine(&self) -> &Affine<f32> {
        self.as_ref()
    }

    pub fn to_matrix(&self) -> Mat4<f32> {
        self.0.to_matrix()
    }

    pub fn translation(&self) -> &Vec3<f32> {
        &self.0.t
    }

    pub fn orientation(&self) -> &Quat<f32> {
        &self.0.o
    }

    pub fn scale(&self) -> &Vec3<f32> {
        &self.0.s
    }
}

impl Default for Model {
    fn default() -> Self {
        Model::builder().build()
    }
}

impl Component for Model {
    type Storage = VecStorage<Self>;
}

impl From<Affine<f32>> for Model {
    fn from(value: Affine<f32>) -> Self {
        Model(value)
    }
}

impl AsRef<Affine<f32>> for Model {
    fn as_ref(&self) -> &Affine<f32> {
        &self.0
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "position: [{}, {}, {}], orientation: {}, scale: [{}, {}, {}]",
            self.translation().x(),
            self.translation().y(),
            self.translation().z(),
            self.orientation(),
            self.scale().x(),
            self.scale().y(),
            self.scale().z(),
        )
    }
}

#[derive(Debug)]
pub struct ModelBuilder(AffineBuilder<f32>);

impl ModelBuilder {
    pub fn with_translation(mut self, t: Vec3<f32>) -> Self {
        self.0 = self.0.with_translation(t);
        self
    }

    pub fn with_orientation(mut self, o: Quat<f32>) -> Self {
        self.0 = self.0.with_orientation(o);
        self
    }

    pub fn with_scale(mut self, s: Vec3<f32>) -> Self {
        self.0 = self.0.with_scale(s);
        self
    }

    pub fn build(self) -> Model {
        Model(self.0.build())
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        ModelBuilder(AffineBuilder::default())
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_ulps_eq, ulps_eq};
    use proptest::prelude::*;
    use glamour::Vec4;

    use super::*;
    use crate::utilities::validate_float;

    #[test]
    fn implements_default() {
        let _: Model = Default::default();
    }

    #[test]
    fn provides_builder() {
        let _: ModelBuilder = Model::builder();
    }

    #[test]
    fn blank_builder_is_the_same_as_default() {
        let ma: Model = Model::builder().build();
        let mb: Model = Default::default();

        assert_eq!(ma, mb);
        // TODO: assert_ulps_eq!(ma, mb);
    }

    #[test]
    fn default_is_identity() {
        let m: Model = Default::default();
        assert_ulps_eq!(m.to_matrix(), &Mat4::identity())
    }

    #[test]
    fn builder_accepts_position() {
        let _: ModelBuilder = ModelBuilder::default().with_translation(Vec3::zero());
    }

    #[test]
    fn builder_accepts_orientaton() {
        let _: ModelBuilder =
            ModelBuilder::default().with_orientation(Quat::identity());
    }

    #[test]
    fn builder_accepts_scale() {
        let _: ModelBuilder = ModelBuilder::default().with_scale(Vec3::zero());
    }

    #[test]
    fn builder_complete_example() {
        let m: Model = ModelBuilder::default()
            .with_translation(Vec3::zero())
            .with_orientation(Quat::identity())
            .with_scale(Vec3::one())
            .build();

        assert_ulps_eq!(m.translation(), Vec3::zero());
        assert_ulps_eq!(m.orientation(), Quat::identity());
        assert_ulps_eq!(m.scale(), Vec3::one());
    }

    #[test]
    fn transform_point_works_for_zeroes() {
        let m: Model = ModelBuilder::default()
            .with_translation(Vec3::zero())
            .with_orientation(Quat::identity())
            .with_scale(Vec3::one())
            .build();
        let p: Vec3<f32> = Vec3::zero();

        let tpt: Vec3<f32> = m.transform_point(&p);
        let tpt: Vec4<f32> = Vec4::new(tpt.x(), tpt.y(), tpt.z(), 1.0);
        let mmul = m.to_matrix() * Vec4::new(p.x(), p.y(), p.z(), 1.0);

        assert_ulps_eq!(tpt, mmul);
    }

    proptest! {
        #[test]
        fn position_may_be_changed(num: [f32; 3]) {
            let mut m = Model::default();

            let p = Vec3::new(num[0], num[1], num[2]);
            m.set_translation(p);

            if !validate_float(&num) {
                return Ok(());
            } else {
                prop_assert_eq!(m.translation(), p);
            }
        }

        #[test]
        fn orientation_may_be_changed(num: [f32; 4]) {
            let mut m = Model::default();

            let o = Quat::new(num[0], num[1], num[2], num[3]);
            m.set_orientation(o);

            if !validate_float(&num) {
                return Ok(());
            } else {
                prop_assert_eq!(m.orientation(), o);
            }
        }

        #[test]
        fn scale_may_be_changed(num: [f32; 3]) {
            let mut m = Model::default();

            let s = Vec3::new(num[0], num[1], num[2]);
            m.set_scale(s);

            if !validate_float(&num) {
                return Ok(());
            } else {
                prop_assert_eq!(m.scale(), s);
            }
        }

        #[test]
        fn transform_point_is_the_same_as_matrix_multiplication(num: [f32; 13]) {
            let m = Model::builder()
                .with_translation(Vec3::new(num[0], num[1], num[2]))
                .with_orientation(Quat::new(num[3], num[4], num[5], num[6]))
                .with_scale(Vec3::new(num[7], num[8], num[9]))
                .build();
            let p = Vec3::new(num[10], num[11], num[12]);

            if !validate_float(&num) {
                return Ok(())
            } else {
                let tpt: Vec3<f32> = m.transform_point(&p);
                let tpt: Vec4<f32> = Vec4::new(tpt.x(), tpt.y(), tpt.z(), 1.0);
                let mmul = m.to_matrix() * Vec4::new(p.x(), p.y(), p.z(), 1.0);

                prop_assert!(ulps_eq!(tpt, mmul), "{:?} != {:?}", tpt, mmul);
            }
        }

        #[test]
        fn transformations_are_invertible(num: [f32; 13]) {
            let m = Model::builder()
                .with_translation(Vec3::new(num[0], num[1], num[2]))
                .with_orientation(Quat::new(num[3], num[4], num[5], num[6]))
                .with_scale(Vec3::new(num[7], num[8], num[9]))
                .build();
            let p = Vec3::new(num[10], num[11], num[12]);

            if !validate_float(&num) {
                return Ok(())
            } else {
                let tpt: Vec3<f32> = m.transform_point(&p);
                let itpt: Vec3<f32> = m.inverse_transform_point(&tpt);

                prop_assert!(ulps_eq!(p, itpt), "{:?} != {:?}", p, itpt);
            }
        }
    }
}
