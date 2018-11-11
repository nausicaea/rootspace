#version 330 core

layout (location = 0) out vec4 color;

uniform vec2 dimensions;
uniform sampler2D diffuse_texture;
// uniform sampler2D normal_texture;

void main() {
    color = vec4(0.3, 0.12, 0.9, 0.6);
}
