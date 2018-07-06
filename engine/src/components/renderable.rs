use graphics::{RenderDataTrait, headless::{HeadlessBackend, HeadlessRenderData}, glium::{GliumBackend, GliumRenderData}};
use failure::Error;
use std::borrow::Borrow;

#[derive(Debug)]
pub struct Renderable<D>
{
    data: D,
}

impl RenderDataTrait<HeadlessBackend> for Renderable<HeadlessRenderData> {
    fn triangle(b: &HeadlessBackend) -> Result<Self, Error> {
        Ok(Renderable {
            data: HeadlessRenderData::triangle(b)?,
        })
    }

    fn cube(b: &HeadlessBackend) -> Result<Self, Error> {
        Ok(Renderable {
            data: HeadlessRenderData::cube(b)?,
        })
    }
}

impl RenderDataTrait<GliumBackend> for Renderable<GliumRenderData> {
    fn triangle(b: &GliumBackend) -> Result<Self, Error> {
        Ok(Renderable {
            data: GliumRenderData::triangle(b)?,
        })
    }

    fn cube(b: &GliumBackend) -> Result<Self, Error> {
        Ok(Renderable {
            data: GliumRenderData::cube(b)?,
        })
    }
}

impl<D> Borrow<D> for Renderable<D> {
    fn borrow(&self) -> &D {
        &self.data
    }
}

#[cfg(test)]
mod tests {
}
