#version 330 core
out vec4 FragColor;
in vec4 ourColor;
in vec3 oPos;
in vec2 TexCoord;
uniform sampler2D tex;
void main() {
    vec4 texColor = texture(tex, TexCoord) * ourColor;
    if (texColor.a < 0.1)
        discard;

    FragColor = texColor;
}