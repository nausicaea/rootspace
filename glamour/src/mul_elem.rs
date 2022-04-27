pub trait MulElem<Rhs = Self> {
    type Output;

    fn mul_elem(self, rhs: Rhs) -> Self::Output;
}
