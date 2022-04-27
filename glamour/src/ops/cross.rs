pub trait Cross<Rhs = Self> {
    type Output;

    fn cross(self, rhs: Rhs) -> Self::Output;
}
