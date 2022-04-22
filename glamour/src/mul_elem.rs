pub trait MulElem<Rhs = Self> {
    type Output;

    fn mul_elementwise(self, rhs: Rhs) -> Self::Output;
}
