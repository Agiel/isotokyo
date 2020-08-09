#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=2) in mat4 a_model;
layout(location=6) in vec4 a_color;
layout(location=7) in vec4 a_source;

layout(location=0) out vec2 v_tex_coords;
layout(location=1) out vec4 v_color;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    v_tex_coords = a_source.xy + a_tex_coords.xy * a_source.zw;
    v_color = a_color;
    gl_Position = u_view_proj * a_model * vec4(a_position, 1.0);
}
