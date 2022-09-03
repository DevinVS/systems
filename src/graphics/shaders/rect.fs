#version 450
layout(location = 0) in vec2 v_tex_coords;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D atlas;

void main() {
    vec4 color = texture(atlas, v_tex_coords);
    f_color = pow(color, vec4(1. / 2.2));
}
