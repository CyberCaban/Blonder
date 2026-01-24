#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 mvp;
uniform float uTime;
// vertex snapping
uniform float snapFactor = 0.0;
out VS_OUT {
    vec2 TexCoords;
    vec3 FragPos;
    vec3 Normal;
} vs_out;
void main() {
    vs_out.FragPos = vec3(model * vec4(aPos, 1.0));
    // TODO: Make inverse matrices on CPU
    vs_out.Normal = mat3(transpose(inverse(model))) * aNormal;
    vs_out.TexCoords = aTexCoord;
    vec4 clipPos = projection * view * vec4(vs_out.FragPos, 1.0);

    // vertex snapping
    vec2 screenPos = clipPos.xy / clipPos.w;
    float snapFactor = max(snapFactor, 0.1);
    if(snapFactor > 0.1) {
        screenPos = floor(screenPos * snapFactor) / snapFactor;
        clipPos.xy = screenPos * clipPos.w;
    }

    gl_Position = clipPos;
}
