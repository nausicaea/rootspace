macro_rules! impl_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Eq)]
        pub struct $name(usize, Option<&'static str>);

        impl Into<usize> for $name {
            fn into(self) -> usize {
                self.0
            }
        }

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                $name(value, None)
            }
        }

        impl From<(usize, Option<&'static str>)> for $name {
            fn from(value: (usize, Option<&'static str>)) -> Self {
                $name(value.0, value.1)
            }
        }

        impl PartialEq for $name {
            fn eq(&self, rhs: &$name) -> bool {
                self.0 == rhs.0
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.0.hash(state)
            }
        }
    };
}

impl_id!(BindGroupLayoutId);
impl_id!(BindGroupId);
impl_id!(BufferId);
impl_id!(TextureId);
impl_id!(TextureViewId);
impl_id!(SamplerId);
impl_id!(PipelineId);
impl_id!(ShaderModuleId);
