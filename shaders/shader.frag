#version 450

layout(location = 0) in vec2 v_tex_coord_raw;
layout(location = 1) flat in ivec3 b_pos;
layout(location = 2) flat in uint face;

layout(location = 0) out vec4 o_target;

layout(set = 0, binding = 1) uniform texture2D t_color;
layout(set = 0, binding = 2) uniform sampler s_color;

const float _2_PI = 6.283185307179586;
const float PI = 3.1415926535897932384626433832795;
const float PI_2 = 1.57079632679489661923;
const float PI_4 = 0.785398163397448309616;

void main() {
    vec2 v_tex_coord = v_tex_coord_raw / 2.0;

    if (b_pos.z >= 15) {
        vec4 tex = texture(sampler2D(t_color, s_color), v_tex_coord);
        o_target = tex;
    } else if (b_pos.z >= 0) {
        // If we are on the top face, use the grass texture.
        if (face == 32) {
            vec4 tex = texture(sampler2D(t_color, s_color), v_tex_coord + vec2(0.5));
            o_target = tex;
        } else {
            vec4 tex = texture(sampler2D(t_color, s_color), v_tex_coord + vec2(0.0, 0.5));
            o_target = tex;
        }
    } else {
        vec4 tex = texture(sampler2D(t_color, s_color), v_tex_coord + vec2(0.5, 0.0));
        o_target = tex;
    }
}
