pub trait Dim: 'static + PartialEq + Clone + Copy {
    const DIM: usize;

    fn as_usize(&self) -> usize;
}

macro_rules! impl_dim {
    ($name:ident, $dim:literal) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub struct $name;

        impl Dim for $name {
            const DIM: usize = $dim;

            fn as_usize(&self) -> usize {
                Self::DIM
            }
        }
    };
}

impl_dim!(D1, 1);
impl_dim!(D2, 2);
impl_dim!(D3, 3);
impl_dim!(D4, 4);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dx_implements_dim() {
        assert_eq!(D1::DIM, 1usize);
        assert_eq!(D2::DIM, 2usize);
        assert_eq!(D3::DIM, 3usize);
        assert_eq!(D4::DIM, 4usize);
        assert_eq!(D1.as_usize(), 1usize);
        assert_eq!(D2.as_usize(), 2usize);
        assert_eq!(D3.as_usize(), 3usize);
        assert_eq!(D4.as_usize(), 4usize);
    }
}
