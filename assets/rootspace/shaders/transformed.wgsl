struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) scalar: f32,
}

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn main(
    in: VertexInput
) -> VertexOutput {
    let front = vec4(1.0, 1.0, 1.0, 0.0);
    let normals = vec4<f32>(in.normals, 1.0);
    let s = dot(front, normals) / (length(front) * length(normals));

    return VertexOutput(
        vec4<f32>(in.position, 1.0) * transform,
        in.tex_coords,
        s,
    );
}
