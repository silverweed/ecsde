#version 330 core

in vec2 tex_coord;
in vec4 vert_color;

out vec4 frag_color;

uniform sampler2D tex;
uniform vec4 color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main()
{
    vec2 pos = tex_coord;
    vec3 sample = texture(tex, tex_coord).rgb;
    float sig_dist = median(sample.r, sample.g, sample.b);
    float w = fwidth(sig_dist);
    float opacity = smoothstep(0.5 - w, 0.5 + w, sig_dist);
    frag_color = color * vec4(vert_color.rgb, opacity);
}
