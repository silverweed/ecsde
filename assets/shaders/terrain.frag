struct Ambient_Light {
    vec3 color;
    float intensity;
};

struct Point_Light {
    vec2 position;
    vec3 color;
    float radius;
    float attenuation;
    float intensity;
};

#define MAX_POINT_LIGHTS 8

uniform sampler2D texture;
uniform vec2 texture_size;
uniform Ambient_Light ambient_light;
uniform Point_Light point_lights[MAX_POINT_LIGHTS];

varying vec2 world_pos;

void main() {
    vec4 pixel = texture2D(texture, gl_TexCoord[0].xy);

    vec3 color = vec3(1.0);
    color *= ambient_light.color * ambient_light.intensity;

    float avg_transparency = 0.0;
    const int EDGE_RADIUS = 7;
    for (int x = -EDGE_RADIUS; x < EDGE_RADIUS; ++x) {
        float x_norm = float(x) / texture_size.x;
        for (int y = -EDGE_RADIUS; y < EDGE_RADIUS; ++y) {
            float y_norm = float(y) / texture_size.y;
            avg_transparency += texture2D(texture, gl_TexCoord[0].xy + vec2(x_norm, y_norm)).a;
        }
    }
    avg_transparency /= float(4 * EDGE_RADIUS * EDGE_RADIUS);
    float light_weight = pow(1.0 - avg_transparency, 0.8);
    float glow = 4.0 * light_weight;

    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
        Point_Light light = point_lights[i];
        vec2 frag_to_light = light.position - world_pos;
        vec2 light_dir = normalize(frag_to_light);
        vec3 diffuse = light.intensity * light.color;

        float dist = length(frag_to_light);
        //float atten = float(dist < light.radius) * 1.0 / (1.0 + dist * light.attenuation);
        float atten = max(0.0, mix(1.0, 0.0, dist / light.radius));

        color += light_weight * (diffuse * (1.0 + glow) * atten);
    }

    gl_FragColor = vec4(color.rgb * pixel.rgb, pixel.a);
}
