#version 150

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

uniform float quad_half_width;
uniform float window_ratio;
uniform float camera_scale;
uniform vec2 tex_size_normalized;

out vec2 tex_coords;

void main() {
	vec4 position = gl_in[0].gl_Position;

	float hw = 0.5 * tex_size_normalized.x * camera_scale;
	float hh = 0.5 * tex_size_normalized.y * camera_scale;
	
	gl_Position = position + vec4(-hw, -hh, 0.0, 0.0);
	tex_coords = vec2(0.0, 0.0);
	EmitVertex();

	gl_Position = position + vec4(hw, -hh, 0.0, 0.0);
	tex_coords = vec2(1.0, 0.0);
	EmitVertex();

	gl_Position = position + vec4(-hw, hh, 0.0, 0.0);
	tex_coords = vec2(0.0, 1.0);
	EmitVertex();

	gl_Position = position + vec4(hw, hh, 0.0, 0.0);
	tex_coords = vec2(1.0, 1.0);
	EmitVertex();

	EndPrimitive();
}
