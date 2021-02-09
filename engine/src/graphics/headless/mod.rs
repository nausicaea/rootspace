use super::{
    BackendTrait, EventTrait, FrameTrait, IndexBufferTrait, ShaderTrait, TextureTrait, Vertex,
    VertexBufferTrait,
};
use crate::{
    assets::Image, components::Renderable, event::EngineEvent, geometry::rect::Rect,
    resources::BackendResource,
};
use anyhow::Result;
#[cfg(any(test, debug_assertions))]
use log::{debug, trace};
use std::{borrow::Cow, convert::TryInto};

#[derive(Debug, Clone, Default, Copy)]
pub struct HeadlessEvent;

impl EventTrait for HeadlessEvent {}

impl TryInto<EngineEvent> for HeadlessEvent {
    type Error = ();

    fn try_into(self) -> Result<EngineEvent, Self::Error> {
        Err(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessTexture {
    dimensions: (u32, u32),
}

impl TextureTrait<HeadlessBackend> for HeadlessTexture {
    fn empty(_backend: &HeadlessBackend, dimensions: (u32, u32)) -> Result<Self> {
        #[cfg(any(test, debug_assertions))]
        debug!("Created an empty texture (dims={:?})", dimensions);

        Ok(HeadlessTexture { dimensions })
    }

    fn from_image(_backend: &HeadlessBackend, image: Image) -> Result<Self> {
        #[cfg(any(test, debug_assertions))]
        debug!(
            "Created a texture from an image (dims={:?})",
            image.dimensions()
        );

        Ok(HeadlessTexture {
            dimensions: image.dimensions(),
        })
    }

    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    #[cfg_attr(not(test), allow(unused_variables))]
    fn write<R: Into<Rect<u32>>>(&self, rect: R, _data: Cow<[u8]>) {
        #[cfg(any(test, debug_assertions))]
        {
            let rect = rect.into();
            assert!(rect.max().x() < self.dimensions.0);
            assert!(rect.max().y() < self.dimensions.1);

            trace!("Wrote to the texture at {}", rect);
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeadlessShader;

impl ShaderTrait<HeadlessBackend> for HeadlessShader {
    fn from_source<S: AsRef<str>>(_backend: &HeadlessBackend, _vs: S, _fs: S) -> Result<Self> {
        Ok(HeadlessShader)
    }
}

#[derive(Debug, Clone)]
pub struct HeadlessVertexBuffer;

impl VertexBufferTrait<HeadlessBackend> for HeadlessVertexBuffer {
    fn from_vertices(_backend: &HeadlessBackend, _vertices: &[Vertex]) -> Result<Self> {
        Ok(HeadlessVertexBuffer)
    }
}

#[derive(Debug, Clone)]
pub struct HeadlessIndexBuffer;

impl IndexBufferTrait<HeadlessBackend> for HeadlessIndexBuffer {
    fn from_indices(_backend: &HeadlessBackend, _indices: &[u16]) -> Result<Self> {
        Ok(HeadlessIndexBuffer)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessFrame;

impl FrameTrait<HeadlessBackend> for HeadlessFrame {
    fn initialize(&mut self, _color: [f32; 4], _depth: f32) {}

    fn render<T: AsRef<[[f32; 4]; 4]>>(
        &mut self,
        _transform: &T,
        _factory: &BackendResource<HeadlessBackend>,
        _data: &Renderable,
    ) -> Result<()> {
        Ok(())
    }

    fn finalize(self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessBackend {
    dimensions: (u32, u32),
}

impl BackendTrait for HeadlessBackend {
    type Event = HeadlessEvent;
    type Frame = HeadlessFrame;
    type IndexBuffer = HeadlessIndexBuffer;
    type Shader = HeadlessShader;
    type Texture = HeadlessTexture;
    type VertexBuffer = HeadlessVertexBuffer;

    #[allow(unused_variables)]
    fn new<S: AsRef<str>>(
        title: S,
        dimensions: (u32, u32),
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self> {
        #[cfg(any(test, debug_assertions))]
        debug!(
            "Created a headless backend (title='{}', dims={:?})",
            title.as_ref(),
            dimensions
        );

        Ok(HeadlessBackend { dimensions })
    }

    fn poll_events<F: FnMut(HeadlessEvent)>(&mut self, _f: F) {}

    fn create_frame(&self) -> HeadlessFrame {
        HeadlessFrame::default()
    }

    fn dpi_factor(&self) -> f64 {
        1.0
    }

    fn physical_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::BackendSettings;
    use file_manipulation::DirPathBuf;
    use std::convert::TryFrom;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    fn backend() {
        assert!(HeadlessBackend::new("Title", (800, 600), false, 0).is_ok());
    }

    #[test]
    fn dpi_factor() {
        let b = HeadlessBackend::new("Title", (800, 600), false, 0).unwrap();

        assert_eq!(b.dpi_factor(), 1.0f64);
    }

    #[test]
    fn frame() {
        let resource_path = DirPathBuf::try_from(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/rootspace")).unwrap();
        let mut f = BackendSettings::new("Title", (800, 600), false, 0, resource_path)
            .build::<HeadlessBackend>()
            .unwrap();

        let vertices = f
            .create_vertex_buffer(&[
                Vertex::new([0.0, 0.5, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
                Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
                Vertex::new([0.5, -0.5, 0.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
            ])
            .unwrap();

        let indices = f.create_index_buffer(&[0, 1, 2]).unwrap();

        let shader = f
            .create_source_shader(
                r#"
                    #version 330 core

                    uniform mat4 transform;

                    layout (location = 0) in vec3 position;
                    layout (location = 1) in vec2 tex_coord;
                    layout (location = 2) in vec3 normals;

                    void main() {
                            gl_Position = transform * vec4(position, 1.0);
                    }
                    "#,
                r#"
                    #version 330 core

                    uniform vec2 dimensions;
                    uniform sampler2D diffuse_texture;
                    // uniform sampler2D normal_texture;

                    out vec4 color;

                    void main() {
                            color = vec4(0.3, 0.12, 0.9, 1.0);
                    }
                    "#,
            )
            .unwrap();

        let diffuse_texture = f.create_empty_texture((32, 32)).unwrap();
        let normal_texture = Some(f.create_empty_texture((32, 32)).unwrap());

        let data = Renderable::new(vertices, indices, shader, diffuse_texture, normal_texture);

        let mut frame: HeadlessFrame = f.create_frame();
        frame.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert!(frame
            .render(&MockLocation::default(), &mut f, &data)
            .is_ok());
        let r = frame.finalize();
        assert!(r.is_ok());
    }
}
