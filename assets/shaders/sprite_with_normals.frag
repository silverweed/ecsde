#extension GL_EXT_gpu_shader4 : enable // required for bitshift operator

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

struct Rect_Light {
    vec3 color;
    vec2 pos_min;
    vec2 pos_max;
    float radius;
    float attenuation;
    float intensity;
};

#define MAX_POINT_LIGHTS 8
#define MAX_RECT_LIGHTS 8
#define DIFFUSE_BIAS 0.2
#define MAX_ENCODED_ANGLE 65535
#define PI 3.14159265359

uniform sampler2D texture;
uniform sampler2D normals;
uniform Ambient_Light ambient_light;
uniform Point_Light point_lights[MAX_POINT_LIGHTS];
uniform float shininess;
uniform vec3 specular_color;
uniform Rect_Light rect_lights[MAX_RECT_LIGHTS];

varying vec2 world_pos;

float decode_rot(vec4 color) {
    int r = int(255.0 * color.r);
    int g = int(255.0 * color.g);

    return float((r << 8) | g) / float(MAX_ENCODED_ANGLE) * 2.0 * PI;
}

vec2 point_to_rect_vector(vec2 point_pos, vec2 rect_pos_min, vec2 rect_pos_max) {
    vec2 pos_relative_to_rect = point_pos - rect_pos_min;

    float rect_half_width = 0.5 * (rect_pos_max.x - rect_pos_min.x);
    float dist_x = max(0.0, abs(pos_relative_to_rect.x - rect_half_width) - rect_half_width);

    float rect_half_height = 0.5 * (rect_pos_max.y - rect_pos_min.y);
    float dist_y = max(0.0, abs(pos_relative_to_rect.y - rect_half_height) - rect_half_height);
    
    return -vec2(dist_x * sign(pos_relative_to_rect.x - rect_half_width), dist_y * sign(pos_relative_to_rect.y - rect_half_height));
}

void main() {
    vec4 pixel = texture2D(texture, gl_TexCoord[0].xy);

    // Note: gl_Color.rg contains the sprite's rotation
    float sprite_rot = decode_rot(gl_Color);
    float vert_alpha = gl_Color.a;
    vec3 color = vec3(1.0);

    color *= ambient_light.color * ambient_light.intensity;

    vec3 normal = 2.0 * (texture2D(normals, gl_TexCoord[0].xy).xyz - 0.5);
    float cos_a = cos(sprite_rot);
    float sin_a = sin(sprite_rot);
    mat3 rotation = mat3(
        cos_a, -sin_a, 0.0,
        sin_a,  cos_a, 0.0,
        0.0,    0.0,   1.0
    );
    normal = rotation * normal;
    normal.z *= -1.0;
    normal.y *= -1.0;
    normal = normalize(normal);

    vec3 view_dir = vec3(0.0, 0.0, -1.0);

    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
        Point_Light light = point_lights[i];
        vec2 frag_to_light = light.position - world_pos;
        vec3 light_dir = normalize(vec3(frag_to_light, 0.0));
        float diff = max(dot(normal, light_dir), 0.0);
        vec3 diffuse = (DIFFUSE_BIAS + diff) * light.color;

        vec3 half_dir = normalize(light_dir + view_dir);
        float spec = pow(max(dot(half_dir, normal), 0.0), max(1.0, shininess));
        vec3 specular = specular_color * spec * light.color;

        vec3 result = light.intensity * (diffuse + specular);

        float dist = length(frag_to_light);
        float atten = pow(max(0.0, mix(1.0, 0.0, dist / light.radius)), 1.0 + light.attenuation);

        color += result * atten;
    }

    for (int i = 0; i < MAX_RECT_LIGHTS; ++i) {
        Rect_Light light = rect_lights[i];
        vec2 frag_to_light = point_to_rect_vector(world_pos, light.pos_min, light.pos_max);
        vec3 light_dir = normalize(vec3(frag_to_light, 0.0));
        float dist = length(frag_to_light);
        float diff = 1.0;
        if (dist > 0.0) {
            diff = max(dot(normal, light_dir), 0.0);
        }
        vec3 diffuse = (DIFFUSE_BIAS + diff) * light.color;

        vec3 half_dir = normalize(light_dir + view_dir);
        float spec = pow(max(dot(half_dir, normal), 0.0), max(1.0, shininess));
        vec3 specular = specular_color * spec * light.color;

        vec3 result = light.intensity * (diffuse + specular);

        float atten = pow(max(0.0, mix(1.0, 0.0, dist / light.radius)), 1.0 + light.attenuation);

        color += result * atten;
    }

    gl_FragColor = vec4(color.rgb * pixel.rgb, pixel.a * vert_alpha);
}
