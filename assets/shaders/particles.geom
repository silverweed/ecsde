#version 150

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

uniform float quad_half_width;
uniform float window_ratio;
uniform float camera_scale;

void main() {
	vec4 position = gl_in[0].gl_Position;
    float hw = quad_half_width * camera_scale;
    float hh = hw * camera_ratio;
	
	gl_Position = position + vec4(-hw, -hh, 0.0, 0.0);
	//gl_TexCoord = vec2(-1.0, -1.0);
	EmitVertex();

	gl_Position = position + vec4(hw, -hh, 0.0, 0.0);
	//gl_TexCoord = vec2(1.0, -1.0);
	EmitVertex();

	gl_Position = position + vec4(-hw, hh, 0.0, 0.0);
	//gl_TexCoord = vec2(1.0, 1.0);
	EmitVertex();

	gl_Position = position + vec4(hw, hh, 0.0, 0.0);
	//gl_TexCoord = vec2(1.0, 1.0);
	EmitVertex();

	EndPrimitive();
}
