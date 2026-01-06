#version 330 core
out vec4 FragColor;
in vec3 ourColor;
in vec3 oPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform float time;
void main() {
    vec3 pos = (oPos + 1.0) / 2.0;
    float t = (sin(time*10) + 1.0) / 2.0;
    FragColor = texture(tex, TexCoord) * vec4(t + oPos.x, cos(t + oPos.y), atan(t + oPos.z), 1.0);
}