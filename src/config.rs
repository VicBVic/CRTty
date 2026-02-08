//! Config file parser (~/.config/crtty.conf).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CrtConfig {
    pub enabled: bool,
    pub scanline_intensity: f32,
    pub phosphor_strength: f32,
    pub curvature: f32,
    pub vignette: f32,
    pub aberration: f32,
}

impl Default for CrtConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scanline_intensity: 0.75,
            phosphor_strength: 1.1,
            curvature: 0.04,
            vignette: 0.35,
            aberration: 0.003,
        }
    }
}

impl CrtConfig {
    pub fn load() -> Self {
        let mut cfg = Self::default();
        let path = match config_path() {
            Some(p) if p.exists() => p,
            _ => {
                eprintln!("[CRTty] No config file found, using defaults");
                return cfg;
            }
        };
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[CRTty] Failed to read {:?}: {}", path, e);
                return cfg;
            }
        };
        let map = parse_ini(&content);

        if let Some(v) = map.get("enabled") {
            cfg.enabled = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Some(v) = map.get("scanline_intensity") {
            if let Ok(f) = v.parse::<f32>() { cfg.scanline_intensity = f.clamp(0.0, 1.0); }
        }
        if let Some(v) = map.get("phosphor_strength") {
            if let Ok(f) = v.parse::<f32>() { cfg.phosphor_strength = f.clamp(0.0, 3.0); }
        }
        if let Some(v) = map.get("curvature") {
            if let Ok(f) = v.parse::<f32>() { cfg.curvature = f.clamp(0.0, 0.5); }
        }
        if let Some(v) = map.get("vignette") {
            if let Ok(f) = v.parse::<f32>() { cfg.vignette = f.clamp(0.0, 2.0); }
        }
        if let Some(v) = map.get("aberration") {
            if let Ok(f) = v.parse::<f32>() { cfg.aberration = f.clamp(0.0, 0.05); }
        }

        eprintln!("[CRTty] Config loaded from {:?}", path);
        cfg
    }
}

fn config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(&xdg);
        if p.is_absolute() {
            return Some(p.join("crtty.conf"));
        }
    }
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config").join("crtty.conf"))
}

fn parse_ini(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    map
}
