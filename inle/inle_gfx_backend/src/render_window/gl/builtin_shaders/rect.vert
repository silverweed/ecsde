#version 330 core

uniform mat3 mvp;
uniform vec2 rect[4];

out vec2 world_pos;

void main() {
	vec2 in_pos = rect[gl_VertexID];
	vec3 fin_pos = mvp * vec3(in_pos, 1.0);
	world_pos = in_pos;
	gl_Position = vec4(fin_pos.x, fin_pos.y, 0.0, 1.0);
}
