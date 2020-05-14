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

#define MAX_POINT_LIGHTS 64

uniform sampler2D texture;
uniform Ambient_Light ambient_light;
uniform Point_Light point_lights[MAX_POINT_LIGHTS];

varying vec2 world_pos;

void main() {
    vec4 pixel = texture2D(texture, gl_TexCoord[0].xy);

 //   if (gl_FragCoord.x < 800.0)
        //pixel = vec4(pow(pixel.r, 1.0/2.2), pow(pixel.g, 1.0/2.2),pow(pixel.b, 1.0/2.2), pixel.a);
        //pixel = vec4(pow(pixel.r, 2.2), pow(pixel.g, 2.2),pow(pixel.b, 2.2), pixel.a);

    vec3 color = vec3(1.0);
    color *= vec3(ambient_light.color) * ambient_light.intensity;

    for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
        Point_Light light = point_lights[i];
        vec2 frag_to_light = light.position - world_pos;
        vec2 light_dir = normalize(frag_to_light);
        vec3 diffuse = light.intensity * light.color;

        float dist = length(frag_to_light);
        //float atten = float(dist < light.radius) * 1.0 / (1.0 + dist * light.attenuation);
        float atten = max(0.0, mix(1.0, 0.0, dist / light.radius));

        color += vec3(diffuse) * atten;
    }

    gl_FragColor = vec4(color.rgb * pixel.rgb, pixel.a);
    //if (gl_FragCoord.x < 800.0)
    //    gl_FragColor = vec4(pow(gl_FragColor.r, 2.2), pow(gl_FragColor.g, 2.2),pow(gl_FragColor.b, 2.2), gl_FragColor.a);
}
