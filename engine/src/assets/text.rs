use crate::{
    assets::{Mesh, Vertex},
    file_manipulation::ReadPath,
    graphics::{BackendTrait, TextureTrait},
};
use failure::Error;
use rusttype::{self, gpu_cache::Cache, point, Font, PositionedGlyph, Rect as RusttypeRect, Scale};
use std::{
    borrow::{Borrow, Cow},
    fmt,
    marker::PhantomData,
    path::{Path, PathBuf},
};
use unicode_normalization::UnicodeNormalization;

pub struct Text<'a, B: BackendTrait> {
    text: String,
    dimensions: (u32, u32),
    scale: f32,
    glyphs: Vec<PositionedGlyph<'a>>,
    cache_cpu: Cache<'a>,
    cache_gpu: B::Texture,
    font: Font<'a>,
    _b: PhantomData<B>,
}

impl<'a, B: BackendTrait> fmt::Debug for Text<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Text {{ dimensions: {:?}, text: {:?}, ... }}",
            self.dimensions, self.text
        )
    }
}

impl<'a, B: BackendTrait> Text<'a, B> {
    pub fn builder() -> TextBuilder<B> {
        TextBuilder::default()
    }

    pub fn mesh(&self, width: f32) -> Mesh {
        let scale = width / self.dimensions.0 as f32;
        generate_mesh(&self.cache_cpu, &self.glyphs, self.dimensions, scale)
    }

    pub fn text(&mut self, text: &str) -> Result<(), Error> {
        let (glyphs, text_height) = layout_paragraph(&self.font, self.scale, self.dimensions.0, text);

        enqueue_glyphs(&mut self.cache_cpu, &glyphs);
        update_cache::<B, B::Texture, _>(&mut self.cache_cpu, &self.cache_gpu)?;

        self.text = text.into();
        self.dimensions.1 = text_height;
        self.glyphs = glyphs;

        Ok(())
    }

    pub fn scale(&mut self, value: f32) {
        self.scale = value;
    }

    pub fn width(&mut self, value: u32) {
        self.dimensions.0 = value;
    }
}

#[derive(Debug)]
pub struct TextBuilder<B: BackendTrait> {
    font_path: Option<PathBuf>,
    cache_gpu: Option<B::Texture>,
    font_scale: f32,
    text_width: u32,
    _b: PhantomData<B>,
}

impl<B: BackendTrait> Default for TextBuilder<B> {
    fn default() -> Self {
        TextBuilder {
            font_path: None,
            cache_gpu: None,
            font_scale: 1.0,
            text_width: 100,
            _b: PhantomData::default(),
        }
    }
}

impl<B: BackendTrait> TextBuilder<B> {
    pub fn font<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.font_path = Some(path.as_ref().into());
        self
    }

    pub fn cache(mut self, texture: B::Texture) -> Self {
        self.cache_gpu = Some(texture);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.text_width = width;
        self
    }

    pub fn layout<'a>(self, text: &str) -> Result<Text<'a, B>, Error> {
        let font_data = self
            .font_path
            .as_ref()
            .ok_or(TextRenderError::MissingFont)?
            .read_to_bytes()?;

        let cache_gpu = self.cache_gpu.ok_or(TextRenderError::MissingCache)?;

        let dims = cache_gpu.dimensions();
        let mut cache_cpu = Cache::builder().dimensions(dims.0, dims.1).build();

        let font: Font = Font::from_bytes(font_data)?;

        let (glyphs, text_height) = layout_paragraph(&font, self.font_scale, self.text_width, text);

        enqueue_glyphs(&mut cache_cpu, &glyphs);
        update_cache::<B, B::Texture, _>(&mut cache_cpu, &cache_gpu)?;

        Ok(Text {
            text: text.into(),
            dimensions: (self.text_width, text_height),
            scale: self.font_scale,
            glyphs,
            cache_cpu,
            cache_gpu,
            font,
            _b: PhantomData::default(),
        })
    }
}

#[derive(Debug, Fail)]
pub enum TextRenderError {
    #[fail(display = "You must provide a font to build an instance of Text")]
    MissingFont,
    #[fail(display = "You must provide a cache texture to build an instance of Text")]
    MissingCache,
    #[fail(display = "Font data presented to rusttype is not in a format that the library recognizes")]
    UnrecognizedFormat,
    #[fail(display = "Font data presented to rusttype was ill-formed (lacking necessary tables, for example)")]
    IllFormed,
    #[fail(
        display = "The caller tried to access the i'th font from a FontCollection, but the collection doesn't contain that many fonts"
    )]
    CollectionIndexOutOfBounds,
    #[fail(
        display = "The caller tried to convert a FontCollection into a font via into_font, but the FontCollection contains more than one font"
    )]
    CollectionContainsMultipleFonts,
    #[fail(
        display = "At least one of the queued glyphs is too big to fit into the cache, even if all other glyphs are removed"
    )]
    GlyphTooLarge,
    #[fail(
        display = "Not all of the requested glyphs can fit into the cache, even if the cache is completely cleared before the attempt"
    )]
    NoRoomForWholeQueue,
}

impl From<rusttype::Error> for TextRenderError {
    fn from(value: rusttype::Error) -> Self {
        use rusttype::Error::*;

        match value {
            UnrecognizedFormat => TextRenderError::UnrecognizedFormat,
            IllFormed => TextRenderError::IllFormed,
            CollectionIndexOutOfBounds => TextRenderError::CollectionIndexOutOfBounds,
            CollectionContainsMultipleFonts => TextRenderError::CollectionContainsMultipleFonts,
        }
    }
}

impl From<rusttype::gpu_cache::CacheWriteErr> for TextRenderError {
    fn from(value: rusttype::gpu_cache::CacheWriteErr) -> Self {
        use rusttype::gpu_cache::CacheWriteErr::*;

        match value {
            GlyphTooLarge => TextRenderError::GlyphTooLarge,
            NoRoomForWholeQueue => TextRenderError::NoRoomForWholeQueue,
        }
    }
}

/// Layouts text into a rectangle of the specified width in pixels, whith each glyph scaled by the
/// specified factor in pixels.
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
            if c == '\n' {
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

    #[cfg(any(test, feature = "diagnostics"))]
    trace!(
        "Layouted text ({} characters, {} glyphs, {}px wide, {}px high)",
        text.len(),
        glyphs.len(),
        width,
        height
    );

    (glyphs, height)
}

fn enqueue_glyphs<'a>(cache: &mut Cache<'a>, glyphs: &[PositionedGlyph<'a>]) {
    for glyph in glyphs {
        cache.queue_glyph(0, glyph.clone());
    }
}

fn update_cache<B: BackendTrait, T: TextureTrait<B>, C: Borrow<T>>(cpu: &mut Cache, gpu: &C) -> Result<(), Error> {
    cpu.cache_queued(|rect, data| {
        gpu.borrow().write(rect, Cow::Borrowed(data));
    })?;

    Ok(())
}

fn generate_mesh<'a>(cache: &Cache<'a>, glyphs: &[PositionedGlyph<'a>], text_dims: (u32, u32), scale: f32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut quad_counter = 0;
    glyphs.iter().for_each(|g| {
        if let Ok(Some((uv_rect, pos_rect))) = cache.rect_for(0, g) {
            let min = point(
                (pos_rect.min.x as f32 + (text_dims.0 as f32) / -2.0) * scale,
                ((text_dims.1 as f32) / 2.0 - pos_rect.min.y as f32) * scale,
            );
            let max = point(
                (pos_rect.max.x as f32 + (text_dims.0 as f32) / -2.0) * scale,
                ((text_dims.1 as f32) / 2.0 - pos_rect.max.y as f32) * scale,
            );
            let pos_rect = RusttypeRect { min, max };

            vertices.push(Vertex::new(
                [pos_rect.min.x, pos_rect.max.y, 0.0],
                [uv_rect.min.x, uv_rect.max.y],
                [0.0, 0.0, 1.0],
            ));
            vertices.push(Vertex::new(
                [pos_rect.min.x, pos_rect.min.y, 0.0],
                [uv_rect.min.x, uv_rect.min.y],
                [0.0, 0.0, 1.0],
            ));
            vertices.push(Vertex::new(
                [pos_rect.max.x, pos_rect.min.y, 0.0],
                [uv_rect.max.x, uv_rect.min.y],
                [0.0, 0.0, 1.0],
            ));
            vertices.push(Vertex::new(
                [pos_rect.max.x, pos_rect.max.y, 0.0],
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

    Mesh { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::{
        headless::{HeadlessBackend, HeadlessEventsLoop, HeadlessTexture},
        BackendTrait, TextureTrait,
    };

    #[test]
    fn text_builder_headless() {
        let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf");
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let cache = HeadlessTexture::empty(&backend, (512, 512)).unwrap();

        let r: Result<Text<HeadlessBackend>, Error> = Text::builder()
            .font(&font_path)
            .cache(cache)
            .scale(24.0)
            .width(100)
            .layout("Hello, World!");

        assert!(r.is_ok());
    }

    #[test]
    fn text_mesh_headless() {
        let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf");
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let cache = HeadlessTexture::empty(&backend, (512, 512)).unwrap();

        let text: Text<HeadlessBackend> = Text::builder()
            .font(&font_path)
            .cache(cache)
            .scale(24.0)
            .width(100)
            .layout("Hello, World!")
            .unwrap();

        let model_width: f32 = 2.0;
        let m: Mesh = text.mesh(model_width);

        let vertices = m.vertices.len() as u16;
        let half_model_width = model_width / 2.0;
        assert!(m.indices.iter().all(|i| i < &vertices));
        assert!(m
            .vertices
            .iter()
            .all(|v| v.position()[0] >= -half_model_width && v.position()[0] <= half_model_width));
    }

    #[test]
    fn text_scale_headless() {
        let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf");
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let cache = HeadlessTexture::empty(&backend, (512, 512)).unwrap();

        let mut text: Text<HeadlessBackend> = Text::builder()
            .font(&font_path)
            .cache(cache)
            .scale(24.0)
            .width(100)
            .layout("Hello, World!")
            .unwrap();

        text.scale(24.0f32);
    }

    #[test]
    fn text_width_headless() {
        let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf");
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let cache = HeadlessTexture::empty(&backend, (512, 512)).unwrap();

        let mut text: Text<HeadlessBackend> = Text::builder()
            .font(&font_path)
            .cache(cache)
            .scale(24.0)
            .width(100)
            .layout("Hello, World!")
            .unwrap();

        text.width(200u32);
    }

    #[test]
    fn text_update_headless() {
        let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf");
        let backend = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let cache = HeadlessTexture::empty(&backend, (512, 512)).unwrap();

        let mut text: Text<HeadlessBackend> = Text::builder()
            .font(&font_path)
            .cache(cache)
            .scale(24.0)
            .width(100)
            .layout("Hello, World!")
            .unwrap();

        assert!(text.text("Hello, you!").is_ok());
    }
}
