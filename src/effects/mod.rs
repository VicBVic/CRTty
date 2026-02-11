pub mod crt;
pub mod greyscale;
pub mod invert;
pub use crt::Crt;
pub use greyscale::Greyscale;
pub use invert::Invert;

use crate::Effect;

/// Built-in effect dispatcher. Chosen at runtime via `CRTTY_EFFECT` env var.
pub enum Builtin {
    Crt(Crt),
    Greyscale(Greyscale),
    Invert(Invert),
}

impl Builtin {
    /// Select effect from the `CRTTY_EFFECT` env var. Defaults to CRT.
    pub fn from_env() -> Self {
        match std::env::var("CRTTY_EFFECT").as_deref() {
            Ok("greyscale") => Self::Greyscale(Greyscale),
            Ok("invert") => Self::Invert(Invert),
            _ => Self::Crt(Crt::default()),
        }
    }

    pub const AVAILABLE: &'static [&'static str] = &["crt", "greyscale", "invert"];
}

impl Effect for Builtin {
    fn fragment_shader(&self) -> &str {
        match self {
            Self::Crt(e) => e.fragment_shader(),
            Self::Greyscale(e) => e.fragment_shader(),
            Self::Invert(e) => e.fragment_shader(),
        }
    }

    fn setup(&mut self, program: u32) {
        match self {
            Self::Crt(e) => e.setup(program),
            Self::Greyscale(_) => {}
            Self::Invert(_) => {}
        }
    }

    fn set_uniforms(&self, program: u32, w: i32, h: i32, frame: u64) {
        match self {
            Self::Crt(e) => e.set_uniforms(program, w, h, frame),
            Self::Greyscale(_) => {}
            Self::Invert(_) => {}
        }
    }

    fn enabled(&self) -> bool {
        match self {
            Self::Crt(e) => e.enabled(),
            Self::Greyscale(_) => true,
            Self::Invert(_) => true,
        }
    }
}
