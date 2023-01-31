use super::Mat;

/// Matrix of 4x4 dimensions
pub type Mat4<R> = Mat<R, 4, 4>;

impl<R> Mat4<R> {
    pub const fn new(v: [[R; 4]; 4]) -> Self {
        Mat(v)
    }
}
