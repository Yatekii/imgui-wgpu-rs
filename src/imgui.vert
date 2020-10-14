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

void main() {
  v_UV = a_UV;
  v_Color = a_Color;
  gl_Position = u_Matrix * vec4(a_Pos.xy, 0.0, 1.0);
}
