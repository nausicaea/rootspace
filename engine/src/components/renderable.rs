use failure::Error;
use graphics::{
    BackendTrait,
    glium::GliumRenderData,
    headless::{HeadlessBackend, HeadlessRenderData, HeadlessTexture},
};
use file_manipulation::ReadPath;
use text_rendering::Text;
use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum SourceData {
    Mesh {
        mesh: PathBuf,
        vertex_shader: PathBuf,
        fragment_shader: PathBuf,
        geometry_shader: Option<PathBuf>,
        diffuse_texture: PathBuf,
        normal_texture: PathBuf,
    },
    Text {
        text: String,
        font: PathBuf,
        vertex_shader: PathBuf,
        fragment_shader: PathBuf,
        geometry_shader: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub struct Renderable<D> {
    data: D,
    source: Option<SourceData>,
}

impl Renderable<HeadlessRenderData> {
    pub fn from_mesh(
        backend: &HeadlessBackend,
        mesh: &Path,
        vs: &Path,
        fs: &Path,
        gs: Option<&Path>,
        dt: &Path,
        nt: &Path,
    ) -> Result<Self, Error> {
        let source = SourceData::Mesh {
            mesh: mesh.into(),
            vertex_shader: vs.into(),
            fragment_shader: fs.into(),
            geometry_shader: gs.map(|p| p.to_path_buf()),
            diffuse_texture: dt.into(),
            normal_texture: nt.into(),
        };

        let _mesh_data = mesh.read_to_mesh()?;
        let _vertex_shader = vs.read_to_string()?;
        let _fragment_shader = vs.read_to_string()?;
        let _geomertry_shader = if let Some(gs) = gs {
            Some(gs.read_to_string()?)
        } else {
            None
        };
        let _diffuse_texture = dt.read_to_image()?;
        let _normal_texture = dt.read_to_image()?;

        Ok(Renderable {
            data: HeadlessRenderData::new(backend)?,
            source: Some(source),
        })
    }

    pub fn from_text(
        backend: &HeadlessBackend,
        text: &str,
        font: &Path,
        vs: &Path,
        fs: &Path,
        gs: Option<&Path>,
    ) -> Result<Self, Error> {
        let source = SourceData::Text {
            text: text.into(),
            font: font.into(),
            vertex_shader: vs.into(),
            fragment_shader: fs.into(),
            geometry_shader: gs.map(|p| p.to_path_buf()),
        };

        let dpi_factor = backend.dpi_factor();
        let cache_length = (512.0 * dpi_factor) as u32;

        let _text: Text<HeadlessTexture> = Text::builder()
            .font(font)
            .cache([cache_length; 2])
            .scale(24.0)
            .width(100)
            .layout(backend, text)?;

        let _vertex_shader = vs.read_to_string()?;
        let _fragment_shader = vs.read_to_string()?;
        let _geomertry_shader = if let Some(gs) = gs {
            Some(gs.read_to_string()?)
        } else {
            None
        };

        Ok(Renderable {
            data: HeadlessRenderData::new(backend)?,
            source: Some(source),
        })
    }
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
            source: None,
        }
    }
}

impl From<GliumRenderData> for Renderable<GliumRenderData> {
    fn from(value: GliumRenderData) -> Self {
        Renderable {
            data: value,
            source: None,
        }
    }
}
