use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::ir::lower::Lower;
use crate::codegen::llvm::LlvmBackend;

pub fn compile_file(source_path: &str, output_path: &str, run: bool) {
    let source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", source_path, e));

    let mut parser = Parser::new(&source);
    let program = parser.parse_program();

    match SemanticAnalyzer::analyze(&program) {
        Ok(_) => {}
        Err(errors) => {
            for e in &errors {
                eprintln!("error: {}", e);
            }
            std::process::exit(1);
        }
    }

    let tac = Lower::lower(&program);
    let ll = LlvmBackend::generate(&tac);

    let base = output_path
        .strip_suffix(".exe")
        .unwrap_or(output_path);
    let ll_path = format!("{}.ll", base);
    std::fs::write(&ll_path, &ll).expect("write .ll");

    LlvmBackend::compile_to_exe(&ll_path, output_path);
    println!("  compiled {}", output_path);

    if run {
        let exe = if output_path.contains('/') || output_path.contains('\\') {
            output_path.to_string()
        } else {
            format!(".\\{}", output_path)
        };
        let status = std::process::Command::new(&exe)
            .status()
            .expect("run exe");
        std::process::exit(status.code().unwrap_or(1));
    }
}
