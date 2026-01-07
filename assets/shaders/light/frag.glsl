#version 330 core
out vec4 FragColor;
in vec4 ourColor;
in vec3 oPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform vec3 lightColor;
void main() {
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
    vec4 textureColor = texture(tex, TexCoord);
    vec4 objectColor = vec4(ourColor.xyz * lightColor, 1.0);
    vec4 ambientColor = vec4(objectColor.xyz * ambient, objectColor.w);
    vec4 result = textureColor * ambientColor;
    if(result.a < 0.1)
        discard;

    FragColor = result;
}