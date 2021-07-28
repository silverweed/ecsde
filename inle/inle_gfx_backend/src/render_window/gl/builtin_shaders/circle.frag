#version 330 core

uniform vec4 color;
uniform vec2 center;
uniform float radius_squared;

in vec2 world_pos;

out vec4 frag_color;

void main() {
	vec2 diff = world_pos - center;
	if (dot(diff, diff) > radius_squared) {
		discard;
	}
	frag_color = color;
}
