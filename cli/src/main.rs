use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

const EFFECTS: &[&str] = &["crt", "greyscale", "invert"];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut effect = "crt".to_string();
    let mut kitty_args = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
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
                kitty_args.extend_from_slice(&args[i + 1..]);
                break;
            }
            _ => kitty_args.push(args[i].clone()),
        }
        i += 1;
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

    let err = Command::new("kitty")
        .args(&kitty_args)
        .env("LD_PRELOAD", &lib)
        .env("ENABLE_CRTTY", "1")
        .env("CRTTY_EFFECT", &effect)
        .exec();

    eprintln!("crtty: failed to launch kitty: {err}");
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
crtty — post-processing shaders for kitty terminal

USAGE:
    crtty [OPTIONS] [-- KITTY_ARGS...]

OPTIONS:
    -s, --shader <NAME|PATH>  Effect name or path to .glsl file (default: crt)
    -l, --list                List available built-in effects
    -h, --help                Show this help

EXAMPLES:
    crtty                              CRT monitor effect
    crtty -s greyscale                 Greyscale shader
    crtty -s ./my_shader.glsl          Custom GLSL file
    crtty -s crt -- --hold -e htop     CRT + pass args to kitty"
    );
}
