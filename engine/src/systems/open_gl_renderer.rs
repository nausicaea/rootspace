use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use ecs::entity::Entity;
use ecs::event::{EventTrait, EventManagerTrait};
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use hierarchy::Hierarchy;
use components::model::Model;

pub struct OpenGlRenderer<E, C>
where
    E: EventTrait,
{
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
}

impl<E, C> OpenGlRenderer<E, C>
where
    E: EventTrait,
{
    pub fn new() -> Self {
        OpenGlRenderer {
            phantom_e: Default::default(),
            phantom_c: Default::default(),
        }
    }
}

impl<E, C> SystemTrait<C, E> for OpenGlRenderer<E, C>
where
    E: EventTrait,
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
    use super::*;

    #[test]
    fn new_renderer() {
        let _s: OpenGlRenderer<MockEvt, MockCtx<MockEvt>> = OpenGlRenderer::new();
    }

    #[test]
    fn stage_filter() {
        let s: OpenGlRenderer<MockEvt, MockCtx<MockEvt>> = OpenGlRenderer::new();
        assert_eq!(s.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    fn render() {
        let mut ctx: MockCtx<MockEvt> = Default::default();
        let mut s: OpenGlRenderer<MockEvt, MockCtx<MockEvt>> = OpenGlRenderer::new();

        assert_ok!(s.render(&mut ctx, &Default::default(), &Default::default()));
    }
}
