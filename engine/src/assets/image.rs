use super::AssetTrait;
use anyhow::Result;
use file_manipulation::FilePathBuf;
use glium::texture::RawImage2d;
use image::{self, GenericImageView};
use std::{convert::TryFrom, fmt, path::Path};

pub struct Image(image::DynamicImage);

impl Image {
    pub fn new_rgba8(width: u32, height: u32) -> Self {
        Image(image::DynamicImage::new_rgba8(width, height))
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }
}

impl AssetTrait for Image {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fp = FilePathBuf::try_from(path.as_ref())?;
        let data = image::open(fp)?;

        Ok(Image(data))
    }
}

impl From<image::DynamicImage> for Image {
    fn from(value: image::DynamicImage) -> Self {
        Image(value)
    }
}

impl<'a> From<Image> for RawImage2d<'a, u8> {
    fn from(value: Image) -> Self {
        let rgba_img = value.0.to_rgba8();
        let dimensions = rgba_img.dimensions();

        RawImage2d::from_raw_rgba_reversed(&rgba_img.into_raw(), dimensions)
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image({:?})", self.0.color())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path() {
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tv-test-image.png");
        let r: Result<Image> = Image::from_path(&p);
        assert!(r.is_ok());
    }

    #[test]
    fn new_image() {
        let _: Image = Image::new_rgba8(256, 256);
    }

    #[test]
    fn from_image() {
        let source = image::DynamicImage::new_rgba8(256, 256);
        let _: Image = source.into();
    }

    #[test]
    fn into_raw_image() {
        let img = Image::new_rgba8(256, 256);
        let _: RawImage2d<u8> = img.into();
    }
}
