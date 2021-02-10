use snowflake::ProcessUniqueId;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId(Option<ProcessUniqueId>);

impl ShaderId {
    pub fn generate() -> Self {
        ShaderId(Some(ProcessUniqueId::new()))
    }
}

impl Default for ShaderId {
    fn default() -> Self {
        ShaderId(None)
    }
}
