#version 330 core
in vec2 TexCoords;
out vec4 FragColor;

uniform sampler2D scene;
uniform sampler2D bloomBlur;
uniform float exposure;
uniform float bloomIntensity;

void main() {
    // Scene color (with tone mapping already applied if HDR)
    vec3 sceneColor = texture(scene, TexCoords).rgb;
    
    // Bloom color - scaled down to avoid overbrightness
    vec3 bloomColor = texture(bloomBlur, TexCoords).rgb * bloomIntensity;
    
    // Combine - bloom adds to the scene subtly
    vec3 result = sceneColor + bloomColor;
    
    // ACES tone mapping (more cinematic, prevents blowout)
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    result = clamp((result * exposure * (a * result + b)) / (result * exposure * (c * result + d) + e), 0.0, 1.0);
    
    // Gamma correction (2.2)
    result = pow(result, vec3(1.0 / 2.2));
    
    FragColor = vec4(result, 1.0);
}
