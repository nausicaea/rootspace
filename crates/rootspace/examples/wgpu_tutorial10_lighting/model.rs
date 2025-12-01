use crate::texture::Texture;
use crate::util;
use crate::vertex::Vertex;
use griffon::base::descriptors::VertexAttributeDescriptor;
use griffon::base::ids::{BindGroupId, BindGroupLayoutId, BufferId};
use griffon::wgpu::util::DeviceExt;
use griffon::wgpu::{BufferUsages, VertexAttribute, VertexStepMode};
use griffon::{Graphics, wgpu};
use std::io::{BufReader, Cursor};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl VertexAttributeDescriptor for ModelVertex {
    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
    const ATTRS: &'static [VertexAttribute] = &[
        VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        VertexAttribute {
            offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
        VertexAttribute {
            offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x3,
        },
    ];
}

#[derive(Debug)]
pub struct Material {
    #[allow(unused)]
    pub name: String,
    #[allow(unused)]
    pub diffuse_texture: Texture,
    pub bind_group: BindGroupId,
}

#[derive(Debug)]
pub struct Mesh {
    #[allow(unused)]
    pub name: String,
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_elements: u32,
    pub material: usize,
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub async fn load_model(file_name: &str, gfx: &mut Graphics, layout: BindGroupLayoutId) -> anyhow::Result<Model> {
    let obj_text = util::load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = util::load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = Texture::from_file(gfx, &m.diffuse_texture).await?;
        let bind_group = gfx
            .create_bind_group(layout)
            .add_texture_view(0, diffuse_texture.view)
            .add_sampler(1, diffuse_texture.sampler)
            .submit();

        materials.push(Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            let vertex_buffer = gfx.create_buffer_init(
                Some(&format!("{:?} Vertex Buffer", file_name)),
                BufferUsages::VERTEX,
                &vertices,
            );
            let index_buffer = gfx.create_buffer_init(
                Some(&format!("{:?} Index Buffer", file_name)),
                BufferUsages::INDEX,
                &m.mesh.indices,
            );

            Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(Model { meshes, materials })
}
