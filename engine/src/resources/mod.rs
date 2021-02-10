pub use backend_resource::index_buffer_id::IndexBufferId;
pub use backend_resource::shader_id::ShaderId;
pub use backend_resource::texture_id::TextureId;
pub use backend_resource::vertex_buffer_id::VertexBufferId;
pub use settings::Settings;

pub use self::{
    backend_resource::BackendResource,
    scene_graph::SceneGraph,
};

pub mod backend_resource;
pub mod scene_graph;
pub mod settings;

