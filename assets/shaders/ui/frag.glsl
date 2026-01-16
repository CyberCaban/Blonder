#version 330 core
in vec2 TexCoord;
in vec4 FragColor;
out vec4 OutColor;

uniform sampler2D uTexture;

void main() {
  vec4 tex;
  tex = texture(uTexture, TexCoord);
  float alpha = tex.a;
  if(alpha < 0.1)
    discard;
  OutColor = (FragColor * tex);
}
