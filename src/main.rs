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
    Lsp,
}

fn main() {
    match Cli::parse().command {
        Command::New { name } => cmd_new(&name),
        Command::Tie { .. } => cmd_stub("tie"),
        Command::Untie { .. } => cmd_stub("untie"),
        Command::Build { source, output } => {
            cmd_build_or_run(source, output, false);
        }
        Command::Run { source, output } => {
            cmd_build_or_run(source, output, true);
        }
        Command::Lsp => {
            knot::lsp::run_lsp();
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

fn cmd_stub(cmd: &str) {
    eprintln!("knot {}: not yet implemented", cmd);
    std::process::exit(1);
}

fn cmd_build_or_run(source: Option<String>, output: Option<String>, run: bool) {
    let src = source.unwrap_or_else(|| {
        eprintln!("error: project build not yet implemented; please specify a source file");
        std::process::exit(1);
    });
    let out = output.unwrap_or_else(|| src.replace(".knot", ".exe"));
    knot::compiler::compile_file(&src, &out, run);
}
