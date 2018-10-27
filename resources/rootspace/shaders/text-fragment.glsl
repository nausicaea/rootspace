#version 330 core

uniform vec2 dimensions;
uniform Sampler2D diffuse_texture;
uniform Sampler2D normal_texture;

out vec4 color;

void main() {
    color = vec4(0.3, 0.12, 0.9, 1.0);
}
