use std::ops::{Index, IndexMut};

use super::super::Mat4;

impl<R> Index<usize> for Mat4<R> {
    type Output = R;

    fn index(&self, index: usize) -> &Self::Output {
        let (i, j) = Self::as_2d_idx(index);
        Index::<(usize, usize)>::index(self, (i, j))
    }
}

impl<R> IndexMut<usize> for Mat4<R> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let (i, j) = Self::as_2d_idx(index);
        IndexMut::<(usize, usize)>::index_mut(self, (i, j))
    }
}

impl<R> Index<(usize, usize)> for Mat4<R> {
    type Output = R;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.0.index(index.0).index(index.1)
    }
}

impl<R> IndexMut<(usize, usize)> for Mat4<R> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.0.index_mut(index.0).index_mut(index.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_supports_1d_indexing() {
        let m: Mat4<f32> = (0..16).into_iter().map(|n| n as f32).collect();
        assert_eq!(m[2], 2.0f32);
    }

    #[test]
    fn mat4_supports_mut_1d_indexing() {
        let mut m: Mat4<f32> = (0..16).into_iter().map(|n| n as f32).collect();
        m[2] = 5.0f32;
        assert_eq!(m[2], 5.0f32);
    }

    #[test]
    fn mat4_supports_2d_indexing() {
        let m: Mat4<f32> = (0..16).into_iter().map(|n| n as f32).collect();
        assert_eq!(m[(1, 1)], 5.0f32);
    }

    #[test]
    fn mat4_supports_mut_2d_indexing() {
        let mut m: Mat4<f32> = (0..16).into_iter().map(|n| n as f32).collect();
        m[(1, 1)] = 8.0f32;
        assert_eq!(m[(1, 1)], 8.0f32);
    }
}
