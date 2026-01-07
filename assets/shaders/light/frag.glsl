#version 330 core
out vec4 FragColor;
in vec4 ourColor;
in vec3 oPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform vec3 lightColor;
void main() {
    vec4 texColor = texture(tex, TexCoord) * vec4(ourColor.xyz * lightColor, 1.0);
    if (texColor.a < 0.1)
        discard;

    FragColor = texColor;
}