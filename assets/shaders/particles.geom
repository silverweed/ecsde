layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

void main() {
	vec4 position = gl_in[0].gl_Position;
	
	gl_Position = position + vec4(-1.0, -1.0, 0.0, 0.0);
	gl_TexCoord = vec2(-1.0, -1.0);
	EmitVertex();

	gl_Position = position + vec4(1.0, -1.0, 0.0, 0.0);
	gl_TexCoord = vec2(1.0, -1.0);
	EmitVertex();

	gl_Position = position + vec4(1.0, 1.0, 0.0, 0.0);
	gl_TexCoord = vec2(1.0, 1.0);
	EmitVertex();

	gl_Position = position + vec4(1.0, -1.0, 0.0, 0.0);
	gl_TexCoord = vec2(1.0, 1.0);
	EmitVertex();

	EndPrimitive();
}
