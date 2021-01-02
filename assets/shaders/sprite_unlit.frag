#version 330 core

uniform sampler2D texture;

in vec4 color;
in vec2 tex_coord;

out vec4 frag_color;

void main() {
    vec4 pixel = texture2D(texture, tex_coord);
    frag_color = pixel * color;
}
