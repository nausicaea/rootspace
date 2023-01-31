use super::Mat;

/// Matrix of 3x3 dimensions
pub type Mat3<R> = Mat<R, 3, 3>;

impl<R> Mat3<R> {
    pub const fn new(v: [[R; 3]; 3]) -> Self {
        Mat(v)
    }
}
