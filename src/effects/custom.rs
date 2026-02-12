//! Custom GLSL shader loaded from a file at runtime.

use crate::Effect;

pub struct Custom {
    source: String,
}

impl Custom {
    pub fn from_file(path: &str) -> Self {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("[CRTty] failed to read shader '{path}': {e}");
            std::process::exit(1);
        });
        Self { source }
    }
}

impl Effect for Custom {
    fn fragment_shader(&self) -> &str {
        &self.source
    }
}
