use crate::components::{Camera, Projection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraSerDe {
    pub(crate) projection: Projection,
    pub(crate) dimensions: (u32, u32),
    pub(crate) fov_y: f32,
    pub(crate) frustum_z: (f32, f32),
    pub(crate) dpi_factor: f64,
}

impl From<Camera> for CameraSerDe {
    fn from(value: Camera) -> Self {
        CameraSerDe {
            projection: value.projection,
            dimensions: value.dimensions,
            fov_y: value.fov_y,
            frustum_z: value.frustum_z,
            dpi_factor: value.dpi_factor,
        }
    }
}
