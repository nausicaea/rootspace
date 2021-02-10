use snowflake::ProcessUniqueId;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexBufferId(Option<ProcessUniqueId>);

impl IndexBufferId {
    pub fn generate() -> Self {
        IndexBufferId(Some(ProcessUniqueId::new()))
    }
}

impl Default for IndexBufferId {
    fn default() -> Self {
        IndexBufferId(None)
    }
}
