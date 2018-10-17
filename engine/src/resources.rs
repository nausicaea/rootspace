use ply::{self, CoerceTo};
use image;
use glium::texture::RawImage2d;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    normals: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3], tex_coord: [f32; 2], normals: [f32; 3]) -> Self {
        Vertex {
            position,
            tex_coord,
            normals,
        }
    }
}

implement_vertex!(Vertex, position, tex_coord, normals);

pub struct Image{
    inner: image::DynamicImage
}

impl Image {
    pub fn new_rgba8(width: u32, height: u32) -> Self {
        Image {
            inner: image::DynamicImage::new_rgba8(width, height),
        }
    }
}

impl From<image::DynamicImage> for Image {
    fn from(value: image::DynamicImage) -> Self {
        Image {
            inner: value,
        }
    }
}

impl<'a> From<Image> for RawImage2d<'a, u8> {
    fn from(value: Image) -> Self {
        let rgba_img = value.inner.to_rgba();
        let dimensions = rgba_img.dimensions();

        RawImage2d::from_raw_rgba_reversed(&rgba_img.into_raw(), dimensions)
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image{{{:?}, ...}}", self.inner.color())
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_ply(data: &ply::Ply) -> Result<Self, Error> {
        let vtx_keyword = "vertex";
        let (eidx, el) = data.element(&[vtx_keyword, "vertices"])
            .ok_or(Error::ElementNotFound(vtx_keyword))?;
        let (pos_x_idx, _) = el.scalar_property(&["x", "pos_x"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "x"))?;
        let (pos_y_idx, _) = el.scalar_property(&["y", "pos_y"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "y"))?;
        let (pos_z_idx, _) = el.scalar_property(&["z", "pos_z"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "z"))?;
        let (tex_u_idx, _) = el.scalar_property(&["s", "u", "tex_u"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "s"))?;
        let (tex_v_idx, _) = el.scalar_property(&["t", "v", "tex_v"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "t"))?;
        let (norm_x_idx, _) = el.scalar_property(&["nx", "norm_x"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "nx"))?;
        let (norm_y_idx, _) = el.scalar_property(&["ny", "norm_y"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "ny"))?;
        let (norm_z_idx, _) = el.scalar_property(&["nz", "norm_z"])
            .ok_or(Error::PropertyNotFound(vtx_keyword, "nz"))?;

        let vertices = data.generate(eidx, |props| {
            let p = [
                props[pos_x_idx].coerce().unwrap(),
                props[pos_y_idx].coerce().unwrap(),
                props[pos_z_idx].coerce().unwrap(),
            ];
            let t = [
                props[tex_u_idx].coerce().unwrap(),
                props[tex_v_idx].coerce().unwrap(),
            ];
            let n = [
                props[norm_x_idx].coerce().unwrap(),
                props[norm_y_idx].coerce().unwrap(),
                props[norm_z_idx].coerce().unwrap(),
            ];
            Vertex::new(p, t, n)
        });

        let fc_keyword = "face";
        let (eidx, el) = data.element(&[fc_keyword, "faces"])
            .ok_or(Error::ElementNotFound(fc_keyword))?;
        let (idx, _) = el.vector_property(&["vertex_index", "vertex_indices"])
            .ok_or(Error::PropertyNotFound(fc_keyword, "vertex_index"))?;

        let indices = data
            .generate(eidx, |p| CoerceTo::<Vec<u16>>::coerce(&p[idx]).unwrap())
            .into_iter()
            .flatten()
            .collect();

        Ok(Mesh { vertices, indices })
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "The element '{}' was not found", _0)]
    ElementNotFound(&'static str),
    #[fail(display = "The property '{}' was not found on element '{}'", _1, _0)]
    PropertyNotFound(&'static str, &'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vertex() {
        let _ = Vertex::new([0.0; 3], [0.0; 2], [0.0; 3]);
    }

    #[test]
    fn from_image() {
        let source = image::DynamicImage::new_rgba8(256, 256);
        let _: Image = source.into();
    }

    #[test]
    fn new_image() {
        let _ = Image::new_rgba8(256, 256);
    }
}
