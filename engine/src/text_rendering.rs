use std::fmt;
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use file_manipulation::{self, ReadPath};
use rusttype::{self, Rect as RusttypeRect, point, vector, Font, Scale, PositionedGlyph, gpu_cache::Cache};
use unicode_normalization::UnicodeNormalization;
use resources::{Vertex, Mesh};
use graphics::TextureTrait;

#[derive(Debug)]
pub struct TextBuilder<T> {
    font_path: Option<PathBuf>,
    cache_size: Option<[u32; 2]>,
    cache_texture: Option<T>,
    font_scale: f32,
    text_width: u32,
}

impl<T> TextBuilder<T>
where
    T: TextureTrait,
{
    pub fn new() -> Self {
        TextBuilder {
            font_path: None,
            cache_size: None,
            cache_texture: None,
            font_scale: 1.0,
            text_width: 100,
        }
    }

    pub fn font(mut self, path: &Path) -> Self {
        self.font_path = Some(path.into());
        self
    }

    pub fn cache(mut self, texture: T) -> Self {
        self.cache_size = Some([texture.width(), texture.height()]);
        self.cache_texture = Some(texture);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    pub fn text_width(mut self, width: u32) -> Self {
        self.text_width = width;
        self
    }

    pub fn layout(self, text: &str) -> Result<Text<T>, Error> {
        let font_data = self.font_path
            .as_ref()
            .ok_or(Error::MissingFont)?
            .read_to_bytes()?;

        let mut cache_cpu = self.cache_size
            .map(|dims| Cache::builder()
                .dimensions(dims[0], dims[1])
                .build())
            .ok_or(Error::MissingCache)?;

        let cache_gpu = self.cache_texture
            .ok_or(Error::MissingCache)?;

        let font: Font = Font::from_bytes(font_data)?;

        let (glyphs, text_height) = layout_paragraph(&font, self.font_scale, self.text_width, text);

        enqueue_glyphs(&mut cache_cpu, &glyphs);
        update_cache(&mut cache_cpu, &cache_gpu)?;

        Ok(Text {
            text: text.into(),
            dimensions: [self.text_width, text_height],
            scale: self.font_scale,
            glyphs,
            cache_cpu,
            cache_gpu,
            font,
        })
    }
}

pub struct Text<'a, T> {
    text: String,
    dimensions: [u32; 2],
    scale: f32,
    glyphs: Vec<PositionedGlyph<'a>>,
    cache_cpu: Cache<'a>,
    cache_gpu: T,
    font: Font<'a>,
}

impl<'a, T> fmt::Debug for Text<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Text {{ dimensions: {:?}, text: {:?}, ... }}", self.dimensions, self.text)
    }
}

impl<'a, T> Text<'a, T>
where
    T: TextureTrait,
{
    pub fn builder() -> TextBuilder<T> {
        TextBuilder::new()
    }

    pub fn mesh(&self, screen_dimensions: [u32; 2]) -> Mesh {
        generate_mesh(&self.cache_cpu, screen_dimensions, self.dimensions, &self.glyphs)
    }

    pub fn text(&mut self, value: &str) -> Result<(), Error> {
        let (glyphs, text_height) = layout_paragraph(&self.font, self.scale, self.dimensions[0], value);

        enqueue_glyphs(&mut self.cache_cpu, &glyphs);
        update_cache(&mut self.cache_cpu, &self.cache_gpu)?;

        self.text = value.into();
        self.dimensions[1] = text_height;
        self.glyphs = glyphs;

        Ok(())
    }

    pub fn scale(&mut self, value: f32) {
        self.scale = value;
    }

    pub fn width(&mut self, value: u32) {
        self.dimensions[0] = value;
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "You must provide a font to build an instance of Text")]
    MissingFont,
    #[fail(display = "You must provide a cache texture to build an instance of Text")]
    MissingCache,
    #[fail(display = "Font data presented to rusttype is not in a format that the library recognizes")]
    UnrecognizedFormat,
    #[fail(display = "Font data presented to rusttype was ill-formed (lacking necessary tables, for example)")]
    IllFormed,
    #[fail(display = "The caller tried to access the i'th font from a FontCollection, but the collection doesn't contain that many fonts")]
    CollectionIndexOutOfBounds,
    #[fail(display = "The caller tried to convert a FontCollection into a font via into_font, but the FontCollection contains more than one font")]
    CollectionContainsMultipleFonts,
    #[fail(display = "At least one of the queued glyphs is too big to fit into the cache, even if all other glyphs are removed")]
    GlyphTooLarge,
    #[fail(display = "Not all of the requested glyphs can fit into the cache, even if the cache is completely cleared before the attempt")]
    NoRoomForWholeQueue,
    #[fail(display = "{}", _0)]
    FileError(#[cause] file_manipulation::FileError),
}

impl From<file_manipulation::FileError> for Error {
    fn from(value: file_manipulation::FileError) -> Self {
        Error::FileError(value)
    }
}

impl From<rusttype::Error> for Error {
    fn from(value: rusttype::Error) -> Self {
        use rusttype::Error::*;

        match value {
            UnrecognizedFormat => Error::UnrecognizedFormat,
            IllFormed => Error::IllFormed,
            CollectionIndexOutOfBounds => Error::CollectionIndexOutOfBounds,
            CollectionContainsMultipleFonts => Error::CollectionContainsMultipleFonts,
        }
    }
}

impl From<rusttype::gpu_cache::CacheWriteErr> for Error {
    fn from(value: rusttype::gpu_cache::CacheWriteErr) -> Self {
        use rusttype::gpu_cache::CacheWriteErr::*;

        match value {
            GlyphTooLarge => Error::GlyphTooLarge,
            NoRoomForWholeQueue => Error::NoRoomForWholeQueue,
        }
    }
}

fn layout_paragraph<'a>(font: &Font<'a>, scale: f32, width: u32, text: &str) -> (Vec<PositionedGlyph<'a>>, u32) {
    let mut glyphs = Vec::new();
    let scale = Scale::uniform(scale);
    let v_metrics = font.v_metrics(scale);
    let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let mut caret = point(0.0, v_metrics.ascent);
    let caret_origin = caret;
    let mut last_glyph_id = None;
    for c in text.nfc() {
        if c.is_control() {
            if let '\n' = c {
                caret = point(0.0, caret.y + advance_height);
            }
            continue;
        }
        let base_glyph = font.glyph(c);
        if let Some(id) = last_glyph_id.take() {
            caret.x += font.pair_kerning(scale, id, base_glyph.id());
        }
        last_glyph_id = Some(base_glyph.id());
        let mut glyph = base_glyph.scaled(scale).positioned(caret);
        if let Some(bb) = glyph.pixel_bounding_box() {
            if bb.max.x > width as i32 {
                caret = point(0.0, caret.y + advance_height);
                glyph = glyph.into_unpositioned().positioned(caret);
                last_glyph_id = None;
            }
        }
        caret.x += glyph.unpositioned().h_metrics().advance_width;
        glyphs.push(glyph);
    }

    let height = (caret.y - caret_origin.y + advance_height).ceil() as u32;

    (glyphs, height)
}

fn enqueue_glyphs<'a>(cache: &mut Cache<'a>, glyphs: &[PositionedGlyph<'a>]) {
    for glyph in glyphs {
        cache.queue_glyph(0, glyph.clone());
    }
}

fn update_cache<T: TextureTrait>(cpu: &mut Cache, gpu: &T) -> Result<(), Error> {
    cpu.cache_queued(|rect, data| {
        gpu.write(
            rect.min.x,
            rect.min.y,
            rect.width(),
            rect.height(),
            Cow::Borrowed(data),
        );
    })?;

    Ok(())
}

fn generate_mesh(
    cache: &Cache,
    screen_dims: [u32; 2],
    text_dims: [u32; 2],
    glyphs: &[PositionedGlyph],
) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let screen_dims = [screen_dims[0] as f32, screen_dims[1] as f32];
    let text_dims = [text_dims[0] as f32, text_dims[1] as f32];
    let origin = point(-text_dims[0] / 2.0, text_dims[1] / 2.0);

    let mut quad_counter = 0;
    glyphs.iter().for_each(|g| {
        if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, g) {
            let ndc_rect = RusttypeRect {
                min: origin
                    + vector(
                        screen_rect.min.x as f32 / screen_dims[0],
                        -screen_rect.min.y as f32 / screen_dims[1],
                    ),
                max: origin
                    + vector(
                        screen_rect.max.x as f32 / screen_dims[0],
                        -screen_rect.max.y as f32 / screen_dims[1],
                    ),
            };

            vertices.push(Vertex::new(
                [ndc_rect.min.x, ndc_rect.max.y, 0.0],
                [uv_rect.min.x, uv_rect.max.y],
                [0.0, 0.0, 1.0],
            ));
            vertices.push(Vertex::new(
                [ndc_rect.min.x, ndc_rect.min.y, 0.0],
                [uv_rect.min.x, uv_rect.min.y],
                [0.0, 0.0, 1.0],
            ));
            vertices.push(Vertex::new(
                [ndc_rect.max.x, ndc_rect.min.y, 0.0],
                [uv_rect.max.x, uv_rect.min.y],
                [0.0, 0.0, 1.0],
            ));
            vertices.push(Vertex::new(
                [ndc_rect.max.x, ndc_rect.max.y, 0.0],
                [uv_rect.max.x, uv_rect.max.y],
                [0.0, 0.0, 1.0],
            ));

            let stride = quad_counter * 4;
            indices.push(stride);
            indices.push(stride + 1);
            indices.push(stride + 2);
            indices.push(stride + 2);
            indices.push(stride + 3);
            indices.push(stride);
            quad_counter += 1;
        }
    });

    Mesh {vertices, indices}
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphics::{BackendTrait, headless::{HeadlessBackend, HeadlessEventsLoop, HeadlessTexture}, glium::{GliumBackend, GliumEventsLoop, GliumTexture}};

    #[test]
    fn text_builder_headless() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = HeadlessTexture::empty(&backend, 512, 512).unwrap();

        let r = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!");

        assert_ok!(r);
    }

    #[test]
    fn text_mesh_headless() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = HeadlessTexture::empty(&backend, 512, 512).unwrap();

        let text = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!")
            .unwrap();

        let _: Mesh = text.mesh([1024, 768]);
    }

    #[test]
    fn text_scale_headless() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = HeadlessTexture::empty(&backend, 512, 512).unwrap();

        let mut text = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!")
            .unwrap();

        text.scale(24.0f32);
    }

    #[test]
    fn text_width_headless() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = HeadlessTexture::empty(&backend, 512, 512).unwrap();

        let mut text = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!")
            .unwrap();

        text.width(200u32);
    }

    #[test]
    fn text_update_headless() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = HeadlessTexture::empty(&backend, 512, 512).unwrap();

        let mut text = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!")
            .unwrap();

        assert_ok!(text.text("Hello, you!"));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn text_builder_glium() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = GliumTexture::empty(&backend, 512, 512).unwrap();

        let r = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!");

        assert_ok!(r);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn text_update_glium() {
        let font_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf"));
        let backend = GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let tex = GliumTexture::empty(&backend, 512, 512).unwrap();

        let mut text = Text::builder()
            .font(&font_path)
            .cache(tex)
            .scale(24.0)
            .text_width(100)
            .layout("Hello, World!")
            .unwrap();

        assert_ok!(text.text("Hello, you!"));
    }
}
