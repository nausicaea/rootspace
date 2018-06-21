use context::SceneGraphTrait;
use ecs::entity::Entity;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use failure::Error;
use math::DepthOrderingTrait;
use std::f32;
use std::marker::PhantomData;
use std::ops::Mul;
use std::time::Duration;
use wrappers::{DisplayTrait, FrameTrait};

pub struct Renderer<E, C, D, V>
where
    V: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r V: Mul<Output = V>,
    E: EventTrait,
    C: SceneGraphTrait<Entity, V>,
    D: DisplayTrait,
{
    pub display: D,
    clear_color: [f32; 4],
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
    phantom_v: PhantomData<V>,
}

impl<E, C, D, V> Renderer<E, C, D, V>
where
    V: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r V: Mul<Output = V>,
    E: EventTrait,
    C: SceneGraphTrait<Entity, V>,
    D: DisplayTrait,
{
    pub fn new(
        events_loop: &D::EventsLoop,
        title: &str,
        dimensions: &[u32; 2],
        vsync: bool,
        msaa: u16,
        clear_color: [f32; 4],
    ) -> Result<Self, Error> {
        let display = D::create(events_loop, title, dimensions, vsync, msaa)?;

        Ok(Renderer {
            display: display,
            clear_color: clear_color,
            phantom_e: Default::default(),
            phantom_c: Default::default(),
            phantom_v: Default::default(),
        })
    }
}

impl<E, C, D, V> SystemTrait<C, E> for Renderer<E, C, D, V>
where
    V: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r V: Mul<Output = V>,
    E: EventTrait,
    C: SceneGraphTrait<Entity, V>,
    D: DisplayTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::RENDER
    }

    fn render(&mut self, ctx: &mut C, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        // Create the current frame.
        let mut target = self.display.create_frame();
        target.clear(&self.clear_color, 1.0);

        // Update the scene graph and sort the nodes according to their z-value.
        let _nodes = ctx.get_current_nodes(true)?;

        // Render all entities
        // for (entity, model) in nodes {
        // }

        // // Finalize the frame
        target.finalize()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ecs::mock::{MockCtx, MockEvt};
    use mock::{MockDisplay, MockModel};

    #[test]
    fn new_renderer() {
        let _s: Renderer<MockEvt, MockCtx<MockEvt>, MockDisplay, MockModel> =
            Renderer::new(&(), "Title", &[800, 600], false, 0, [1.0, 1.0, 1.0, 1.0]).unwrap();
    }

    #[test]
    fn stage_filter() {
        let s: Renderer<MockEvt, MockCtx<MockEvt>, MockDisplay, MockModel> =
            Renderer::new(&(), "Title", &[800, 600], false, 0, [1.0, 1.0, 1.0, 1.0]).unwrap();
        assert_eq!(s.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    fn render() {
        let mut ctx: MockCtx<MockEvt> = Default::default();
        let mut s: Renderer<MockEvt, MockCtx<MockEvt>, MockDisplay, MockModel> =
            Renderer::new(&(), "Title", &[800, 600], false, 0, [1.0, 1.0, 1.0, 1.0]).unwrap();

        assert_ok!(s.render(&mut ctx, &Default::default(), &Default::default()));
    }
}
