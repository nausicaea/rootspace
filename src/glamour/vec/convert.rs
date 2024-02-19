use crate::glamour::vec::Vec4;

impl<R> From<[R; 4]> for Vec4<R>
where
    R: Copy,
{
    fn from(value: [R; 4]) -> Self {
        Vec4 {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}
