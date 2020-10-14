#version 450

layout(set = 1, binding = 0) uniform texture2D u_Texture;
layout(set = 1, binding = 1) uniform sampler u_Sampler;

layout(location = 0) in vec2 v_UV;
layout(location = 1) in vec4 v_Color;

layout(location = 0) out vec4 o_Target;

vec4 srgb_to_linear(vec4 srgb) {
  vec3 color_srgb = srgb.rgb;
  vec3 selector = ceil(color_srgb - 0.04045); // 0 if under value, 1 if over
  vec3 under = color_srgb / 12.92;
  vec3 over = pow((color_srgb + 0.055) / 1.055, vec3(2.4));
  vec3 result = mix(under, over, selector);
  return vec4(result, srgb.a);
}

void main() {
  #ifdef LINEAR_OUTPUT
    o_Target = srgb_to_linear(v_Color) * texture(sampler2D(u_Texture, u_Sampler), v_UV);
  #else
    o_Target = v_Color * texture(sampler2D(u_Texture, u_Sampler), v_UV);
  #endif
}
