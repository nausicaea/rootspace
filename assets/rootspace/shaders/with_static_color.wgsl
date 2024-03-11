struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) scalar: f32,
}

@fragment
fn main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(0.34, 0.34, 0.87, 1.0) * in.scalar;
}
