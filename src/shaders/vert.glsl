#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec4 a_color;

layout(location = 0) out vec4 v_color;

void main() {
     f_color = a_color;
     gl_Position = vec4(a_pos, 1.0);
}