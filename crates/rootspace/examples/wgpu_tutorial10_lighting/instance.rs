use griffon::base::descriptors::VertexAttributeDescriptor;
use griffon::wgpu;
use griffon::wgpu::{VertexAttribute, VertexStepMode};

#[derive(Debug)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let model = cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation);
        InstanceRaw {
            model: model.into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl VertexAttributeDescriptor for InstanceRaw {
    const STEP_MODE: VertexStepMode = VertexStepMode::Instance;
    const ATTRS: &'static [VertexAttribute] = &[
        VertexAttribute {
            offset: 0,
            // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
            // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
            shader_location: 5,
            format: wgpu::VertexFormat::Float32x4,
        },
        // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
        // for each vec4. We don't have to do this in code though.
        VertexAttribute {
            offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
            shader_location: 6,
            format: wgpu::VertexFormat::Float32x4,
        },
        VertexAttribute {
            offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
            shader_location: 7,
            format: wgpu::VertexFormat::Float32x4,
        },
        VertexAttribute {
            offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
            shader_location: 8,
            format: wgpu::VertexFormat::Float32x4,
        },
        VertexAttribute {
            offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
            shader_location: 9,
            format: wgpu::VertexFormat::Float32x3,
        },
        VertexAttribute {
            offset: size_of::<[f32; 19]>() as wgpu::BufferAddress,
            shader_location: 10,
            format: wgpu::VertexFormat::Float32x3,
        },
        VertexAttribute {
            offset: size_of::<[f32; 22]>() as wgpu::BufferAddress,
            shader_location: 11,
            format: wgpu::VertexFormat::Float32x3,
        },
    ];
}
