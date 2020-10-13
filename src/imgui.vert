#version 450

layout(set = 0, binding = 0) uniform View {
  mat4 u_Matrix;
};

layout(location = 0) in vec2 a_Pos;
layout(location = 1) in vec2 a_UV;
layout(location = 2) in vec4 a_Color;

layout(location = 0) out vec2 v_UV;
layout(location = 1) out vec4 v_Color;

// Built-in:
// vec4 gl_Position

vec4 srgb_to_linear(vec4 srgb) {
  vec3 color_srgb = srgb.rgb;
  vec3 selector = ceil(color_srgb - 0.04045); // 0 if under value, 1 if over
  vec3 under = color_srgb / 12.92;
  vec3 over = pow((color_srgb + 0.055) / 1.055, vec3(2.4));
  vec3 result = mix(under, over, selector);
  return vec4(result, srgb.a);
}

void main() {
  v_UV = a_UV;
#ifdef LINEAR_OUTPUT
  v_Color = srgb_to_linear(a_Color);
#else
  v_Color = a_Color;
#endif
  gl_Position = u_Matrix * vec4(a_Pos.xy, 0.0, 1.0);
}
