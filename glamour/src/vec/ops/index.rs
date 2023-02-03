use super::super::Vec4;

impl<R> std::ops::Index<usize> for Vec4<R> {
    type Output = R;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
