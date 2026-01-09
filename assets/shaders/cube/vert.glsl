#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 mvp;
uniform float uTime;
uniform vec3 lightPos;
out vec2 TexCoord;
out vec3 FragPos;
out vec3 Normal;
out vec3 LightPos;
void main() {
    gl_Position = projection * view * model * vec4(aPos.x, aPos.y, aPos.z, 1.0);
    FragPos = vec3(view * model * vec4(aPos, 1.0));
    // TODO: Make inverse matrices on CPU
    Normal = mat3(transpose(inverse(view * model))) * aNormal;
    LightPos = vec3(view * vec4(lightPos, 1.0));
    TexCoord = aTexCoord;
}
