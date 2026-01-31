#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D tex;
void main()
{
    vec4 texture = texture(tex, TexCoords);
    FragColor = texture;
}

