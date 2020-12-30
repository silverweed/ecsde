#version 330 core

out vec4 vert_color;

uniform vec2 pos[2];
uniform vec4 color[2];

void main() {
	vec2 pos = pos[gl_VertexID];
	gl_Position = vec4(pos, 0.0, 1.0);
	vert_color = color[gl_VertexID];
}
