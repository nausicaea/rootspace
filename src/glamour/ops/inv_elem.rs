pub trait InvElem {
    type Output;

    fn inv_elem(self) -> Self::Output;
}
