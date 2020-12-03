uniform sampler2D texture;

varying vec2 world_pos;
varying vec2 tex_coords;

void main() {
    vec4 pixel = texture2D(texture, tex_coords.xy);
    gl_FragColor = pixel;
}
