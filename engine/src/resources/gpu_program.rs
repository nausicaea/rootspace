use std::path::{Path, PathBuf};
use file_manipulation::ReadPath;

#[derive(Debug)]
pub struct GpuProgram {
}

impl GpuProgram {
    pub fn builder() -> GpuProgramBuilder {
        GpuProgramBuilder::default()
    }
}

#[derive(Debug)]
pub struct GpuProgramBuilder {
    vs: Option<PathBuf>,
    fs: Option<PathBuf>,
}

impl Default for GpuProgramBuilder {
    fn default() -> Self {
        GpuProgramBuilder {
            vs: None,
            fs: None,
        }
    }
}

impl GpuProgramBuilder {
    pub fn vertex_shader(mut self, path: &Path) -> Self {
        self.vs = Some(path.into());
        self
    }

    pub fn fragment_shader(mut self, path: &Path) -> Self {
        self.fs = Some(path.into());
        self
    }

    pub fn compile(self) -> Result<GpuProgram, GpuProgramError> {
        let vs = self.vs
            .ok_or(GpuProgramError::MissingVertexShader)
            .map(|p| p.read_to_string())?;

        let fs = self.fs
            .ok_or(GpuProgramError::MissingFragmentShader)
            .map(|p| p.read_to_string())?;

        Ok(GpuProgram {})
    }
}

#[derive(Debug, Fail)]
pub enum GpuProgramError {
    #[fail(display = "You cannot build a GpuProgram without a vertex shader")]
    MissingVertexShader,
    #[fail(display = "You cannot build a GpuProgram without a fragment shader")]
    MissingFragmentShader,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let vs = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vertex.glsl"));
        let fs = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fragment.glsl"));
        let r: Result<GpuProgram, GpuProgramError> = GpuProgram::builder()
            .vertex_shader(&vs)
            .fragment_shader(&fs)
            .compile();

        assert_ok!(r);
    }
}
