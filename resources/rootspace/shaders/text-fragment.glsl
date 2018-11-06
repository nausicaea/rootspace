#version 330 core

layout (location = 0) out vec4 color;

uniform vec2 dimensions;
uniform sampler2D diffuse_texture;
// uniform sampler2D normal_texture;

in vec2 frag_tex_coord;

const vec3 text_color = vec3(0.0, 0.0, 0.0);

void main() {
    vec4 text_data = texture(diffuse_texture, frag_tex_coord);
    float alpha = text_data.r;
    float color_factor = text_data.a;
    // color = vec4(text_color * color_factor * alpha, alpha);
    color = vec4(0.3, 0.12, 0.9, 1.0);
}
