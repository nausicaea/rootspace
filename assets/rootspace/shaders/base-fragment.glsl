#version 330 core

uniform vec2 physical_dimensions;
uniform sampler2D diffuse_texture;
// uniform sampler2D normal_texture;

layout (location = 0) out vec4 color;

in vec2 frag_tex_coord;

void main() {
    color = texture(diffuse_texture, frag_tex_coord);
}
