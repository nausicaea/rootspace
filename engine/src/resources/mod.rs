pub mod backend_resource;
pub mod scene_graph;

pub use self::{
    backend_resource::{BackendSettings, BackendResource, IndexBufferId, ShaderId, TextureId, VertexBufferId},
    scene_graph::SceneGraph,
};
