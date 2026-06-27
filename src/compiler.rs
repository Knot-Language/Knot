use crate::error::DiagnosticBag;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::ir::lower::Lower;
use crate::codegen::llvm::LlvmBackend;

pub fn compile_file(source_path: &str, output_path: &str, run: bool) {
    let source = match std::fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", source_path, e);
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(&source);
    let program = parser.parse_program();

    if parser.diagnostics.has_errors() {
        eprintln!("\n--- parse errors ---");
        parser.diagnostics.print_all();
        std::process::exit(1);
    }
    if !parser.diagnostics.diagnostics.is_empty() {
        parser.diagnostics.print_all();
    }

    let mut diagnostics = DiagnosticBag::new(&source);
    match SemanticAnalyzer::analyze(&program, &mut diagnostics) {
        Ok(_) => {}
        Err(errors) => {
            for (msg, _) in &errors {
                eprintln!("error: {}", msg);
            }
            diagnostics.print_all();
            std::process::exit(1);
        }
    }
    if !diagnostics.diagnostics.is_empty() {
        diagnostics.print_all();
    }

    let tac = Lower::lower(&program);
    let ll = LlvmBackend::generate(&tac);

    let base = output_path
        .strip_suffix(".exe")
        .unwrap_or(output_path);
    let ll_path = format!("{}.ll", base);

    if output_path.contains("..") {
        eprintln!("warning: output path contains '..', this may write files outside the current directory");
    }

    if let Err(e) = std::fs::write(&ll_path, &ll) {
        eprintln!("error: failed to write '{}': {}", ll_path, e);
        std::process::exit(1);
    }

    LlvmBackend::compile_to_exe(&ll_path, output_path);
    println!("  compiled {}", output_path);

    if run {
        let exe = if output_path.contains('/') || output_path.contains('\\') {
            output_path.to_string()
        } else {
            format!(".\\{}", output_path)
        };
        match std::process::Command::new(&exe).status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("error: failed to run '{}': {}", exe, e);
                std::process::exit(1);
            }
        }
    }
}
