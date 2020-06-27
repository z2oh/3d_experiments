#version 450

layout(location = 0) in vec4 a_Pos;
layout(location = 1) in vec3 a_Normal;
layout(location = 2) in vec2 a_TexCoord;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 v_Pos;
layout(location = 2) out vec3 v_Normal;

layout(set = 0, binding = 0) uniform Camera {
    mat4 c_Transform;
};

layout(set = 0, binding = 3) uniform Transform {
    mat4 u_Transform;
};

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = c_Transform * u_Transform * a_Pos;
    v_Pos = vec3(a_Pos.xyz);
    v_Normal = a_Normal;
}
