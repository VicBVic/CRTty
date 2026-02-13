use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

const EFFECTS: &[&str] = &["crt", "greyscale", "invert"];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut effect = "crt".to_string();
    let mut app = "kitty".to_string();
    let mut migrate_only = false;
    let mut app_args = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--migrate-config" => {
                migrate_only = true;
            }
            "-a" | "--app" => {
                i += 1;
                if i < args.len() {
                    app = args[i].clone();
                } else {
                    eprintln!("crtty: --app requires a value (kitty or alacritty)");
                    std::process::exit(1);
                }
            }
            "-s" | "--shader" => {
                i += 1;
                if i < args.len() {
                    effect = args[i].clone();
                } else {
                    eprintln!("crtty: --shader requires a name");
                    std::process::exit(1);
                }
            }
            "-l" | "--list" => {
                println!("Available effects:");
                for e in EFFECTS {
                    let tag = if *e == "crt" { " (default)" } else { "" };
                    println!("  {e}{tag}");
                }
                return;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            "--" => {
                app_args.extend_from_slice(&args[i + 1..]);
                break;
            }
            _ => app_args.push(args[i].clone()),
        }
        i += 1;
    }

    if app != "kitty" && app != "alacritty" {
        eprintln!("crtty: unsupported app '{app}'");
        eprintln!("Supported: kitty, alacritty");
        std::process::exit(1);
    }

    if migrate_only {
        migrate_config_for_app(&app);
        return;
    }

    if !is_custom_shader(&effect) && !EFFECTS.contains(&effect.as_str()) {
        eprintln!("crtty: unknown effect '{effect}'");
        eprintln!("Run `crtty --list` for available effects.");
        eprintln!("Or pass a path to a .glsl file.");
        std::process::exit(1);
    }

    if is_custom_shader(&effect) {
        let path = std::path::Path::new(&effect);
        if !path.exists() {
            eprintln!("crtty: shader file not found: {effect}");
            std::process::exit(1);
        }
        if let Ok(abs) = std::fs::canonicalize(path) {
            effect = abs.to_string_lossy().to_string();
        }
    }

    let lib = find_lib();
    if !std::path::Path::new(&lib).exists() {
        eprintln!("crtty: library not found at {lib}");
        eprintln!("Run `make install` first.");
        std::process::exit(1);
    }

    let err = Command::new(&app)
        .args(&app_args)
        .env("LD_PRELOAD", &lib)
        .env("ENABLE_CRTTY", "1")
        .env("CRTTY_APP", &app)
        .env("CRTTY_EFFECT", &effect)
        .exec();

    eprintln!("crtty: failed to launch {app}: {err}");
    std::process::exit(1);
}

fn find_lib() -> String {
    if let Ok(p) = env::var("CRTTY_LIB") {
        return p;
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let lib = bin_dir
                .parent()
                .unwrap_or(bin_dir)
                .join("lib/libcrtty_crt.so");
            if lib.exists() {
                return lib.to_string_lossy().to_string();
            }
        }
    }
    let home = env::var("HOME").unwrap_or_default();
    format!("{home}/.local/lib/libcrtty_crt.so")
}

fn is_custom_shader(name: &str) -> bool {
    name.ends_with(".glsl") || name.contains('/')
}

fn print_help() {
    println!(
        "\
crtty — post-processing shaders for kitty/alacritty

USAGE:
    crtty [OPTIONS] [-- APP_ARGS...]

OPTIONS:
    -a, --app <APP>           App to launch: kitty | alacritty (default: kitty)
    -s, --shader <NAME|PATH>  Effect name or path to .glsl file (default: crt)
    -l, --list                List available built-in effects
    --migrate-config       Move legacy ~/.config/crtty.conf to per-app config
    -h, --help                Show this help

EXAMPLES:
    crtty                               CRT monitor effect (kitty)
    crtty --app alacritty               Launch alacritty with CRTty
    crtty -s greyscale                  Greyscale shader
    crtty -s ./my_shader.glsl           Custom GLSL file
    crtty --migrate-config              Migrate legacy config to kitty.conf
    crtty --app alacritty -- --hold     CRT + pass args to alacritty"
    );
}

fn config_dir() -> Option<std::path::PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let p = std::path::PathBuf::from(&xdg);
        if p.is_absolute() {
            return Some(p);
        }
    }
    env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".config"))
}

fn migrate_config_for_app(app: &str) {
    let Some(base) = config_dir() else {
        eprintln!("crtty: could not determine config directory");
        std::process::exit(1);
    };

    let legacy = base.join("crtty.conf");
    let new_dir = base.join("crtty");
    let target = new_dir.join(format!("{app}.conf"));

    if !legacy.exists() {
        eprintln!("crtty: no legacy config found at {}", legacy.display());
        return;
    }
    if target.exists() {
        eprintln!(
            "crtty: target already exists (won't overwrite): {}",
            target.display()
        );
        return;
    }

    if let Err(e) = std::fs::create_dir_all(&new_dir) {
        eprintln!("crtty: failed to create {}: {e}", new_dir.display());
        std::process::exit(1);
    }

    match std::fs::rename(&legacy, &target) {
        Ok(_) => {
            println!(
                "crtty: migrated {} -> {}",
                legacy.display(),
                target.display()
            );
        }
        Err(_) => match std::fs::copy(&legacy, &target) {
            Ok(_) => {
                let _ = std::fs::remove_file(&legacy);
                println!(
                    "crtty: migrated {} -> {}",
                    legacy.display(),
                    target.display()
                );
            }
            Err(e) => {
                eprintln!(
                    "crtty: failed migrating {} -> {}: {e}",
                    legacy.display(),
                    target.display()
                );
                std::process::exit(1);
            }
        },
    }
}
