pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer {
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_renderer() {
        let _r = Renderer::new();
    }
}
