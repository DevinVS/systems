#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coords;

layout(location = 0) out vec2 v_tex_coords;

layout(set = 0, binding = 0) uniform Data {
    mat4 worldview;
} uniforms;

void main() {
    v_tex_coords = tex_coords;
    vec4 pos = uniforms.worldview * vec4(position, 1.0, 1.0);
    gl_Position = pos;
}