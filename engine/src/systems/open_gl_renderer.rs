use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use wrappers::{FrameTrait, DisplayTrait, EventsLoopTrait};

pub struct OpenGlRenderer<E, C, D>
where
    E: EventTrait,
    D: DisplayTrait,
{
    pub display: D,
    clear_color: (f32, f32, f32, f32),
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
}

impl<E, C, D> OpenGlRenderer<E, C, D>
where
    E: EventTrait,
    D: DisplayTrait,
{
    pub fn new(events_loop: &D::EventsLoop, title: &str, dimensions: &[u32; 2], vsync: bool, msaa: u16, clear_color: &[f32; 4]) -> Result<Self, D::Error> {
        let display = D::create(events_loop, title, dimensions, vsync, msaa)?;

        Ok(OpenGlRenderer {
            display: display,
            clear_color: (
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            ),
            phantom_e: Default::default(),
            phantom_c: Default::default(),
        })
    }
}

impl<E, C, D> SystemTrait<C, E> for OpenGlRenderer<E, C, D>
where
    E: EventTrait,
    D: DisplayTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::RENDER
    }

    fn render(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        // // Create the current frame.
        // let mut target = self.display.draw();
        // target.clear_color_and_depth(self.clear_color, 1.0);

        // // Update the scene graph.
        // ctx.scene_graph
        //     .update(&|entity, _, parent_model| {
        //         let current_model = ctx.borrow_component(entity).ok()?;
        //         Some(parent_model * current_model)
        //     })
        //     .unwrap();
        // // Get a reference to the camera.
        // // Get a reference to the Ui state
        // // Sort the nodes according to their z-value
        // let mut nodes = ctx.scene_graph.iter().collect::<Vec<_>>();
        // nodes.sort_unstable_by_key(|(_, model)| (model.translation().z / f32::EPSILON).round() as i32);

        // // Render all entities
        // for (entity, model) in nodes {
        // }

        // // Finalize the frame
        // target.finish().unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use ecs::mock::{MockEvt, MockCtx};
    use mock::MockDisplay;
    use super::*;

    #[test]
    fn new_renderer() {
        let _s: OpenGlRenderer<MockEvt, MockCtx<MockEvt>, MockDisplay> = OpenGlRenderer::new();
    }

    #[test]
    fn stage_filter() {
        let s: OpenGlRenderer<MockEvt, MockCtx<MockEvt>, MockDisplay> = OpenGlRenderer::new();
        assert_eq!(s.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    fn render() {
        let mut ctx: MockCtx<MockEvt> = Default::default();
        let mut s: OpenGlRenderer<MockEvt, MockCtx<MockEvt>, MockDisplay> = OpenGlRenderer::new();

        assert_ok!(s.render(&mut ctx, &Default::default(), &Default::default()));
    }
}
