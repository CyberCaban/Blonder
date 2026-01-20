#version 330 core

struct Material {
    sampler2D specular;
    sampler2D emission;
    float shininess;
};
struct Light {
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};
out vec4 FragColor;
in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoord;
uniform sampler2D tex;
uniform vec3 lightColor;
uniform vec3 lightPos;
uniform vec3 cameraPos;
uniform float uTime;

uniform Light light;
uniform Material material;

uniform int isSelected;
uniform vec3 highlightColor;
uniform float highlightIntensity;

void main() {
    vec4 texColor = texture(tex, TexCoord);
    if(texColor.a < 0.1)
        discard;
    vec4 specularTex = texture(material.specular, TexCoord);
    vec4 emissionTex = texture(material.emission, TexCoord);

    // ambient
    vec3 ambient = light.ambient * texColor.rgb;

    // diffuse
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * (diff * texColor.rgb);

    // specular
    vec3 viewDir = normalize(cameraPos - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * (spec * specularTex.rgb);

    vec3 result = ambient + diffuse + specular + emissionTex.rgb;

    if(isSelected == 1) {

        result = mix(result, highlightColor, highlightIntensity);

        // result = result * (1.0 + highlightIntensity);

        float edge = 1.0 - max(dot(norm, viewDir), 0.0);
        edge = pow(edge, 3.0) * 2.0;
        result = mix(result, highlightColor, edge * highlightIntensity);
    }

    FragColor = vec4(result * texColor.rgb, texColor.a);
}