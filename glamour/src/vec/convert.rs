use super::super::Vec4;

impl<R> From<[R; 4]> for Vec4<R> {
    fn from(value: [R; 4]) -> Self {
        Vec4(value)
    }
}
