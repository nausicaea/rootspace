use approx::assert_relative_eq;
use glamour::affine::Affine;
use glamour::mat::Mat4;
use glamour::num::{ToMatrix, Zero};
use glamour::quat::Quat;
use glamour::unit::Unit;
use glamour::vec::Vec4;
use num_traits::Inv;
use std::f32::consts::PI;

/// The test attempts to quantify the error of calculating the model-view matrix either using [`Affine`] or directly with [`Mat4`]. The double inverse of the camera-view matrix is wrong, but was accidentally used in [`rootspace::systems::renderer::Renderer`].
#[test]
#[should_panic]
fn model_view_matrix_with_double_inverse_error() {
    // NOTE: Here the first half of the erroneous double-inverse.
    let camera_view = Affine::with_look_at_rh(
        Vec4::new_point(0.0, 5.0, -10.0),
        Vec4::new_point(0.0, 0.0, 0.0),
        Vec4::y(),
    )
    .inv();

    let cube = {
        let position = Vec4::new_point(1.5, 0.0, 1.5);
        let (axis, angle) = (Unit::from(position), PI / 4.0);

        Affine::builder()
            .with_translation(position)
            .with_orientation(Quat::with_axis_angle(axis, angle))
            .build()
    };

    // Calculate the model-view matrix via the Affine matrix simplification.
    // NOTE: Here the second half of the erroneous double-inverse.
    let model_view_affine = camera_view.inv() * cube;

    // Calculate the model-view matrix as a product of two matrices.
    // NOTE: Here the second half of the erroneous double-inverse.
    let camera_view = camera_view.to_matrix().inv();
    let model_view_matrix = camera_view * cube.to_matrix();

    assert_relative_eq!(model_view_affine - model_view_matrix, Mat4::zero());
}

/// The test attempts to quantify the error of calculating the model-view matrix either using [`Affine`] or directly with [`Mat4`]. This is the same test without the erroneous double-inverse in the camera-view matrix.
#[test]
fn model_view_matrix_error() {
    let camera_view = Affine::with_look_at_rh(
        Vec4::new_point(0.0, 5.0, -10.0),
        Vec4::new_point(0.0, 0.0, 0.0),
        Vec4::y(),
    );

    let cube = {
        let position = Vec4::new_point(1.5, 0.0, 1.5);
        let (axis, angle) = (Unit::from(position), PI / 4.0);

        Affine::builder()
            .with_translation(position)
            .with_orientation(Quat::with_axis_angle(axis, angle))
            .build()
    };

    // Calculate the model-view matrix via the Affine matrix simplification
    let model_view_affine = camera_view * cube;

    // Calculate the model-view matrix as a product of two matrices.
    let camera_view = camera_view.to_matrix();
    let model_view_matrix = camera_view * cube.to_matrix();

    assert_relative_eq!(model_view_affine - model_view_matrix, Mat4::zero());
}
