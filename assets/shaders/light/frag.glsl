#version 330 core

struct Material {
    sampler2D specular;
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
in vec3 LightPos;
uniform sampler2D tex;
uniform vec3 lightColor;
uniform vec3 lightPos;
uniform vec3 cameraPos;
uniform Light light;
uniform Material material;
void main() {
    vec4 texColor = texture(tex, TexCoord);
    if(texColor.a < 0.1)
        discard;
    vec4 specularTex = texture(material.specular, TexCoord);

    // ambient
    vec3 ambient = light.ambient * texColor.rgb;

    // diffuse
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(LightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * (diff * texColor.rgb);

    // specular
    vec3 viewDir = normalize(-FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * (spec * specularTex.rgb);

    FragColor = texColor * (vec4(ambient + diffuse + specular, 1.0));
}
