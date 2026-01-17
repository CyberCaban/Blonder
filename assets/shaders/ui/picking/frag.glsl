#version 330 core

flat in uint vObjectId;
layout(location = 0) out uint fragColor; 

void main()
{
    fragColor = vObjectId;
}