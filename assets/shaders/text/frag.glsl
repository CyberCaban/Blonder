#version 330 core
in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D uTexture;
uniform vec3 uTextColor;

void main() {
  // Получаем значение из красного канала (где хранится форма буквы)
  float alpha = texture(uTexture, TexCoord).r;

  // Если альфа очень маленькая, отбрасываем пиксель
  if(alpha < 0.1)
    discard;

  FragColor = vec4(uTextColor, alpha);
}
