pub trait Norm {
    type Output;

    fn norm(self) -> Self::Output;
}
