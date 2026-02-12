#version 330 core
in vec2 v_uv;
out vec4 o_color;
uniform sampler2D u_input;
uniform float u_time;
uniform vec2 u_resolution;

// Barrel distortion
vec2 curve(vec2 uv) {
    uv = uv * 2.0 - 1.0;
    uv *= 1.0 + dot(uv, uv) * 0.06;
    return uv * 0.5 + 0.5;
}

void main() {
    vec2 uv = curve(v_uv);

    // Off-screen: black with rounded corners
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        o_color = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    // Chromatic aberration — shifts drift slowly
    float abb = 0.0025 + sin(u_time * 0.7) * 0.0005;
    vec3 c;
    c.r = texture(u_input, vec2(uv.x + abb, uv.y)).r;
    c.g = texture(u_input, uv).g;
    c.b = texture(u_input, vec2(uv.x - abb, uv.y)).b;

    // Phosphor sub-pixel grid
    float px = floor(mod(gl_FragCoord.x, 3.0));
    vec3 mask = vec3(0.8);
    if      (px == 0.0) mask.r = 1.0;
    else if (px == 1.0) mask.g = 1.0;
    else                mask.b = 1.0;
    c *= mix(vec3(1.0), mask, 0.25);

    // Rolling scanlines — move downward like a real CRT
    float scanY = gl_FragCoord.y + u_time * 60.0;
    float scan = 0.92 + 0.08 * sin(scanY * 3.14159 * 2.0 / 2.0);
    c *= scan;

    // Random-ish per-line flicker (hash based on scanline + time)
    float line = floor(gl_FragCoord.y);
    float seed = fract(sin(line * 43.7589 + floor(u_time * 12.0) * 17.137) * 4758.5453);
    float lineFlicker = 0.97 + 0.03 * (seed * 2.0 - 1.0);
    c *= lineFlicker;

    // Occasional bright scanline streak (random hot line)
    float streak = smoothstep(0.992, 1.0, seed) * 0.15;
    c += streak;

    // Thicker slow-rolling bar (interference band)
    float bar = 1.0 - 0.06 * smoothstep(0.0, 1.0,
        sin((v_uv.y + u_time * 0.12) * 6.2832) * 0.5 + 0.5);
    c *= bar;

    // Phosphor glow / bloom
    c = pow(c, vec3(0.85)) * 1.15;

    // Flicker — subtle brightness oscillation
    c *= 0.97 + 0.03 * sin(u_time * 60.0 * 3.14159);

    // Vignette
    vec2 vig = v_uv * (1.0 - v_uv);
    float v = pow(vig.x * vig.y * 16.0, 0.3);
    c *= v;

    // Slight green/amber tint for that old monitor feel
    c *= vec3(0.92, 1.0, 0.88);

    o_color = vec4(c, 1.0);
}
