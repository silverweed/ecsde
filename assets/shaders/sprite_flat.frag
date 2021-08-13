#version 330 core

// size: 16 B
struct Ambient_Light {
    vec4 color_and_intensity;
};

// size: 32 B
struct Point_Light {
    vec4 color_and_intensity;
    vec2 position;
    float radius;
    float attenuation;
};

// size: 48 B
struct Rect_Light {
    vec4 color_and_intensity;
    vec2 pos_min;
    vec2 pos_max;
    float radius;
    float attenuation;
    float _pad1;
    float _pad2;
};

#define MAX_POINT_LIGHTS 4
#define MAX_RECT_LIGHTS 4

uniform sampler2D tex;

layout (std140) uniform LightsBlock {
	Ambient_Light ambient_light;
	Point_Light point_lights[MAX_POINT_LIGHTS];
	Rect_Light rect_lights[MAX_RECT_LIGHTS];
};

in vec4 color;
in vec2 world_pos;
in vec2 tex_coord;

out vec4 frag_color;

vec2 point_to_rect_vector(vec2 point_pos, vec2 rect_pos_min, vec2 rect_pos_max) {
    vec2 pos_relative_to_rect = point_pos - rect_pos_min;

    float rect_half_width = 0.5 * (rect_pos_max.x - rect_pos_min.x);
    float dist_x = max(0.0, abs(pos_relative_to_rect.x - rect_half_width) - rect_half_width);

    float rect_half_height = 0.5 * (rect_pos_max.y - rect_pos_min.y);
    float dist_y = max(0.0, abs(pos_relative_to_rect.y - rect_half_height) - rect_half_height);

    return vec2(dist_x * sign(pos_relative_to_rect.x - rect_half_width), dist_y * sign(pos_relative_to_rect.y - rect_half_height));
}

void main() {
    vec4 pixel = texture(tex, tex_coord);

    vec3 color = vec3(1.0);
    color *= vec3(ambient_light.color_and_intensity.rgb) * ambient_light.color_and_intensity.w;

    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
        Point_Light light = point_lights[i];
        vec2 frag_to_light = light.position - world_pos;
        vec3 diffuse = light.color_and_intensity.w * light.color_and_intensity.rgb;

        float dist = length(frag_to_light);
        float atten = pow(max(0.0, mix(1.0, 0.0, dist / light.radius)), 1.0 + light.attenuation);

        color += vec3(diffuse) * atten;
    }

    for (int i = 0; i < MAX_RECT_LIGHTS; ++i) {
        Rect_Light light = rect_lights[i];

        vec2 frag_to_light = point_to_rect_vector(world_pos, light.pos_min, light.pos_max);
        vec2 light_dir = normalize(frag_to_light);
        vec3 diffuse = light.color_and_intensity.w * light.color_and_intensity.rgb;

        float dist = length(frag_to_light);
        float atten = pow(max(0.0, mix(1.0, 0.0, dist / light.radius)), 1.0 + light.attenuation);

        color += vec3(diffuse) * atten;
    }

    frag_color = vec4(color * pixel.rgb, pixel.a);
}
