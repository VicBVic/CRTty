//! Greyscale effect.

use crate::Effect;

pub struct Greyscale;

impl Effect for Greyscale {
    fn fragment_shader(&self) -> &str {
        "#version 330 core
         in vec2 v_uv;
         out vec4 o_color;
         uniform sampler2D u_input;
         void main() {
             vec3 c = texture(u_input, v_uv).rgb;
             float l = dot(c, vec3(0.299, 0.587, 0.114));
             o_color = vec4(l, l, l, 1.0);
         }"
    }
}
