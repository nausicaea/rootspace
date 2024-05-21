use super::super::Vec4;

impl<R> std::ops::Index<usize> for Vec4<R> {
    type Output = R;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            i => panic!("index out of bounds: the len is {} but the index is {}", 4, i),
        }
    }
}

impl<R> std::ops::IndexMut<usize> for Vec4<R> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            i => panic!("index out of bounds: the len is {} but the index is {}", 4, i),
        }
    }
}
