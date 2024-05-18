#version 460 core

layout(location = 0) in vec3 in_position_coordinates;
layout(location = 1) in vec2 in_texture_coordinates;

layout(location = 0) out vec3 out_vertex_color;
layout(location = 1) out vec2 out_texture_coordinates;

void main() {
    gl_Position = vec4(in_position_coordinates, 1.);

    float m = gl_VertexIndex % 3;
    if (m == 0) {
        out_vertex_color = vec3(1., 0., 0.);
    } else if (m == 1) {
        out_vertex_color = vec3(0., 1., 0.);
    } else {
        out_vertex_color = vec3(0., 0., 1.);
    }

    out_texture_coordinates = in_texture_coordinates;
}
