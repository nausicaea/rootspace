use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use wrappers::{FrameTrait, DisplayTrait};

pub struct OpenGlRenderer<E, C, D>
where
    E: EventTrait,
    D: DisplayTrait,
{
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
    phantom_d: PhantomData<D>,
}

impl<E, C, D> OpenGlRenderer<E, C, D>
where
    E: EventTrait,
    D: DisplayTrait,
{
    pub fn new() -> Self {
        OpenGlRenderer {
            phantom_e: Default::default(),
            phantom_c: Default::default(),
            phantom_d: Default::default(),
        }
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
    use super::*;

    #[derive(Default)]
    struct MockFrame {
        pub error_out: bool,
    }

    impl MockFrame {
        pub fn new(error_out: bool) -> Self {
            MockFrame {
                error_out: error_out,
            }
        }
    }

    impl FrameTrait for MockFrame {
        type Error = ();

        fn finalize(self) -> Result<(), Self::Error> {
            if self.error_out {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    struct MockDisplay {
        pub cause_frame_to_error: bool,
    }

    impl MockDisplay {
        pub fn new(cause_frame_to_error: bool) -> Self {
            MockDisplay {
                cause_frame_to_error: cause_frame_to_error,
            }
        }
    }

    impl DisplayTrait for MockDisplay {
        type Frame = MockFrame;

        fn create_frame(&self) -> Self::Frame {
            MockFrame::new(self.cause_frame_to_error)
        }
    }

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
