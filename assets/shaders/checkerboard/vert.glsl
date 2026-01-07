#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec4 aColor;
layout(location = 2) in vec2 aTexCoord;
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 mvp;
out vec4 ourColor;
out vec2 TexCoord;
out vec3 oPos;
out vec3 FragPos;
void main() {
    FragPos = vec3(model * vec4(aPos, 1.0));
    gl_Position = mvp * vec4(aPos, 1.0);
    ourColor = aColor;
    TexCoord = aTexCoord;
    oPos = aPos;
}