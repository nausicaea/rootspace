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
