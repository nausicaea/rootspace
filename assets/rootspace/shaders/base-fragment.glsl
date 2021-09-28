#version 330 core

uniform vec2 physical_dimensions;
uniform sampler2D diffuse_texture;
// uniform sampler2D normal_texture;

layout (location = 0) out vec4 color;

in vec2 frag_tex_coord;

void main() {
    color = texture(diffuse_texture, frag_tex_coord);
    // color = vec4(frag_tex_coord.x, frag_tex_coord.y, 1.0, 1.0);
}
