// Shader pre-processing hint: https://elyshaffir.github.io/Taiga-Blog/2022/01/08/using_include_statements_in_wgsl.html
// Function reference: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html

const TAU = 6.283185307179586476925286766559005768394338798;

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

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(2) @binding(1)
var s_diffuse: sampler;

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

    let with_camera = step(0.5, instance.with_camera);
    let local_position = vec4<f32>(vertex.position, 1.0);
    let view_position = model_view * local_position;
    let clip_position = with_camera * camera.projection * view_position + (1.0 - with_camera) * view_position;

    let view_normal = normalize(normal * vec4<f32>(vertex.normal, 0.0));

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
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Light source properties
    let ambient_light_intensity = 0.05;
    let point_light_intensity = 1.0;

    // Material properties
    let ambient_reflectivity = 1.0;
    let diffuse_reflectivity = 1.0;
    let specular_reflectivity = 1.0;
    let smoothness = 32.0;

    let incoming_light_direction = normalize(light.model_view * vec4(0.0, 0.0, 0.0, 1.0) - vec4(in.view_position, 1.0)).xyz;
    let outgoing_light_direction = reflect(-incoming_light_direction, in.view_normal);
    let viewing_direction = normalize(vec3(0.0, 0.0, -1.0) - in.view_position);
    let surface_normal = in.view_normal;

    let normalization_factor = (smoothness + 2.0) / TAU;
    let diffuse_component = dot(incoming_light_direction, surface_normal);
    let specular_component = normalization_factor * pow(dot(outgoing_light_direction, viewing_direction), smoothness);

    let phong_light_intensity = ambient_light_intensity * ambient_reflectivity + point_light_intensity * (diffuse_reflectivity * diffuse_component + specular_reflectivity * specular_component);
    let color = phong_light_intensity * light.color * object_color.rgb;

    return vec4<f32>(
        color,
        object_color.a,
    );
}

// vim: set filetype=wgsl :
