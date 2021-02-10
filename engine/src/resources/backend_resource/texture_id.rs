use snowflake::ProcessUniqueId;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(Option<ProcessUniqueId>);

impl TextureId {
    pub fn generate() -> Self {
        TextureId(Some(ProcessUniqueId::new()))
    }
}

impl Default for TextureId {
    fn default() -> Self {
        TextureId(None)
    }
}
