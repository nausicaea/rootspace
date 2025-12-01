struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct Camera {
    projection: mat4x4<f32>,
}

struct Light {
    model_view: mat4x4<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> light: Light;

@vertex
fn vertex_main(
    vertex: VertexInput,
) -> VertexOutput {
    let scale = 0.25;

    let local_position = vec4<f32>(vertex.position, 1.0);
    let view_position = light.model_view * local_position;
    let clip_position = camera.projection * view_position;

    return VertexOutput(
        clip_position,
        light.color,
    );
}

@fragment
fn fragment_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

// vim: set filetype=wgsl :
