use ecs::{EventQueue, RegAdd};
use crate::{
    events::winit_mappings::WindowEvent,
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
};

pub type Resources<S> = RegAdd![AssetDatabase, Graphics, EventQueue<WindowEvent>, Statistics, S];
