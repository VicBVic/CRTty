//! Config file parser (~/.config/crtty/{kitty,alacritty}.conf).

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
        Self::defaults_for("kitty")
    }
}

impl CrtConfig {
    /// Return tuned defaults for a specific terminal emulator.
    pub fn defaults_for(app: &str) -> Self {
        match app {
            "alacritty" => Self {
                enabled: true,
                scanline_intensity: 0.05,
                phosphor_strength: 0.6,
                curvature: 0.02,
                vignette: 0.25,
                aberration: 0.005,
            },
            // kitty (and anything else) — the "reference" preset
            _ => Self {
                enabled: true,
                scanline_intensity: 0.35,
                phosphor_strength: 0.6,
                curvature: 0.09,
                vignette: 0.35,
                aberration: 0.009,
            },
        }
    }

    pub fn load() -> Self {
        let app = std::env::var("CRTTY_APP").unwrap_or_else(|_| "kitty".to_string());
        let mut cfg = Self::defaults_for(&app);
        migrate_legacy_config();
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
            if let Ok(f) = v.parse::<f32>() {
                cfg.scanline_intensity = f.clamp(0.0, 1.0);
            }
        }
        if let Some(v) = map.get("phosphor_strength") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.phosphor_strength = f.clamp(0.0, 3.0);
            }
        }
        if let Some(v) = map.get("curvature") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.curvature = f.clamp(0.0, 0.5);
            }
        }
        if let Some(v) = map.get("vignette") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.vignette = f.clamp(0.0, 2.0);
            }
        }
        if let Some(v) = map.get("aberration") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.aberration = f.clamp(0.0, 0.05);
            }
        }

        eprintln!("[CRTty] Config loaded from {:?}", path);
        cfg
    }
}

fn config_path() -> Option<PathBuf> {
    let app = std::env::var("CRTTY_APP").unwrap_or_else(|_| "kitty".to_string());
    let app_cfg = format!("{}.conf", app);

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(&xdg);
        if p.is_absolute() {
            return Some(p.join("crtty").join(&app_cfg));
        }
    }
    std::env::var("HOME").ok().map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("crtty")
            .join(&app_cfg)
    })
}

fn legacy_config_path() -> Option<PathBuf> {
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

fn migrate_legacy_config() {
    let Some(old) = legacy_config_path() else {
        return;
    };
    if !old.exists() {
        return;
    }

    let Some(new_path) = config_path() else {
        return;
    };
    if new_path.exists() {
        return;
    }

    if let Some(parent) = new_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("[CRTty] Failed creating config dir {:?}: {}", parent, e);
            return;
        }
    }

    match fs::rename(&old, &new_path) {
        Ok(_) => {
            eprintln!("[CRTty] Migrated legacy config {:?} -> {:?}", old, new_path);
        }
        Err(_) => match fs::copy(&old, &new_path) {
            Ok(_) => {
                let _ = fs::remove_file(&old);
                eprintln!("[CRTty] Migrated legacy config {:?} -> {:?}", old, new_path);
            }
            Err(e) => {
                eprintln!(
                    "[CRTty] Failed migrating legacy config {:?} -> {:?}: {}",
                    old, new_path, e
                );
            }
        },
    }
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
