//! Built-in CRT monitor effect.

use crate::config::CrtConfig;
use crate::gl;
use crate::Effect;

pub struct Crt {
    cfg: CrtConfig,
    u_scanline: i32,
    u_phosphor: i32,
    u_curvature: i32,
    u_vignette: i32,
    u_aberration: i32,
    u_screen_w: i32,
    u_screen_h: i32,
}

impl Default for Crt {
    fn default() -> Self {
        Self {
            cfg: CrtConfig::load(),
            u_scanline: -1,
            u_phosphor: -1,
            u_curvature: -1,
            u_vignette: -1,
            u_aberration: -1,
            u_screen_w: -1,
            u_screen_h: -1,
        }
    }
}

impl Effect for Crt {
    fn fragment_shader(&self) -> &str {
        FRAG_SRC
    }

    fn setup(&mut self, program: u32) {
        self.u_scanline = gl::get_uniform_location(program, "scanline_intensity");
        self.u_phosphor = gl::get_uniform_location(program, "phosphor_strength");
        self.u_curvature = gl::get_uniform_location(program, "curvature");
        self.u_vignette = gl::get_uniform_location(program, "vignette_strength");
        self.u_aberration = gl::get_uniform_location(program, "aberration");
        self.u_screen_w = gl::get_uniform_location(program, "screen_width");
        self.u_screen_h = gl::get_uniform_location(program, "screen_height");
    }

    fn set_uniforms(&self, _program: u32, width: i32, height: i32, _frame: u64) {
        gl::uniform_1f(self.u_scanline, self.cfg.scanline_intensity);
        gl::uniform_1f(self.u_phosphor, self.cfg.phosphor_strength);
        gl::uniform_1f(self.u_curvature, self.cfg.curvature);
        gl::uniform_1f(self.u_vignette, self.cfg.vignette);
        gl::uniform_1f(self.u_aberration, self.cfg.aberration);
        gl::uniform_1f(self.u_screen_w, width as f32);
        gl::uniform_1f(self.u_screen_h, height as f32);
    }

    fn enabled(&self) -> bool {
        self.cfg.enabled
    }
}

const FRAG_SRC: &str = r#"#version 330 core
in vec2 v_uv;
out vec4 o_color;

uniform sampler2D u_input;

uniform float scanline_intensity;
uniform float phosphor_strength;
uniform float curvature;
uniform float vignette_strength;
uniform float aberration;
uniform float screen_width;
uniform float screen_height;

// ── Barrel Distortion ──
vec2 barrel_distort(vec2 uv) {
    vec2 c = uv * 2.0 - 1.0;
    float r2 = dot(c, c);
    c *= 1.0 + r2 * curvature;
    return c * 0.5 + 0.5;
}

// ── Scanlines ──
float scanline(vec2 uv) {
    float y_pixel = uv.y * screen_height;
    float s = sin(y_pixel * 3.14159265);
    return 1.0 - scanline_intensity * s * s;
}

// ── Phosphor Glow (5-tap cross) ──
vec3 phosphor_glow(vec2 uv) {
    vec2 texel = vec2(1.0 / screen_width, 1.0 / screen_height);
    vec3 center = texture(u_input, uv).rgb;
    vec3 up     = texture(u_input, uv + vec2(0.0, texel.y)).rgb;
    vec3 down   = texture(u_input, uv - vec2(0.0, texel.y)).rgb;
    vec3 left   = texture(u_input, uv - vec2(texel.x, 0.0)).rgb;
    vec3 right  = texture(u_input, uv + vec2(texel.x, 0.0)).rgb;
    vec3 bloom = (center * 4.0 + up + down + left + right) / 8.0;
    float luma = dot(center, vec3(0.2126, 0.7152, 0.0722));
    float glow_factor = smoothstep(0.3, 1.0, luma);
    return mix(center, bloom * phosphor_strength, glow_factor * 0.3);
}

// ── Vignette ──
float vignette_mask(vec2 uv) {
    vec2 c = uv * 2.0 - 1.0;
    return 1.0 - dot(c, c) * vignette_strength;
}

// ── Chromatic Aberration ──
vec3 chromatic_sample(vec2 uv) {
    vec2 dir = uv - 0.5;
    float r = texture(u_input, uv + dir * aberration).r;
    float g = texture(u_input, uv).g;
    float b = texture(u_input, uv - dir * aberration).b;
    return vec3(r, g, b);
}

void main() {
    vec2 uv = barrel_distort(v_uv);

    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        o_color = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    vec3 color;
    if (aberration > 0.0001) {
        color = chromatic_sample(uv);
    } else {
        color = texture(u_input, uv).rgb;
    }

    if (phosphor_strength > 0.01) {
        vec3 glowed = phosphor_glow(uv);
        color = mix(color, glowed, 0.5);
    }

    color *= scanline(uv);
    color *= vignette_mask(uv);
    o_color = vec4(color, 1.0);
}
"#;
