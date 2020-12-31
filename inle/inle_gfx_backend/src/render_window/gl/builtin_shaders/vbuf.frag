#version 330 core

in vec4 vert_color;
in vec2 tex_coord;

out vec4 out_color;

uniform sampler2D tex;

void main() {
	out_color = vert_color * texture(tex, tex_coord);
}
