#version 330 core

layout (location = 0) in vec2 in_pos;

uniform mat3 mvp;

void main() {
	vec3 pos = mvp * vec3(in_pos, 1.0);
	gl_Position = vec4(pos.xy, 0.0, 1.0);
}
