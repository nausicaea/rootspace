// Shader pre-processing hint: https://elyshaffir.github.io/Taiga-Blog/2022/01/08/using_include_statements_in_wgsl.html
// Function reference: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(4) model_view_0: vec4<f32>,
    @location(5) model_view_1: vec4<f32>,
    @location(6) model_view_2: vec4<f32>,
    @location(7) model_view_3: vec4<f32>,
    @location(8) normal_0: vec4<f32>,
    @location(9) normal_1: vec4<f32>,
    @location(10) normal_2: vec4<f32>,
    @location(11) normal_3: vec4<f32>,
    @location(12) with_camera: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) view_position: vec3<f32>,
    @location(1) view_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec3<f32>,
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
    instance: InstanceInput,
) -> VertexOutput {
    let model_view = mat4x4<f32>(
        instance.model_view_0,
        instance.model_view_1,
        instance.model_view_2,
        instance.model_view_3,
    );

    let normal = mat4x4<f32>(
        instance.normal_0,
        instance.normal_1,
        instance.normal_2,
        instance.normal_3,
    );

    let with_camera = clamp(instance.with_camera, 0.0, 1.0);
    let local_position = vec4<f32>(vertex.position, 1.0);
    let view_position = local_position * model_view;
    let clip_position = view_position * camera.projection * with_camera + view_position * (1.0 - with_camera);

    let view_normal = normalize(vec4<f32>(vertex.normal, 0.0) * normal);

    return VertexOutput(
        clip_position,
        view_position.xyz,
        view_normal.xyz,
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

    let light_local_position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let light_view_position = light_local_position * light.model_view;
    let light_dir = normalize(light_view_position.xyz - in.view_position);
    let diffuse_strength = max(dot(in.view_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let view_dir = normalize(-in.view_position);
    let reflect_dir = reflect(-light_dir, in.view_normal);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular_color = light.color * specular_strength;

    return vec4<f32>(
        //(ambient_color + diffuse_color + specular_color) * object_color.xyz, 
        (specular_color) * object_color.xyz, 
        object_color.a,
    );
}

// vim: set filetype=wgsl :
