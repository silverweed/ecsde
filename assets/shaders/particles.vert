#version 330 core

layout (location = 0) in vec4 in_color;
layout (location = 1) in vec2 in_pos;
layout (location = 2) in vec2 in_tex_coord;

out vec2 tex_coords;
out vec4 color;

uniform mat3 emitter_mvp;

mat3 matrix_from_pos_rot_scale(vec2 pos, float angle, vec2 scale) {
	float s = sin(angle);
	float c = cos(angle);
        float sxc = scale.x * c;
        float syc = scale.y * c;
        float sxs = scale.x * s;
        float sys = scale.y * s;
        float tx = pos.x;
        float ty = pos.y;

	return mat3(
        	sxc,  sys, tx,
		-sxs, syc, ty,
		0.0,  0.0, 1.0
	);
}

void main() {
	vec2 pos = vec2(0.0, 0.0);
	float angle = -0.0;
	vec2 scale = vec2(12.0, 12.0);
	mat3 particle_transform = matrix_from_pos_rot_scale(pos, angle, scale);

	vec3 clip_pos = emitter_mvp * particle_transform * vec3(in_pos, 1.0);
	gl_Position = vec4(clip_pos.xy, 0.0, 1.0);
	tex_coords = in_tex_coord;
	color = in_color;
}
