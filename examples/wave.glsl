#version 330 core
in vec2 v_uv;
out vec4 o_color;
uniform sampler2D u_input;
uniform float u_time;
uniform vec2 u_resolution;

void main() {
    float wave = sin(v_uv.y * 40.0 + u_time * 3.0) * 0.003;
    vec3 c;
    c.r = texture(u_input, v_uv + vec2(wave, 0.0)).r;
    c.g = texture(u_input, v_uv).g;
    c.b = texture(u_input, v_uv - vec2(wave, 0.0)).b;

    float scan = 0.95 + 0.05 * sin(v_uv.y * u_resolution.y * 3.14159 + u_time * 2.0);
    c *= scan;

    o_color = vec4(c, 1.0);
}
