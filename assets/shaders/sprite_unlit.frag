uniform sampler2D texture;

varying vec2 world_pos;

void main() {
    vec4 pixel = texture2D(texture, gl_TexCoord[0].xy);
    gl_FragColor = pixel;
}
