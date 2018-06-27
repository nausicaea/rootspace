use components::model::DepthOrderingTrait;
use components::renderable::RenderTrait;
use context::SceneGraphTrait;
use ecs::database::DatabaseTrait;
use ecs::entity::Entity;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use failure::Error;
use std::f32;
use std::marker::PhantomData;
use std::ops::Mul;
use std::time::Duration;
use wrappers::glium::{DisplayTrait, FrameTrait};

pub struct Renderer<E, C, D, M, R>
where
    E: EventTrait,
    C: SceneGraphTrait<Entity, M> + DatabaseTrait,
    D: DisplayTrait,
    M: DepthOrderingTrait + Clone + Default + 'static,
    for<'r> &'r M: Mul<Output = M>,
    R: RenderTrait<D::Frame, M> + 'static,
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
    R: RenderTrait<D::Frame, M> + 'static,
{
    pub fn new(
        events_loop: &D::EventsLoop,
        title: &str,
        dimensions: [u32; 2],
        vsync: bool,
        msaa: u16,
        clear_color: [f32; 4],
    ) -> Result<Self, Error> {
        let display = D::create(events_loop, title, dimensions, vsync, msaa)?;

        Ok(Renderer {
            display,
            clear_color,
            phantom_e: PhantomData::default(),
            phantom_c: PhantomData::default(),
            phantom_m: PhantomData::default(),
            phantom_r: PhantomData::default(),
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
    R: RenderTrait<D::Frame, M> + 'static,
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
                r.render(&mut target, model)?;
            }
        }

        // Finalize the frame
        target.finalize()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ecs::mock::MockEvt;
    use mock::{MockCtx, MockModel, MockRenderable};
    use wrappers::glium::{HeadlessDisplay, HeadlessEventsLoop};

    #[test]
    fn new_renderer() {
        let _s: Renderer<
            MockEvt,
            MockCtx<MockEvt>,
            HeadlessDisplay,
            MockModel,
            MockRenderable,
        > = Renderer::new(
            &HeadlessEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0,
            [1.0, 1.0, 1.0, 1.0],
        ).unwrap();
    }

    #[test]
    fn stage_filter() {
        let s: Renderer<
            MockEvt,
            MockCtx<MockEvt>,
            HeadlessDisplay,
            MockModel,
            MockRenderable,
        > = Renderer::new(
            &HeadlessEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0,
            [1.0, 1.0, 1.0, 1.0],
        ).unwrap();

        assert_eq!(s.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    fn render() {
        let mut ctx: MockCtx<MockEvt> = Default::default();
        let a = ctx.create_entity();
        ctx.insert_node(a);
        ctx.add(a, MockModel::new(100.0)).unwrap();
        ctx.add(a, MockRenderable::default()).unwrap();
        let b = ctx.create_entity();
        ctx.insert_node(b);
        ctx.add(b, MockModel::new(50.0)).unwrap();
        let c = ctx.create_entity();
        ctx.add(c, MockRenderable::default()).unwrap();
        let d = ctx.create_entity();
        ctx.insert_node(d);

        let mut s: Renderer<
            MockEvt,
            MockCtx<MockEvt>,
            HeadlessDisplay,
            MockModel,
            MockRenderable,
        > = Renderer::new(
            &HeadlessEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0,
            [1.0, 1.0, 1.0, 1.0],
        ).unwrap();

        assert_ok!(s.render(&mut ctx, &Default::default(), &Default::default()));

        assert_eq!(ctx.update_graph_calls, 1);
        assert_eq!(ctx.get_nodes_calls(), 1);
        assert_eq!(
            ctx.borrow::<MockRenderable>(&a)
                .map(|c| c.render_calls())
                .unwrap(),
            1
        );
        assert_eq!(
            ctx.borrow::<MockRenderable>(&c)
                .map(|c| c.render_calls())
                .unwrap(),
            0
        );
    }
}
