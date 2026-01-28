#version 330 core
out vec4 FragColor;

in VS_OUT {
    vec3 Normal;
    vec3 FragPos;
} fs_in;

void main() {
    // Вариант 1: Цветные нормали
    vec3 normalColor = normalize(fs_in.Normal) * 0.5 + 0.5;
    FragColor = vec4(normalColor, 1.0);
    
    // Вариант 2: XYZ нормали в RGB
    // FragColor = vec4(abs(fs_in.Normal), 1.0);
    
    // Вариант 3: Только направление (без отрицательных значений)
    // FragColor = vec4(fs_in.Normal * 0.5 + 0.5, 1.0);
}