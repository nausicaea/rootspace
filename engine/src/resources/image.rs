use image;
use failure::Error;
use file_manipulation::VerifyPath;
use glium::texture::RawImage2d;
use std::path::Path;
use std::fmt;

pub struct Image(image::DynamicImage);

impl Image {
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        path.ensure_extant_file()?;
        let data = image::open(path)?;

        Ok(Image(data))
    }

    pub fn new_rgba8(width: u32, height: u32) -> Self {
        Image(image::DynamicImage::new_rgba8(width, height))
    }
}

impl From<image::DynamicImage> for Image {
    fn from(value: image::DynamicImage) -> Self {
        Image(value)
    }
}

impl<'a> From<Image> for RawImage2d<'a, u8> {
    fn from(value: Image) -> Self {
        let rgba_img = value.0.to_rgba();
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
    use std::path::PathBuf;

    #[test]
    fn from_path() {
        let p = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tv-test-image.png"));
        let r: Result<Image, Error> = Image::from_path(&p);
        assert_ok!(r);
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
