#version 330 core

// Note: these vertices come from the batcher, and are relative to the batch's coordinate system.
// The second model matrix we're multiplying here is the entire batch's world position.
layout (location = 0) in vec4 in_color;
layout (location = 1) in vec2 in_pos;
layout (location = 2) in vec2 in_tex_coord;

out vec4 color;
out vec2 tex_coord;

uniform mat3 mvp;

void main() {
    vec3 pos = mvp * vec3(in_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, 1.0);
    tex_coord = in_tex_coord;
    color = in_color;
}
