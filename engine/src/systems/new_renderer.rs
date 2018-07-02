use graphics::{BackendTrait, FrameTrait};
use failure::Error;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Renderer<L, R, F, E, B> {
    backend: B,
    frames: usize,
    draw_calls: usize,
    _l: PhantomData<L>,
    _r: PhantomData<R>,
    _f: PhantomData<F>,
    _e: PhantomData<E>,
}

impl<L, R, F, E, B> Renderer<L, R, F, E, B>
where
    L: AsRef<[[f32; 4]; 4]>,
    F: FrameTrait<R>,
    B: BackendTrait<E, F>,
{
    pub fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            frames: 0,
            draw_calls: 0,
            _l: PhantomData::default(),
            _r: PhantomData::default(),
            _f: PhantomData::default(),
            _e: PhantomData::default(),
        })
    }

    #[cfg(any(test, feature = "diagnostics"))]
    pub fn average_draw_calls(&self) -> f32 {
        if self.frames > 0 {
            self.draw_calls as f32 / self.frames as f32
        } else {
            0.0
        }
    }

    pub fn render(&mut self, nodes: &[(L, R)]) -> Result<(), Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            self.frames += 1;
        }
        let mut target = self.backend.create_frame();

        target.initialize([0.0, 0.0, 0.0, 1.0], 1.0);

        for (m, d) in nodes {
            #[cfg(any(test, feature = "diagnostics"))]
            {
                self.draw_calls += 1;
            }
            target.render(m, d)?;
        }

        target.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphics::RenderDataTrait;
    use graphics::headless::{HeadlessBackend as HB, HeadlessFrame as HF, HeadlessRenderData as HRD, HeadlessEventsLoop as HEL};
    use graphics::glium::{GliumBackend as GB, GliumFrame as GF, GliumRenderData as GRD, GliumEventsLoop as GEL};
    use std::f32;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    fn new_headless() {
        assert_ok!(Renderer::<MockLocation, HRD, HF, HEL, HB>::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    fn render_headless() {
        let mut r = Renderer::<MockLocation, HRD, HF, HEL, HB>::new(&Default::default(), "Title", [800, 600], false, 0).unwrap();

        let nodes = vec![
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
        ];

        assert_ok!(r.render(&nodes));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 4);
        assert_ulps_eq!(r.average_draw_calls(), 4.0, epsilon = f32::EPSILON);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn new_glium() {
        assert_ok!(Renderer::<MockLocation, GRD, GF, GEL, GB>::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn render_glium() {
        let mut r = Renderer::<MockLocation, GRD, GF, GEL, GB>::new(&Default::default(), "Title", [800, 600], false, 0).unwrap();

        let nodes = vec![
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
        ];

        assert_ok!(r.render(&nodes));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 4);
        assert_ulps_eq!(r.average_draw_calls(), 4.0, epsilon = f32::EPSILON);
    }
}
