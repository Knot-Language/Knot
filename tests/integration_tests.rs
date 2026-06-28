use knot::codegen::llvm::LlvmBackend;
use knot::ir::lower::Lower;
use knot::parser::Parser;
use knot::semantic::SemanticAnalyzer;
use knot::error::DiagnosticBag;

fn compile(source: &str) -> Result<String, String> {
    let mut parser = Parser::new(source);
    let program = parser.parse_program();

    let mut diagnostics = DiagnosticBag::new(source);
    match SemanticAnalyzer::analyze(&program, &mut diagnostics) {
        Ok(_) => {}
        Err(errors) => {
            return Err(errors.iter().map(|(m, _)| m.clone()).collect::<Vec<_>>().join("\n"));
        }
    }

    let tac = Lower::lower(&program);
    let ll = LlvmBackend::generate(&tac);
    Ok(ll)
}

fn compile_success(source: &str) -> String {
    match compile(source) {
        Ok(ll) => ll,
        Err(e) => panic!("compilation failed:\n{}", e),
    }
}

// ── Basic Programs ─────────────────────────────────

#[test]
fn simple_main_returns_zero() {
    let ll = compile_success("func main() -> I32 { return 0 }");
    assert!(ll.contains("define i32 @main("));
    assert!(ll.contains("ret i32 0"));
}

#[test]
fn main_with_addition() {
    let ll = compile_success("func main() -> I32 { x = 1\n y = 2\n return x + y }");
    assert!(ll.contains("@main("));
}

#[test]
fn main_with_if_else() {
    let ll = compile_success("func main() -> I32 { if true { return 1 } else { return 0 } }");
    assert!(ll.contains("@main("));
    assert!(ll.contains("br"));
}

#[test]
fn main_with_while() {
    let ll = compile_success("func main() -> I32 { x = 0\n while x < 10 { x = x + 1 }\n return x }");
    assert!(ll.contains("@main("));
}

#[test]
fn for_loop_range() {
    let ll = compile_success("func main() -> I32 { s = 0\n for i in 0..10 { s = s + i }\n return s }");
    assert!(ll.contains("@main("));
}

#[test]
fn function_call_basic() {
    let ll = compile_success(
        "func add(a: I32, b: I32) -> I32 { return a + b }\nfunc main() -> I32 { return add(1, 2) }",
    );
    assert!(ll.contains("@add("));
    assert!(ll.contains("call i32 @add("));
}

#[test]
fn multiple_functions() {
    let ll = compile_success(
        "func double(x: I32) -> I32 { return x * 2 }\nfunc main() -> I32 { return double(21) }",
    );
    assert!(ll.contains("@double("));
    assert!(ll.contains("@main("));
}

// ── Generics ───────────────────────────────────────

#[test]
fn generic_identity_function() {
    let ll = compile_success(
        "func identity[T](x: T) -> T { return x }\nfunc main() -> I32 { return identity(42) }",
    );
    assert!(ll.contains("@identity_I32("));
    assert!(ll.contains("call i32 @identity_I32("));
}

// ── Types ──────────────────────────────────────────

#[test]
fn array_type_annotation() {
    let ll = compile_success(
        "func process(arr: Array[I32]) -> I32 { return 0 }\nfunc main() -> I32 { a = [1, 2, 3]\n return process(a) }",
    );
    assert!(ll.contains("@process("));
}

#[test]
fn float_operations() {
    let ll = compile_success("func main() -> I32 { x = 3.14\n y = 2.0\n z = x + y\n return 0 }");
    assert!(ll.contains("fadd double"));
}

// ── Exception Handling ─────────────────────────────

#[test]
fn try_catch_basic() {
    let ll = compile_success(
        "func main() -> I32 { try { return 1 } catch e { return 0 } }",
    );
    assert!(ll.contains("@main("));
    assert!(ll.contains("catch"));
}

#[test]
fn try_catch_with_throw() {
    let ll = compile_success(
        "func main() -> I32 { try { throw 404 } catch e { return e } }",
    );
    assert!(ll.contains("@main("));
    assert!(ll.contains("@knot_exception"));
}

#[test]
fn nested_try_catch() {
    let ll = compile_success(
        "func main() -> I32 { try { try { throw 1 } catch e { throw 2 } } catch e { return e } }",
    );
    assert!(ll.contains("@main("));
    assert!(ll.contains("@knot_exception"));
}

// ── Class ──────────────────────────────────────────

#[test]
fn class_definition_and_constructor() {
    let ll = compile_success(
        "class Point { x: I32\n func new(x: I32) { this.x = x } }\nfunc main() -> I32 { p = Point::new(10)\n return 0 }",
    );
    assert!(ll.contains("%Point = type"));
    assert!(ll.contains("@Point__new("));
}

#[test]
fn enum_definition() {
    let ll = compile_success(
        "enum Color { Red\n Green\n Blue }\nfunc main() -> I32 { return 0 }",
    );
    assert!(ll.contains("@main("));
}

// ── Error Cases ────────────────────────────────────

#[test]
fn type_mismatch_gives_error() {
    let result = compile(
        "func main() -> I32 { x = 42\n x = true\n return x }",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("type mismatch") || err.contains("cannot assign"), "got: {}", err);
}

#[test]
fn undefined_variable_gives_error() {
    let result = compile(
        "func main() -> I32 { return unknown }",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable"));
}

// ── Imports (check basic compilation without real files) ──

#[test]
fn import_statement_parsed_and_lowered() {
    let ll = compile_success(
        "import \"nonexistent.knot\"\nfunc main() -> I32 { return 0 }",
    );
    assert!(ll.contains("@main("));
}

// ── Lambda ─────────────────────────────────────────

#[test]
fn lambda_in_function() {
    let ll = compile_success(
        "func main() -> I32 { double = (x: I32) -> x * 2\n return 0 }",
    );
    assert!(ll.contains("@main("));
}

// ── Match ──────────────────────────────────────────

#[test]
fn match_statement_produces_ir() {
    let ll = compile_success(
        "func main() -> I32 { result = match 1 { 1 => \"one\" 2 => \"two\" else => \"other\" }\n return 0 }",
    );
    assert!(ll.contains("@main("));
}

// ── Assert ─────────────────────────────────────────

#[test]
fn assert_produces_ir() {
    let ll = compile_success(
        "func main() -> I32 { assert true\n return 0 }",
    );
    assert!(ll.contains("@main("));
    assert!(ll.contains("@knot_exception"));
}

#[test]
fn extern_func_emits_declare() {
    let ll = compile_success("
        extern func puts(s: Array[Char]) -> I32
        func main() -> I32 {
            return 0
        }
    ");
    assert!(ll.contains("declare i32 @puts(ptr)"), "expected declare for puts, got:\n{}", ll);
}

#[test]
fn extern_class_parsed() {
    let ll = compile_success("
        extern class File { fd: I32 }
        extern func fopen(path: Array[Char], mode: Array[Char]) -> Array[Char]
        func main() -> I32 {
            return 0
        }
    ");
    assert!(ll.contains("declare ptr @fopen(ptr, ptr)"), "expected declare for fopen, got:\n{}", ll);
}

#[test]
fn operator_overload_dispatches() {
    let ll = compile_success(
        "class Box {\n    val: I32\n    func new(v: I32) { this.val = v }\n    operator +(other: Box) -> I32 { return this.val + other.val }\n}\nfunc main() -> I32 {\n    a = Box::new(10)\n    b = Box::new(20)\n    return a + b\n}",
    );
    assert!(ll.contains("@Box__op_plus("), "expected Box__op_plus, got:\n{}", ll);
}
