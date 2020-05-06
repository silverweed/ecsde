struct Ambient_Light {
    vec3 color;
    float intensity;
};

struct Point_Light {
    vec2 position;
    vec3 color;
    float radius;
    float attenuation;
};

#define MAX_POINT_LIGHTS 64

uniform sampler2D texture;
uniform Ambient_Light ambient_light;
uniform Point_Light point_lights[MAX_POINT_LIGHTS];

varying vec2 world_pos;

void main() {
    vec4 pixel = texture2D(texture, gl_TexCoord[0].xy);

    vec4 color = vec4(1.0, 1.0, 1.0, 1.0);
    color *= vec4(ambient_light.color, 1.0) * ambient_light.intensity;

    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
        Point_Light light = point_lights[i];
        vec2 frag_to_light = light.position - world_pos;
        vec2 light_dir = normalize(frag_to_light);
        vec3 diffuse = light.color;

        float dist = length(frag_to_light);
        float atten = float(dist < light.radius) * 1.0 / (1.0 + dist * light.attenuation);

        color += vec4(diffuse, 0.0) * atten;
    }

    gl_FragColor = color * pixel;
}
