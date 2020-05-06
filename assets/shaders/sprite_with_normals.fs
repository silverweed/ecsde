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
};

#define MAX_POINT_LIGHTS 64
#define DIFFUSE_BIAS 0.2
#define MAX_ENCODED_ANGLE 65535
#define PI 3.14159265359

uniform sampler2D texture;
uniform sampler2D normals;
uniform Ambient_Light ambient_light;
uniform Point_Light point_lights[MAX_POINT_LIGHTS];
uniform float shininess;
uniform vec3 specular_color;

varying vec2 world_pos;

float decode_rot(vec4 color) {
    uint r = uint(255.0 * color.r);
    uint g = uint(255.0 * color.g);
    uint b = uint(255.0 * color.b);
    uint a = uint(255.0 * color.a);

    return float((r << 24) | (g << 16) | (b << 8) | a) / float(MAX_ENCODED_ANGLE) * 2.0 * PI;
}

void main() {
    vec4 pixel = texture2D(texture, gl_TexCoord[0].xy);

    // Note: gl_Color contains the sprite's rotation
    float sprite_rot = decode_rot(gl_Color);
    vec4 color = vec4(1.0, 1.0, 1.0, 1.0);

    color *= vec4(ambient_light.color, 1.0) * ambient_light.intensity;

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

        vec3 result = diffuse + specular;

        float dist = length(frag_to_light);
        //float atten = float(dist < light.radius) * 1.0 / (1.0 + dist * light.attenuation);
        float atten = max(0.0, mix(1.0, 0.0, dist / light.radius));

        color += vec4(result, 0.0) * atten;
    }

    gl_FragColor = color * pixel;
}
