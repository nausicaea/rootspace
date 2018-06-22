use context::SceneGraphTrait;
use ecs::database::DatabaseTrait;
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
use wrappers::glium::{DisplayTrait, FrameTrait};
use components::renderable::RenderTrait;

pub struct Renderer<E, C, D, M, R>
where
    E: EventTrait,
    C: SceneGraphTrait<Entity, M> + DatabaseTrait,
    D: DisplayTrait,
    M: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r M: Mul<Output = M>,
    R: RenderTrait + 'static,
{
    pub display: D,
    clear_color: [f32; 4],
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
    phantom_m: PhantomData<M>,
    phantom_r: PhantomData<R>,
}

impl<E, C, D, M, R> Renderer<E, C, D, M, R>
where
    E: EventTrait,
    C: SceneGraphTrait<Entity, M> + DatabaseTrait,
    D: DisplayTrait,
    M: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r M: Mul<Output = M>,
    R: RenderTrait + 'static,
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
            phantom_m: Default::default(),
            phantom_r: Default::default(),
        })
    }
}

impl<E, C, D, M, R> SystemTrait<C, E> for Renderer<E, C, D, M, R>
where
    E: EventTrait,
    C: SceneGraphTrait<Entity, M> + DatabaseTrait,
    D: DisplayTrait,
    M: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r M: Mul<Output = M>,
    R: RenderTrait + 'static,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::RENDER
    }

    fn render(&mut self, ctx: &mut C, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        // Create the current frame.
        let mut target = self.display.create_frame();
        target.clear(&self.clear_color, 1.0);

        // Update the scene graph and sort the nodes according to their z-value.
        ctx.update_graph()?;
        let nodes = ctx.get_nodes(true);

        // Render all entities
        for (entity, model) in nodes {
            if let Ok(r) = ctx.borrow::<R>(entity) {
                r.draw();
            }
        }

        // Finalize the frame
        target.finalize()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ecs::mock::{MockCtx, MockEvt};
    use mock::{MockModel, MockRenderable};
    use wrappers::glium::{HeadlessDisplay, HeadlessEventsLoop};

    #[test]
    fn new_renderer() {
        let _s: Renderer<MockEvt, MockCtx<MockEvt>, HeadlessDisplay, MockModel, MockRenderable> =
            Renderer::new(&HeadlessEventsLoop::default(), "Title", &[800, 600], false, 0, [1.0, 1.0, 1.0, 1.0]).unwrap();
    }

    #[test]
    fn stage_filter() {
        let s: Renderer<MockEvt, MockCtx<MockEvt>, HeadlessDisplay, MockModel, MockRenderable> =
            Renderer::new(&HeadlessEventsLoop::default(), "Title", &[800, 600], false, 0, [1.0, 1.0, 1.0, 1.0]).unwrap();
        assert_eq!(s.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    fn render() {
        let mut ctx: MockCtx<MockEvt> = Default::default();
        let mut s: Renderer<MockEvt, MockCtx<MockEvt>, HeadlessDisplay, MockModel, MockRenderable> =
            Renderer::new(&HeadlessEventsLoop::default(), "Title", &[800, 600], false, 0, [1.0, 1.0, 1.0, 1.0]).unwrap();

        assert_ok!(s.render(&mut ctx, &Default::default(), &Default::default()));
    }
}
