#version 450

layout (location = 0) in vec4 in_color;
layout (location = 1) in vec3 in_position;
layout (location = 0) out vec4 out_color;

layout (binding = 0) uniform VS_UBO_VIEW_INFO
{
    mat4 view_mtx;
    mat4 projection_mtx;
} camera;

layout (push_constant) uniform VS_CB_MODEL_INFO {
    mat4 world_mtx;
} model;

void main() {
    out_color = in_color;
    gl_Position = camera.projection_mtx * camera.view_mtx * model.world_mtx * vec4(in_position, 1.0);
}
