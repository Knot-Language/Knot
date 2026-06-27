# Knot

> [中文](README_ZH-CN.md)

**Knot** is a minimalist, explicit, zero-overhead systems programming language.

## Features

- **Minimal Syntax** — Curly-brace blocks, newline as statement terminator, no semicolons
- **Compile-Time GC** — All memory freed at compile-time determined points, zero runtime pauses
- **Zero-Cost Abstractions** — mixin, wrap, generics fully inlined at compile time
- **Static & Dynamic** — Static type inference by default, optional `Any` dynamic type

## Quick Start

### Install

```bash
git clone https://github.com/your/knot
cd knot
cargo build --release
```

> Requires `clang` on PATH. Set `KNOT_CLANG=/path/to/clang` if installed elsewhere.

### Compile & Run

```bash
knot run hello.knot
```

Or build only:

```bash
knot build hello.knot -o hello.exe
```

Create a new project:

```bash
knot new myapp
```

### Examples

```knot
func main() -> I32 {
    x = 42
    y = x + 58
    return y   // → exit code 100
}
```

```knot
class Point {
    x: I32
    func get_x() -> I32 {
        return this.x
    }
}

func main() -> I32 {
    return 42
}
```

## CLI

```
knot new <name>                 Create a new project
knot build <source> [-o <exe>]  Compile a source file
knot run <source> [-o <exe>]    Compile and run a source file
knot tie [package]              (not yet implemented)
knot untie [package]            (not yet implemented)
```

| Option | Description |
|--------|-------------|
| `<source>` | Knot source file (.knot) |
| `-o, --output` | Output executable path (default: `<source>.exe`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Docs

- [Language Syntax](SYNTAX.md)
- Design document (TBD)

## Project Structure

```
src/
├── main.rs          # CLI entry
├── lib.rs           # Library root
├── compiler.rs      # Compilation pipeline
├── error.rs         # Error types
├── lexer/           # Lexical analysis
├── parser/          # Parsing + AST
│   ├── ast.rs       #   AST definitions
│   └── symbol.rs    #   Symbol table
├── semantic/        # Semantic analysis
│   ├── mod.rs       #   Statement checks
│   └── check.rs     #   Expression type checks
├── ir/              # Intermediate Representation
│   ├── tac.rs       #   Three-address code
│   └── lower.rs     #   AST → TAC lowering
└── codegen/
    └── llvm.rs      # LLVM code generation
```

## Pipeline

```
Source → Lexer → Parser → Semantic → IR → LLVM → Executable
```

## License

MIT
