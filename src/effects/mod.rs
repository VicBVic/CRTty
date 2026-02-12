pub mod crt;
pub mod custom;
pub mod greyscale;
pub mod invert;
pub use crt::Crt;
pub use custom::Custom;
pub use greyscale::Greyscale;
pub use invert::Invert;

use crate::Effect;

/// Built-in effect dispatcher. Chosen at runtime via `CRTTY_EFFECT` env var.
pub enum Builtin {
    Crt(Crt),
    Greyscale(Greyscale),
    Invert(Invert),
    Custom(Custom),
}

impl Builtin {
    /// Select effect from the `CRTTY_EFFECT` env var. Defaults to CRT.
    pub fn from_env() -> Self {
        match std::env::var("CRTTY_EFFECT").as_deref() {
            Ok("greyscale") => Self::Greyscale(Greyscale),
            Ok("invert") => Self::Invert(Invert),
            Ok(path) if path.ends_with(".glsl") || path.contains('/') => {
                Self::Custom(Custom::from_file(path))
            }
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
            Self::Custom(e) => e.fragment_shader(),
        }
    }

    fn setup(&mut self, program: u32) {
        match self {
            Self::Crt(e) => e.setup(program),
            Self::Greyscale(_) => {}
            Self::Invert(_) => {}
            Self::Custom(_) => {}
        }
    }

    fn set_uniforms(&self, program: u32, w: i32, h: i32, frame: u64) {
        match self {
            Self::Crt(e) => e.set_uniforms(program, w, h, frame),
            Self::Greyscale(_) => {}
            Self::Invert(_) => {}
            Self::Custom(_) => {}
        }
    }

    fn enabled(&self) -> bool {
        match self {
            Self::Crt(e) => e.enabled(),
            Self::Greyscale(_) => true,
            Self::Invert(_) => true,
            Self::Custom(_) => true,
        }
    }
}
