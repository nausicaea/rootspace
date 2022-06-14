struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] normals: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] normals: vec3<f32>;
};

struct FragmentOutput {
    [[location(0)]] fragment_color: vec4<f32>;
};

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;

[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(vertex)]]
fn vs_main(vertex_input: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(vertex_input.position, 1.0),
        vertex_input.tex_coords,
        vertex_input.normals,
    );
}

[[stage(fragment)]]
fn fs_main(vertex_output: VertexOutput) -> FragmentOutput {
    return FragmentOutput(
        textureSample(t_diffuse, s_diffuse, vertex_output.tex_coords),
    );
}
