struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(1)
var<uniform> transform: mat4x4<f32>;

@vertex
fn main(
    in: VertexInput
) -> VertexOutput {
    return VertexOutput(
        transform * vec4<f32>(in.position, 1.0),
        in.tex_coords,
    );
}
