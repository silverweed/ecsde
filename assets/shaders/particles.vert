varying vec2 world_pos;

void main() {
    world_pos = vec2(gl_Vertex);
    gl_Position = gl_ModelViewProjectionMatrix * gl_Vertex;
    gl_FrontColor = gl_Color;
}
