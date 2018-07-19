use graphics::{headless::{HeadlessBackend, HeadlessRenderData}, glium::{GliumBackend, GliumRenderData}};
use failure::Error;
use std::borrow::Borrow;

#[derive(Debug)]
pub struct Renderable<D>
{
    data: D,
}

impl<D> Borrow<D> for Renderable<D> {
    fn borrow(&self) -> &D {
        &self.data
    }
}

impl From<HeadlessRenderData> for Renderable<HeadlessRenderData> {
    fn from(value: HeadlessRenderData) -> Self {
        Renderable {
            data: value,
        }
    }
}

impl From<GliumRenderData> for Renderable<GliumRenderData> {
    fn from(value: GliumRenderData) -> Self {
        Renderable {
            data: value,
        }
    }
}

#[cfg(test)]
mod tests {
}
