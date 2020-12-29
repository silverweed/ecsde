#version 330 core

layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec4 in_color;
layout (location = 2) in vec2 in_tex_coord;

out vec4 vert_color;
out vec2 tex_coord;

uniform mat3 transform;

void main() {
	vec3 pos = transform * vec3(in_pos, 1.0);
	gl_Position = vec4(pos.xy, 0.0, 1.0);
	vert_color = in_color;
	tex_coord = in_tex_coord;
}
