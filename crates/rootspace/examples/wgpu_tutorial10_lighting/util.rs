use std::path::Path;
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let txt = {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/wgpu_tutorial10_lighting/res")
            .join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let data = {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/wgpu_tutorial10_lighting/res")
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}
