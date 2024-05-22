// Shader pre-processing hint: https://elyshaffir.github.io/Taiga-Blog/2022/01/08/using_include_statements_in_wgsl.html
// Function reference: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(4) transform_0: vec4<f32>,
    @location(5) transform_1: vec4<f32>,
    @location(6) transform_2: vec4<f32>,
    @location(7) transform_3: vec4<f32>,
    @location(8) with_camera: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct Camera {
    view_projection: mat4x4<f32>,
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera_transform: Camera;

@group(1) @binding(0)
var<uniform> light: Light;

@vertex
fn vertex_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );

    let local_position = vec4<f32>(vertex.position, 1.0);
    let world_position = local_position * model_transform;
    let with_camera = clamp(instance.with_camera, 0.0, 1.0);
    let clip_position = world_position * camera_transform.view_projection * with_camera + world_position * (1.0 - with_camera);

    let world_normal = normalize(vec4<f32>(vertex.normal, 0.0) * model_transform);

    return VertexOutput(
        clip_position,
        world_position.xyz,
        world_normal.xyz,
        vertex.tex_coords,
        light.color,
    );
}

@fragment
fn fragment_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let object_color = vec4<f32>(0.34, 0.34, 0.87, 1.0);

    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    return vec4<f32>(
        (ambient_color + diffuse_color) * object_color.xyz, 
        object_color.a,
    );
}

// vim: set filetype=wgsl :
