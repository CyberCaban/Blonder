#version 330 core
in vec2 TexCoords;
out vec4 FragColor;

uniform sampler2D image;
uniform float threshold;

void main() {
    vec3 color = texture(image, TexCoords).rgb;
    
    // Only very bright areas bloom (filters out specular)
    float brightness = dot(color, vec3(0.2126, 0.7152, 0.0722));
    
    // Only bloom if pixel is much brighter than threshold
    vec3 brightColor = color * step(threshold, brightness);
    
    // Scale down to prevent overbloom
    brightColor *= 0.5;
    
    FragColor = vec4(brightColor, 1.0);
}
