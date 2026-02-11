//! Color inversion effect.

use crate::Effect;

pub struct Invert;

impl Effect for Invert {
    fn fragment_shader(&self) -> &str {
        "#version 330 core
         in vec2 v_uv;
         out vec4 o_color;
         uniform sampler2D u_input;
         void main() {
             o_color = vec4(1.0 - texture(u_input, v_uv).rgb, 1.0);
         }"
    }
}
