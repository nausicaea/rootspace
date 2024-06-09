use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use ecs::{
    event_queue::{receiver_id::ReceiverId, EventQueue},
    resources::Resources,
    system::System,
    with_resources::WithResources,
};
use glamour::{affine::Affine, quat::Quat, vec::Vec4};
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{Camera, Transform};

#[derive(Debug)]
pub struct CameraController {
    receiver: ReceiverId<WindowEvent>,
    physical_key_to_dof: HashMap<PhysicalKey, (Signum, DoF)>,
}

impl WithResources for CameraController {
    #[tracing::instrument(skip_all)]
    async fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        Ok(CameraController {
            receiver: res.write::<EventQueue<WindowEvent>>().subscribe::<Self>(),
            physical_key_to_dof: [
                (PhysicalKey::Code(KeyCode::KeyW), (Signum::Negative, DoF::Z)),
                (PhysicalKey::Code(KeyCode::KeyS), (Signum::Positive, DoF::Z)),
                (PhysicalKey::Code(KeyCode::KeyA), (Signum::Negative, DoF::X)),
                (PhysicalKey::Code(KeyCode::KeyD), (Signum::Positive, DoF::X)),
                (PhysicalKey::Code(KeyCode::KeyZ), (Signum::Negative, DoF::Y)),
                (PhysicalKey::Code(KeyCode::KeyC), (Signum::Positive, DoF::Y)),
                (PhysicalKey::Code(KeyCode::KeyQ), (Signum::Negative, DoF::ZX)),
                (PhysicalKey::Code(KeyCode::KeyE), (Signum::Positive, DoF::ZX)),
                (PhysicalKey::Code(KeyCode::ArrowLeft), (Signum::Negative, DoF::XY)),
                (PhysicalKey::Code(KeyCode::ArrowRight), (Signum::Positive, DoF::XY)),
                (PhysicalKey::Code(KeyCode::ArrowUp), (Signum::Positive, DoF::YZ)),
                (PhysicalKey::Code(KeyCode::ArrowDown), (Signum::Negative, DoF::YZ)),
            ]
            .into_iter()
            .collect(),
        })
    }
}

#[async_trait]
impl System for CameraController {
    #[tracing::instrument(skip_all)]
    async fn run(&mut self, res: &Resources, _t: Duration, dt: Duration) {
        let dx = dt.as_secs_f32() * 1.00;

        let events = res.write::<EventQueue<WindowEvent>>().receive(&self.receiver);

        let mut delta_transform: Affine<f32> = Affine::identity();
        for event in events {
            if let WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key, .. },
                ..
            } = event
            {
                if let Some((signum, dof)) = self.physical_key_to_dof.get(&physical_key) {
                    match (signum, dof) {
                        (Signum::Positive, DoF::X) => delta_transform.t.x = dx,
                        (Signum::Negative, DoF::X) => delta_transform.t.x = -dx,
                        (Signum::Positive, DoF::Y) => delta_transform.t.y = dx,
                        (Signum::Negative, DoF::Y) => delta_transform.t.y = -dx,
                        (Signum::Positive, DoF::Z) => delta_transform.t.z = dx,
                        (Signum::Negative, DoF::Z) => delta_transform.t.z = -dx,
                        (Signum::Positive, DoF::XY) => delta_transform.o = Quat::with_axis_angle(Vec4::z(), dx),
                        (Signum::Negative, DoF::XY) => delta_transform.o = Quat::with_axis_angle(Vec4::z(), -dx),
                        (Signum::Positive, DoF::YZ) => delta_transform.o = Quat::with_axis_angle(Vec4::x(), dx),
                        (Signum::Negative, DoF::YZ) => delta_transform.o = Quat::with_axis_angle(Vec4::x(), -dx),
                        (Signum::Positive, DoF::ZX) => delta_transform.o = Quat::with_axis_angle(Vec4::y(), dx),
                        (Signum::Negative, DoF::ZX) => delta_transform.o = Quat::with_axis_angle(Vec4::y(), -dx),
                    }
                }
            }
        }

        for (_, _, trf) in res.iter_rw::<Camera, Transform>() {
            trf.affine.t = delta_transform.t + trf.affine.t;
            trf.affine.o = delta_transform.o * trf.affine.o;
            trf.affine.s *= delta_transform.s;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DoF {
    Z,
    Y,
    X,
    XY,
    ZX,
    YZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Signum {
    Positive,
    Negative,
}
