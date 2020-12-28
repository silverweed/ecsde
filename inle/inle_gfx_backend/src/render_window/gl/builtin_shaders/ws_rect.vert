#version 330 core

//layout (location = 0) in vec2 in_pos;

uniform mat3 mvp;
uniform vec2 rect[4];

void main() {
	vec2 in_pos = rect[gl_VertexID];
	vec3 fin_pos = mvp * vec3(in_pos, 1.0);
	gl_Position = vec4(fin_pos.x, fin_pos.y, 0.0, 1.0);
}
