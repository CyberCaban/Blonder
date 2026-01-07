#version 330 core
out vec4 FragColor;
in vec3 FragPos;
in vec4 ourColor;
in vec3 oPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform int checkerboardPattern;
uniform int checkerboardFrame;
uniform float farPlane;
uniform vec3 cameraPos;
bool wasPixelRendered(ivec2 coord) {
    int pattern_x = (coord.x / 2) & 1;
    int pattern_y = (coord.y / 2) & 1;
    int pixel_pattern = (pattern_y << 1) | pattern_x;

    return pixel_pattern == checkerboardFrame;
}
bool isFarCulled() {
    float dist = length(FragPos - cameraPos);
    return dist > farPlane;
}
void main() {
    vec4 texColor = texture(tex, TexCoord) * ourColor;
    if(texColor.a < 0.1)
        discard;

    ivec3 coord = ivec3(gl_FragCoord.xyz);
    // if (((coord.x + coord.y) & 1) == checkerboardPattern) 
    //     discard;
    // if(coord.z > 0.1 && ((coord.x + coord.y) & 1) == checkerboardPattern)
    //     discard;

    if(isFarCulled() && wasPixelRendered(coord.xy))
        discard;

    // if(isFarCulled() && ((coord.x + coord.y) & 1) == checkerboardPattern)
    //     discard;
    FragColor = texColor;
}