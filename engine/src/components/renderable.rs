use failure::Error;
use file_manipulation::ReadPath;
use graphics::{
    glium::GliumRenderData,
    headless::{HeadlessBackend, HeadlessRenderData, HeadlessTexture},
    BackendTrait,
};
use resources::{Image, Mesh, Text};
use std::{
    borrow::Borrow,
    marker::PhantomData,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Renderable<D> {
    data: D,
}

impl Renderable<HeadlessRenderData> {
    pub fn builder() -> RenderableBuilder<HeadlessRenderData> {
        RenderableBuilder::default()
    }
}

impl<D> Borrow<D> for Renderable<D> {
    fn borrow(&self) -> &D {
        &self.data
    }
}

impl From<HeadlessRenderData> for Renderable<HeadlessRenderData> {
    fn from(value: HeadlessRenderData) -> Self {
        Renderable { data: value }
    }
}

impl From<GliumRenderData> for Renderable<GliumRenderData> {
    fn from(value: GliumRenderData) -> Self {
        Renderable { data: value }
    }
}

#[derive(Debug)]
pub struct RenderableBuilder<D> {
    _d: PhantomData<D>,
}

impl<D> Default for RenderableBuilder<D> {
    fn default() -> Self {
        RenderableBuilder {
            _d: PhantomData::default(),
        }
    }
}

impl RenderableBuilder<GliumRenderData> {
    pub fn create_text(self) -> Result<Renderable<GliumRenderData>, RenderableError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum RenderableError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_text_headless() {
        let r: Result<Renderable<HeadlessRenderData>, RenderableError> = Renderable::builder().create_text();

        assert_ok!(r);
    }
}
