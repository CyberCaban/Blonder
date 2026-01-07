#version 330 core
out vec4 FragColor;
in vec3 FragPos;
in vec4 ourColor;
in vec3 oPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform int checkerboardPattern;
uniform float farPlane;
uniform vec3 cameraPos;
void main() {
    vec4 texColor = texture(tex, TexCoord) * ourColor;
    if(texColor.a < 0.1)
        discard;

    ivec3 coord = ivec3(gl_FragCoord.xyz);
    // if (((coord.x + coord.y) & 1) == checkerboardPattern) 
    //     discard;
    // if(coord.z > 0.1 && ((coord.x + coord.y) & 1) == checkerboardPattern)
    //     discard;

    float dist = length(FragPos - cameraPos);
    if(dist > farPlane && ((coord.x + coord.y) & 1) == checkerboardPattern)
        discard;
    FragColor = texColor;
}