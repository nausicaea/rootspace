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

