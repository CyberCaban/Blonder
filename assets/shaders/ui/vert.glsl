#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec4 aColor;
layout(location = 2) in vec2 aTexCoord;

out vec2 TexCoord;
out vec4 FragColor;

uniform mat4 projection;

void main() {
  gl_Position = projection * vec4(aPos.xy, 0.0, 1.0);
  FragColor = aColor;
  TexCoord = aTexCoord;
}
