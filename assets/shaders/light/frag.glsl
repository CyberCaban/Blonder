#version 330 core

struct Material {
    sampler2D specular;
    sampler2D emission;
    float shininess;
};
struct DirLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;

    float constant;
    float linear;
    float quadratic;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct SpotLight {
    vec3 position;
    vec3 direction;
    float cutOff;
    float outerCutOff;

    float constant;
    float linear;
    float quadratic;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

#define MAX_POINT_LIGHTS 8
#define MAX_DIR_LIGHTS 4
#define MAX_SPOT_LIGHTS 4

out vec4 FragColor;
in VS_OUT {
    vec2 TexCoords;
    vec3 FragPos;
    vec3 Normal;
} fs_in;
uniform sampler2D tex;
uniform vec3 lightColor;
uniform vec3 cameraPos;
uniform float uTime;

uniform DirLight dirLights[MAX_DIR_LIGHTS];
uniform PointLight pointLights[MAX_POINT_LIGHTS];
uniform SpotLight spotLights[MAX_SPOT_LIGHTS];
uniform int numDirLights;
uniform int numPointLights;
uniform int numSpotLights;
uniform Material material;

uniform int isSelected;
uniform vec3 highlightColor;
uniform float highlightIntensity;

vec3 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir);
vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
vec3 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir);

// dithering
uniform float ditherIntensity;
vec3 applyDither(vec3 color, vec2 uv, float intensity);
const mat4 ditherMatrix = mat4(
    0.0,  8.0,  2.0,  10.0,
    12.0, 4.0,  14.0, 6.0,
    3.0,  11.0, 1.0,  9.0,
    15.0, 7.0,  13.0, 5.0
) / 16.0;

void main() {
    vec4 texColor = texture(tex, fs_in.TexCoords);
    if(texColor.a < 0.1)
        discard;

    vec3 norm = normalize(fs_in.Normal);
    vec3 viewDir = normalize(cameraPos - fs_in.FragPos);
    vec3 emission = vec3(texture(material.emission, fs_in.TexCoords));

    vec3 result = emission;

    for(int i = 0; i < numDirLights; i++) result += CalcDirLight(dirLights[i], norm, viewDir);

    for(int i = 0; i < numPointLights; i++) result += CalcPointLight(pointLights[i], norm, fs_in.FragPos, viewDir);

    for(int i = 0; i < numSpotLights; i++) result += CalcSpotLight(spotLights[i], norm, fs_in.FragPos, viewDir);

    if(isSelected == 1) {
        result = mix(result, highlightColor, highlightIntensity);

        float edge = 1.0 - max(dot(norm, viewDir), 0.0);
        edge = pow(edge, 3.0) * 2.0;
        result = mix(result, highlightColor, edge * highlightIntensity);
    }

    result = applyDither(result, texColor.xy, ditherIntensity);

    FragColor = vec4(result, 1.0) * texColor;
}

vec3 applyDither(vec3 color, vec2 uv, float intensity)
{
    if (intensity <= 0.0) return color;
    
    // Получаем координаты в матрице 4x4
    ivec2 pos = ivec2(mod(gl_FragCoord.xy, 4));
    float threshold = ditherMatrix[pos.x][pos.y];
    
    // Добавляем дизеринг
    vec3 dithered = color + (threshold - 0.5) * intensity / 255.0;
    return clamp(dithered, 0.0, 1.0);
}

// calculates the color when using a directional light.
vec3 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // combine results
    vec3 ambient = light.ambient * vec3(texture(tex, fs_in.TexCoords));
    vec3 diffuse = light.diffuse * diff * vec3(texture(tex, fs_in.TexCoords));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, fs_in.TexCoords));
    return (ambient + diffuse + specular);
}

// calculates the color when using a point light.
vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // combine results
    vec3 ambient = light.ambient * vec3(texture(tex, fs_in.TexCoords));
    vec3 diffuse = light.diffuse * diff * vec3(texture(tex, fs_in.TexCoords));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, fs_in.TexCoords));
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    return (ambient + diffuse + specular);
}

// calculates the color when using a spot light.
vec3 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // spotlight intensity
    float theta = dot(lightDir, normalize(-light.direction));
    float epsilon = light.cutOff - light.outerCutOff;
    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
    // combine results
    vec3 ambient = light.ambient * vec3(texture(tex, fs_in.TexCoords));
    vec3 diffuse = light.diffuse * diff * vec3(texture(tex, fs_in.TexCoords));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, fs_in.TexCoords));
    ambient *= attenuation * intensity;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;
    return (ambient + diffuse + specular);
}