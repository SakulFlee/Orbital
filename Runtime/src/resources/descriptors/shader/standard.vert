#version 460 core

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 vertexColor;

void main() {
    gl_Position = vec4(position, 1.);

    float m = gl_VertexIndex % 3;
    if (m == 0) {
        vertexColor = vec3(1., 0., 0.);
    } else if (m == 1) {
        vertexColor = vec3(0., 1., 0.);
    } else {
        vertexColor = vec3(0., 0., 1.);
    }
}
