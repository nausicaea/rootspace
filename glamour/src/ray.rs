use num_traits::Num;
use super::mat::Vec4;

#[derive(Debug, PartialEq, Clone)]
pub struct Ray<R> {
    pub o: Vec4<R>,
    pub d: Vec4<R>,
}

impl<R> Ray<R> {
    pub fn new(origin: Vec4<R>, direction: Vec4<R>) -> Self {
        Ray {
            o: origin,
            d: direction,
        }
    }
}

impl<R> Ray<R> 
where
    R: Num + Copy,
{
    pub fn at(&self, position: &R) -> Vec4<R> {
        &self.o + &(&self.d * position)
    }
}

