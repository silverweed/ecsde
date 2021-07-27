#version 330 core

layout (location = 0) in vec4 in_color;
layout (location = 1) in vec2 in_pos;
layout (location = 2) in vec2 in_tex_coord;

out vec2 tex_coords;
out vec4 color;

uniform mat3 mvp;

void main() {
	vec3 clip_pos = mvp * vec3(in_pos, 1.0);
	gl_Position = vec4(clip_pos.xy, 0.0, 1.0);
	tex_coords = in_tex_coord;
	color = in_color;
}
