mod approx;
mod cmp;
mod convert;
mod num;
mod ops;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Vec4<R> {
    pub x: R,
    pub y: R,
    pub z: R,
    pub w: R,
}

impl<R> Vec4<R> {
    pub const fn new(x: R, y: R, z: R, w: R) -> Self {
        Vec4 { x, y, z, w }
    }
}

impl<R> std::fmt::Display for Vec4<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let prettyprint = f.alternate();

        if prettyprint {
            write!(f, "[\n{},\n {},\n {},\n {}\n]", self.x, self.y, self.z, self.w)
        } else {
            write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
        }
    }
}
