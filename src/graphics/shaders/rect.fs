#version 450
layout(location = 0) in vec2 v_tex_coords;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D atlas;

void main() {
    f_color = texture(atlas, v_tex_coords);
}
