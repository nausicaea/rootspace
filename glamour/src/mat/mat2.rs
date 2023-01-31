use super::Mat;

/// Matrix of 2x2 dimensions
pub type Mat2<R> = Mat<R, 2, 2>;

impl<R> Mat2<R> {
    pub const fn new(v: [[R; 2]; 2]) -> Self {
        Mat(v)
    }
}
