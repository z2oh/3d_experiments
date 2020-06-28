#version 450

layout(location = 0) in ivec3 v_pos;
layout(location = 1) in ivec3 b_pos;
layout(location = 2) in vec2 v_tex_coord;
layout(location = 3) in uint data;

layout(location = 0) out vec2 o_v_tex_coord;
layout(location = 1) out ivec3 o_b_pos;
layout(location = 2) out uint face;

layout(set = 0, binding = 0) uniform Camera {
    mat4 c_transform;
};

layout(set = 0, binding = 3) uniform Transform {
    mat4 u_transform;
};

void main() {
    o_v_tex_coord = v_tex_coord;
    o_b_pos = b_pos;
    ivec3 clamped_pos = v_pos;

    face = data;

    gl_Position = c_transform * u_transform * vec4(clamped_pos, 1.0);
}
