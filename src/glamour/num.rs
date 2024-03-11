use super::mat::Mat4;

pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
}

pub trait ToMatrix<N> {
    fn to_matrix(&self) -> Mat4<N>;
}
