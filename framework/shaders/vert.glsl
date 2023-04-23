#version 450

layout (location = 0) in vec3 in_position;
layout (location = 0) out vec4 out_color;

layout (binding = 0) uniform VS_UBO_VIEW_INFO
{
    mat4 view_mtx;
    mat4 projection_mtx;
} camera;

layout(push_constant) uniform ObjectData {
    vec4 color;
    mat4 transform;
} object;

void main() {
    out_color = object.color;
    gl_Position = camera.projection_mtx * camera.view_mtx * object.transform * vec4(in_position, 1.0);
}
