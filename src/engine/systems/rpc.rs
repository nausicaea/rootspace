use std::time::Duration;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use crate::ecs::resources::Resources;
use crate::ecs::system::System;
use crate::ecs::with_resources::WithResources;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rpc;

impl WithResources for Rpc {
    fn with_res(_res: &Resources) -> Result<Self, Error> {
        todo!()
    }
}

impl System for Rpc {
    fn run(&mut self, _res: &Resources, _t: &Duration, _dt: &Duration) {
        todo!()
    }
}