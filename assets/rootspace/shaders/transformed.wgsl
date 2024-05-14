struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(4) transform_0: vec4<f32>,
    @location(5) transform_1: vec4<f32>,
    @location(6) transform_2: vec4<f32>,
    @location(7) transform_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) scalar: f32,
}

@group(0) @binding(0)
var<uniform> camera_transform: mat4x4<f32>;

@vertex
fn main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );

    let front = vec4(1.0, 1.0, 1.0, 0.0);
    let normals = vec4<f32>(vertex.normals, 0.0);
    let s = dot(front, normals) / (length(front) * length(normals));

    return VertexOutput(
        vec4<f32>(vertex.position, 1.0) * model_transform * camera_transform,
        vertex.tex_coords,
        s,
    );
}
