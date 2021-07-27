#version 330 core

in vec2 tex_coords;
in vec4 color;

out vec4 frag_color;

uniform sampler2D tex;

void main() {
	vec4 pixel = texture(tex, tex_coords);
	frag_color = color * pixel;
}
