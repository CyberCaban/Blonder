#version 330 core

struct Material {
    sampler2D diffuse;
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
    vec4 FragPosLightSpace;
} fs_in;

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
uniform float specularIntensity;

vec3 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir);
vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
vec3 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir);

// dithering
uniform float ditherIntensity;
vec3 applyDither(vec3 color, vec2 uv, float intensity);
const mat4 ditherMatrix = mat4(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0) / 16.0;

uniform float scanlineIntensity;
vec3 applyScanlines(vec3 color, vec2 uv, float intensity);

// shadows
uniform sampler2D shadowMap;

void main() {
    vec4 texColor = texture(material.diffuse, fs_in.TexCoords);
    if(texColor.a < 0.1)
        discard;

    vec3 norm = normalize(fs_in.Normal);
    vec3 viewDir = normalize(cameraPos - fs_in.FragPos);
    vec3 emission = vec3(texture(material.emission, fs_in.TexCoords)) * 5.5;

    vec3 result = vec3(0.0);

    for(int i = 0; i < numDirLights; i++) result += CalcDirLight(dirLights[i], norm, viewDir);

    for(int i = 0; i < numPointLights; i++) result += CalcPointLight(pointLights[i], norm, fs_in.FragPos, viewDir);

    for(int i = 0; i < numSpotLights; i++) result += CalcSpotLight(spotLights[i], norm, fs_in.FragPos, viewDir);

    if(isSelected == 1) {
        result = mix(result, highlightColor, highlightIntensity);

        float edge = 1.0 - max(dot(norm, viewDir), 0.0);
        edge = pow(edge, 3.0) * 2.0;
        result = mix(result, highlightColor, edge * highlightIntensity);
    }
    result = result * texColor.rgb;
    result = clamp(result, 0.0, 1.0);
    result += emission;

    result = applyDither(result, texColor.xy, ditherIntensity);
    result = applyScanlines(result, texColor.xy, scanlineIntensity);

    FragColor = vec4(result, 1.0);
}

float ShadowCalculation(vec4 fragPosLightSpace, vec3 lightPos) {
    // perform perspective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;
    // get closest depth value from light's perspective (using [0,1] range fragPosLight as coords)
    // get depth of current fragment from light's perspective
    float currentDepth = projCoords.z;
    float closestDepth = texture(shadowMap, projCoords.xy).r;
    // calculate bias (based on depth map resolution and slope)
    vec3 normal = normalize(fs_in.Normal);
    vec3 lightDir = normalize(lightPos - fs_in.FragPos);
    float cosTheta = clamp(dot(normal, lightDir), 0.0, 1.0);
    // float bias = 0.0005 * tan(acos(cosTheta));
    // bias = clamp(bias, 0.001, 0.05);
    float bias = max(0.005 * (1.0 - dot(normal, lightDir)), 0.0005);

    // check whether current frag pos is in shadow
    // float shadow = currentDepth - bias > closestDepth  ? 1.0 : 0.0;
    if(currentDepth - bias <= closestDepth)
        return 0.0;
    // PCF
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    for(int x = -1; x <= 1; ++x) {
        for(int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow /= 9.0;

    // float transition = 1.0 - smoothstep(0.8, 1.0, projCoords.z);
    // shadow *= transition;

    // keep the shadow at 0.0 when outside the far_plane region of the light's frustum.
    // if(projCoords.z > 1.0)
    //     shadow = 0.0;

    return shadow;
}

vec3 applyScanlines(vec3 color, vec2 uv, float intensity) {
    if(intensity <= 0.0)
        return color;

    // Получаем вертикальную позицию
    float scanlinePos = mod(gl_FragCoord.y, 2.0);

    // Чередующиеся темные линии
    float scanline = mix(0.7, 1.0, scanlinePos);

    return color * scanline * (1.0 - intensity) + color * intensity;
}

vec3 applyDither(vec3 color, vec2 uv, float intensity) {
    if(intensity <= 0.0)
        return color;

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
    // float shadow = CalculateShadow(fs_in.FragPos, normal, lightDir);
    float shadow = ShadowCalculation(fs_in.FragPosLightSpace, lightDir);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading (reduced intensity to minimize bloom artifacts)
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // combine results
    vec3 ambient = clamp(light.ambient * vec3(texture(material.diffuse, fs_in.TexCoords)), 0.0, 1.0);
    vec3 diffuse = clamp(light.diffuse * diff * vec3(texture(material.diffuse, fs_in.TexCoords)), 0.0, 1.0);
    vec3 specular = clamp(light.specular * spec * vec3(texture(material.specular, fs_in.TexCoords)) * specularIntensity, 0.0, 1.0);
    return ambient + (1.0 - shadow) * (diffuse + specular);
}

// calculates the color when using a point light.
vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading (reduced intensity to minimize bloom artifacts)
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // combine results
vec3 ambient = clamp(light.ambient * vec3(texture(material.diffuse, fs_in.TexCoords)), 0.0, 1.0);
    vec3 diffuse = clamp(light.diffuse * diff * vec3(texture(material.diffuse, fs_in.TexCoords)), 0.0, 1.0);
    vec3 specular = clamp(light.specular * spec * vec3(texture(material.specular, fs_in.TexCoords)) * specularIntensity, 0.0, 1.0);
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
    // specular shading (reduced intensity to minimize bloom artifacts)
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // spotlight intensity
    float theta = dot(lightDir, normalize(-light.direction));
    float epsilon = light.cutOff - light.outerCutOff;
    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
    // combine results
vec3 ambient = clamp(light.ambient * vec3(texture(material.diffuse, fs_in.TexCoords)), 0.0, 1.0);
    vec3 diffuse = clamp(light.diffuse * diff * vec3(texture(material.diffuse, fs_in.TexCoords)), 0.0, 1.0);
    vec3 specular = clamp(light.specular * spec * vec3(texture(material.specular, fs_in.TexCoords)) * specularIntensity, 0.0, 1.0);
    ambient *= attenuation * intensity;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;
    return (ambient + diffuse + specular);
}