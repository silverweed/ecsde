#version 330 core

layout (location = 0) in vec4 in_color;
layout (location = 1) in vec2 in_pos;
layout (location = 2) in vec2 in_tex_coord;

out vec4 vert_color;
out vec2 tex_coord;

uniform vec2 win_half_size;
uniform mat3 transform;

void main() {
	vec3 pos = transform * vec3(in_pos, 1.0);
	pos.x = (pos.x - win_half_size.x) / win_half_size.x;
	pos.y = (win_half_size.y - pos.y) / win_half_size.y;
	gl_Position = vec4(pos.xy, 0.0, 1.0);
	vert_color = in_color;
	tex_coord = in_tex_coord;
}
