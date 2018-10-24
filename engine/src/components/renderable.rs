use std::borrow::Borrow;

#[derive(Debug)]
pub struct Renderable<D> {
    data: D,
}

impl<D> Borrow<D> for Renderable<D> {
    fn borrow(&self) -> &D {
        &self.data
    }
}
