struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera_transform: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> light: Light;

@vertex
fn vertex_main(
    vertex: VertexInput,
) -> VertexOutput {
    let scale = 0.25;
    let world_position = vec4<f32>(vertex.position.xyz * scale + light.position.xyz, 1.0);
    let clip_position = world_position * camera_transform;

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
