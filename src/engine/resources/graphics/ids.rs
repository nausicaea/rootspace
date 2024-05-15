macro_rules! impl_id {
    ($($name:ident),+ $(,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name(usize);

            impl From<$name> for usize {
                fn from(value: $name) -> Self {
                    value.0
                }
            }

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    $name(value)
                }
            }
        )+
    };
}

impl_id! {
    BindGroupLayoutId,
    BindGroupId,
    BufferId,
    TextureId,
    TextureViewId,
    SamplerId,
    PipelineId,
    ShaderModuleId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(u32);

impl InstanceId {
    pub fn to_u32(&self) -> u32 {
        Into::into(*self)
    }
}

impl From<InstanceId> for u32 {
    fn from(value: InstanceId) -> Self {
        value.0
    }
}

impl From<usize> for InstanceId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl From<u32> for InstanceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
