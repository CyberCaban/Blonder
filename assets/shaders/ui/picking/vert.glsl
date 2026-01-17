#version 330 core

layout(location = 0) in vec3 aPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform uint objectId;

flat out uint vObjectId;

void main()
{
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    vObjectId = objectId;
}