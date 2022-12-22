use ecs::{System, SerializationName, EventQueue, ReceiverId, WithResources};
use winit::event::{KeyboardInput, VirtualKeyCode, ElementState};

use crate::{resources::{statistics::Statistics, graphics::Graphics}, events::window_event::WindowEvent};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Renderer(ReceiverId<WindowEvent>, usize);

impl WithResources for Renderer {
    fn with_resources(res: &ecs::Resources) -> Self {
        let receiver_id = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        Renderer(receiver_id, 0)
    }
}

impl SerializationName for Renderer {}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, t: &std::time::Duration, dt: &std::time::Duration) {
        let start_mark = std::time::Instant::now();
        let mut world_draw_calls: usize = 0;
        let mut ui_draw_calls: usize = 0;

        let swap_render_pipelines = res.borrow_mut::<EventQueue<WindowEvent>>()
            .receive(&self.0)
            .into_iter()
            .find_map(|e| match e {
                WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Released, virtual_keycode: Some(VirtualKeyCode::Space), ..}, .. } => Some(true),
                _ => None,
            }).unwrap_or(false);

        let mut gfx = res.borrow_mut::<Graphics>();

        if swap_render_pipelines {
            self.1 ^= 1;
            gfx.set_rp(self.1);
        }

        gfx.render().unwrap();

        let mut stats = res.borrow_mut::<Statistics>();
        stats.update_draw_calls(world_draw_calls, ui_draw_calls);
        stats.update_frame_time(start_mark.elapsed());
    }
}
