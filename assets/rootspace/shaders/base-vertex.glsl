#version 330 core

uniform mat4 transform;

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 tex_coord;
layout (location = 2) in vec3 normals;

void main() {
    gl_Position = transform * vec4(position, 1.0);
}
