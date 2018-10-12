use failure::Error;
use graphics::{
    glium::{GliumBackend, GliumRenderData},
    headless::{HeadlessBackend, HeadlessRenderData},
};
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

        // let mesh_data = load_mesh_file(mesh)?;
        // let vertex_shader = load_text_file(vs)?;
        // let fragment_shader = load_text_file(fs)?;
        // let geometry_shader = if let Some(gs) = gs {
        //     Some(load_text_file(gs)?)
        // } else {
        //     None
        // };

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

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn nothing() {
    //     let source = SourceData::
    //     Renderable::from_source()
    // }
}
