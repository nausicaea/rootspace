use crate::ecs::resources::Resources;
use crate::ecs::system::System;
use crate::ecs::with_resources::WithResources;
use anyhow::Error;
use std::time::Duration;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Rpc;

impl WithResources for Rpc {
    async fn with_res(_res: &Resources) -> Result<Self, Error> {
        todo!()
    }
}

#[async_trait]
impl System for Rpc {
    async fn run(&mut self, _res: &Resources, _t: Duration, _dt: Duration) {
        todo!()
    }
}
