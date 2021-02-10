use snowflake::ProcessUniqueId;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexBufferId(Option<ProcessUniqueId>);

impl VertexBufferId {
    pub fn generate() -> Self {
        VertexBufferId(Some(ProcessUniqueId::new()))
    }
}

impl Default for VertexBufferId {
    fn default() -> Self {
        VertexBufferId(None)
    }
}
