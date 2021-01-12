use ecs::RegAdd;
use ecs::Component;
use ecs::EventQueue;
use crate::components::{Camera, Info, Model, Renderable, Status, UiModel};
use crate::resources::{BackendSettings, SceneGraph};
use crate::event::EngineEvent;

pub type TypeRegistry<RR> = RegAdd![
    <Info as Component>::Storage,
    <Status as Component>::Storage,
    <Camera as Component>::Storage,
    <Renderable as Component>::Storage,
    <UiModel as Component>::Storage,
    <Model as Component>::Storage,
    SceneGraph<UiModel>,
    SceneGraph<Model>,
    EventQueue<EngineEvent>,
    BackendSettings,
    RR
];
