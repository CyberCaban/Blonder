#version 330 core
out vec4 FragColor;

#define far 200.0
#define near 0.1

float LinearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0;
    return (2.0 * near * far) / (far + near - z * (far - near));
}

void main() {
    vec3 color = vec3(LinearizeDepth(gl_FragCoord.z) / far);
    FragColor = vec4(color * 1.1, 1.0);
}