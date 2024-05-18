#version 460 core

layout(location = 0) in vec3 in_vertex_color;
layout(location = 1) in vec2 in_texture_coordinate;

layout(location = 0) out vec4 out_fragment_color;

layout(set = 0, binding = 0) uniform texture2D u_albedo_texture;
layout(set = 0, binding = 1) uniform sampler u_albedo_sampler;

void main() {
    out_fragment_color = texture(
        sampler2D(u_albedo_texture, u_albedo_sampler), 
        in_texture_coordinate
    );
}
