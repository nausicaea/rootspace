use serde::{Serialize, Deserialize};
use ecs::{ReceiverId, Resources, WithResources, EventQueue, Entity, SerializationName, System, Storage};
use crate::{event::EngineEvent, components::Status};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusManager {
    receiver: ReceiverId<EngineEvent>,
}

impl WithResources for StatusManager {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        StatusManager { receiver }
    }
}

impl StatusManager {
    fn on_set_status(&self, res: &Resources, entity: Entity, enabled: Option<bool>, visible: Option<bool>) {
        res.borrow_components_mut::<Status>()
            .entry(entity)
            .and_modify(|s| {
                if let Some(e) = enabled {
                    s.set_enabled(e);
                }

                if let Some(v) = visible {
                    s.set_visible(v);
                }
            })
            .or_insert(Status::new(enabled.unwrap_or(true), visible.unwrap_or(true)));
    }
}

impl SerializationName for StatusManager {}

impl System for StatusManager {
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::SetStatus { entity, enabled, visible } => self.on_set_status(res, entity, enabled, visible),
                _ => (),
            }
        }
    }
}