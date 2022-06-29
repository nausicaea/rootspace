use std::ops::{Index, IndexMut};

use super::super::Mat;

impl<R, const I: usize, const J: usize> Index<usize> for Mat<R, I, J> {
    type Output = R;

    fn index(&self, index: usize) -> &Self::Output {
        let (i, j) = Self::as_2d_idx(index);
        Index::<(usize, usize)>::index(self, (i, j))
    }
}

impl<R, const I: usize, const J: usize> IndexMut<usize> for Mat<R, I, J> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let (i, j) = Self::as_2d_idx(index);
        IndexMut::<(usize, usize)>::index_mut(self, (i, j))
    }
}

impl<R, const I: usize, const J: usize> Index<(usize, usize)> for Mat<R, I, J> {
    type Output = R;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.0.index(index.0).index(index.1)
    }
}

impl<R, const I: usize, const J: usize> IndexMut<(usize, usize)> for Mat<R, I, J> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.0.index_mut(index.0).index_mut(index.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat_supports_1d_indexing() {
        let m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[2], 3.0f32);
    }

    #[test]
    fn mat_supports_mut_1d_indexing() {
        let mut m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[2] = 5.0f32;
        assert_eq!(m[2], 5.0f32);
    }

    #[test]
    fn mat_supports_2d_indexing() {
        let m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[(1, 1)], 4.0f32);
    }

    #[test]
    fn mat_supports_mut_2d_indexing() {
        let mut m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[(1, 1)] = 5.0f32;
        assert_eq!(m[(1, 1)], 5.0f32);
    }
}
