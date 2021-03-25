#version 330 core

in vec2 tex_coord;

out vec4 out_color;

uniform vec4 color;
uniform sampler2D tex;

void main() {
	vec4 texel = texture(tex, tex_coord);
	if (texel.a < 0.5) {
		discard;
	}
	out_color = color * texel;
}
