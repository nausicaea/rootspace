pub trait RenderTrait {
    fn draw(&self);
}

pub struct Renderable;

impl RenderTrait for Renderable {
    fn draw(&self) {
        unimplemented!()
    }
}
