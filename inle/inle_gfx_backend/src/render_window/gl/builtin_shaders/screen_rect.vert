#version 330 core

layout (location = 0) in vec2 in_pos;

/* (left, top, width, height) */
uniform vec4 rect;
uniform vec2 win_half_size;

void main() {
	vec2 fin_pos = in_pos * vec2(rect.z, rect.w) + vec2(rect.x + 0.5 * rect.z, rect.y + 0.5 * rect.w);
	fin_pos.x = (fin_pos.x - win_half_size.x) / win_half_size.x;
	fin_pos.y = -(fin_pos.y - win_half_size.y) / win_half_size.y;
	gl_Position = vec4(fin_pos.x, fin_pos.y, 0.0, 1.0);
}
