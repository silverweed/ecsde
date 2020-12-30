#version 330 core

layout (location = 0) in vec2 in_pos;

uniform mat3 transform;
uniform vec2 win_half_size;

void main() {
	vec3 pos = transform * vec3(in_pos, 1.0);
	pos.x = (pos.x - win_half_size.x) / win_half_size.x;
	pos.y = (win_half_size.y - pos.y) / win_half_size.y;

	gl_Position = vec4(pos.xy, 0.0, 1.0);
}
