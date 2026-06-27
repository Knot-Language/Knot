use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "knot", version, about = "Knot language compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New { name: String },
    Tie { #[arg(required = false)] _package: Option<String> },
    Untie { #[arg(required = false)] _package: Option<String> },
    Lsp,
    Build {
        source: Option<String>,
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },
    Run {
        source: Option<String>,
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },
}

fn main() {
    match Cli::parse().command {
        Command::New { name } => cmd_new(&name),
        Command::Tie { _package } => cmd_tie(_package),
        Command::Untie { _package } => cmd_untie(_package),
        Command::Lsp => knot::lsp::run(),
        Command::Build { source, output } => {
            cmd_build_or_run(source, output, false);
        }
        Command::Run { source, output } => {
            cmd_build_or_run(source, output, true);
        }
    }
}

fn cmd_new(name: &str) {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        eprintln!("error: project name '{}' must not contain path separators or '..'", name);
        std::process::exit(1);
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        eprintln!("error: project name '{}' must only contain letters, digits, underscores, and hyphens", name);
        std::process::exit(1);
    }
    let dir = std::path::Path::new(name);
    if dir.exists() {
        eprintln!("error: directory '{}' already exists", name);
        std::process::exit(1);
    }
    std::fs::create_dir(dir).expect("create dir");
    std::fs::create_dir(dir.join("src")).expect("create src");
    std::fs::write(
        dir.join("Knot.toml"),
        format!(
            "[project]\nname = \"{}\"\nversion = \"0.1.0\"\nentry = \"src/main.knot\"\n\n[dependencies]\n",
            name
        ),
    )
    .expect("write Knot.toml");
    std::fs::write(
        dir.join("src").join("main.knot"),
        "func main() -> I32 {\n    return 0\n}\n",
    )
    .expect("write main.knot");
    println!("created project '{}'", name);
}

fn cmd_tie(package: Option<String>) {
    let pkg = package.unwrap_or_else(|| {
        eprintln!("error: 'knot tie' requires a package name");
        std::process::exit(1);
    });
    let toml_path = "Knot.toml";
    if !std::path::Path::new(toml_path).exists() {
        eprintln!("error: 'Knot.toml' not found — run 'knot new <name>' first");
        std::process::exit(1);
    }
    let content = match std::fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", toml_path, e); std::process::exit(1); }
    };
    if content.contains(&format!("\"{}\"", pkg)) {
        eprintln!("note: dependency '{}' already exists in Knot.toml", pkg);
        return;
    }
    let new_content = if content.contains("[dependencies]\n") {
        content.replace("[dependencies]\n", &format!("[dependencies]\n\"{}\" = \"*\"\n", pkg))
    } else {
        format!("{}\n[dependencies]\n\"{}\" = \"*\"\n", content.trim_end(), pkg)
    };
    if let Err(e) = std::fs::write(toml_path, &new_content) {
        eprintln!("error: failed to write '{}': {}", toml_path, e);
        std::process::exit(1);
    }
    println!("added dependency '{}' to Knot.toml", pkg);
}

fn cmd_untie(package: Option<String>) {
    let pkg = package.unwrap_or_else(|| {
        eprintln!("error: 'knot untie' requires a package name");
        std::process::exit(1);
    });
    let toml_path = "Knot.toml";
    if !std::path::Path::new(toml_path).exists() {
        eprintln!("error: 'Knot.toml' not found");
        std::process::exit(1);
    }
    let content = match std::fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", toml_path, e); std::process::exit(1); }
    };
    let dep_line = format!("\"{}\" = \"*\"", pkg);
    if !content.contains(&dep_line) {
        eprintln!("note: dependency '{}' not found in Knot.toml", pkg);
        return;
    }
    let new_content = content
        .replace(&format!("{}\n", dep_line), "")
        .replace(&dep_line, "");
    if let Err(e) = std::fs::write(toml_path, &new_content) {
        eprintln!("error: failed to write '{}': {}", toml_path, e);
        std::process::exit(1);
    }
    println!("removed dependency '{}' from Knot.toml", pkg);
}

fn resolve_project_entry() -> Option<String> {
    let toml_path = "Knot.toml";
    if !std::path::Path::new(toml_path).exists() {
        return None;
    }
    let content = match std::fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(_) => return None,
    };
    for line in content.lines() {
        if line.trim().starts_with("entry") {
            if let Some(val) = line.split('=').nth(1) {
                let entry = val.trim().trim_matches('"');
                if !entry.is_empty() {
                    return Some(entry.to_string());
                }
            }
        }
    }
    None
}

fn cmd_build_or_run(source: Option<String>, output: Option<String>, run: bool) {
    let src = source.or_else(|| resolve_project_entry()).unwrap_or_else(|| {
        eprintln!("error: no source file specified and no 'Knot.toml' entry point found");
        eprintln!("hint: run 'knot new <name>' to create a project, or specify a source file");
        std::process::exit(1);
    });
    let out = output.unwrap_or_else(|| src.replace(".knot", ".exe"));
    knot::compiler::compile_file(&src, &out, run);
}
