#version 330 core
out vec4 FragColor;
in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform vec3 lightColor;
uniform vec3 lightPos;
void main() {
    // ambient
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;

    // diffuse
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    vec4 texColor = texture(tex, TexCoord);
    if (texColor.a < 0.1)
        discard;

    FragColor = texColor * (vec4(ambient + diffuse, 1.0));
}
