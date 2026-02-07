#version 330 core
in vec2 TexCoords;
out vec4 FragColor;

uniform sampler2D image;
uniform bool horizontal;
uniform float weight[5] = float[] (0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main() {
    vec2 tex_offset = 1.0 / textureSize(image, 0);
    
    // Start with center sample
    vec3 result = texture(image, TexCoords).rgb * weight[0];
    
    if(horizontal) {
        for(int i = 1; i < 5; ++i) {
            vec3 sample1 = texture(image, TexCoords + vec2(tex_offset.x * float(i), 0.0)).rgb;
            vec3 sample2 = texture(image, TexCoords - vec2(tex_offset.x * float(i), 0.0)).rgb;
            result += (sample1 + sample2) * weight[i];
        }
    } else {
        for(int i = 1; i < 5; ++i) {
            vec3 sample1 = texture(image, TexCoords + vec2(0.0, tex_offset.y * float(i))).rgb;
            vec3 sample2 = texture(image, TexCoords - vec2(0.0, tex_offset.y * float(i))).rgb;
            result += (sample1 + sample2) * weight[i];
        }
    }
    
    FragColor = vec4(result, 1.0);
}
