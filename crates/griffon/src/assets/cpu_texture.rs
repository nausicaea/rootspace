use assam::{Error, LoadAsset};

#[derive(Debug)]
pub struct CpuTexture {
    pub label: Option<String>,
    pub image: image::DynamicImage,
}

impl LoadAsset for CpuTexture {
    type Output = Self;

    async fn with_path(_res: &ecs::Resources, path: &std::path::Path) -> anyhow::Result<Self::Output> {
        let label = path.file_stem().and_then(|n| n.to_str()).map(|n| n.to_owned());

        let image_format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(image::ImageFormat::from_extension)
            .ok_or(Error::UnsupportedFileFormat)?;

        let f = std::fs::File::open(path)?;
        let buf = std::io::BufReader::new(f);
        let image = image::load(buf, image_format)?;

        Ok(CpuTexture { label, image })
    }
}
