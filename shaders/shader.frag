#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in vec3 pos;
layout(location = 2) in vec3 normal;
layout(location = 0) out vec4 o_Target;
layout(set = 0, binding = 1) uniform texture2D t_Color;
layout(set = 0, binding = 2) uniform sampler s_Color;

// Function from Iñigo Quiles @ https://www.shadertoy.com/view/MsS3Wc.
// Smooth HSV to RGB conversion.
vec3 hsv2rgb(in vec3 c)
{
    vec3 rgb = clamp( abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),6.0)-3.0)-1.0, 0.0, 1.0 );
    rgb = rgb*rgb*(3.0-2.0*rgb); // cubic smoothing
    return c.z * mix( vec3(1.0), rgb, c.y);
}

const float _2_PI = 6.283185307179586;
const float PI = 3.1415926535897932384626433832795;
const float PI_2 = 1.57079632679489661923;
const float PI_4 = 0.785398163397448309616;

void main() {
    // The current height of the cube, mapped to [0.0, 1.0].
    float height_mag = (pos.z + 1.5) / 3.0;

    // Our texture color. For now, this is just pure white.
    vec4 tex = texture(sampler2D(t_Color, s_Color), v_TexCoord);

    // Convert our xy position to polar coordinates.
    float r_coord = length(pos.xy) / 32.0;
    float theta = atan(pos.y, pos.x);

    // Use our polar coordinates to sample HSV, and then convert that to RGB.
    vec3 hsv = vec3((theta / _2_PI) + 0.5, r_coord, height_mag);
    vec3 rgb = hsv2rgb(hsv);

    o_Target = vec4(rgb, 1.0);
}
