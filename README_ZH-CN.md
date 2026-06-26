# Knot

> [English](README.md)

**Knot** 是一门极简、显式、零开销的系统编程语言。

## 特性

- **极简语法** — 大括号块，换行即语句结束，无分号
- **编译期 GC** — 所有内存在编译期确定释放点，运行时零停顿
- **零成本抽象** — mixin、wrap、泛型在编译时完全内联
- **动静皆宜** — 默认静态类型推断，可选动态类型 `Any`

## 快速开始

### 安装

```bash
git clone https://github.com/your/knot
cd knot
cargo build --release
```

> 需要 `clang` 在 PATH 中。如安装在其他位置，设 `KNOT_CLANG=/path/to/clang`。

### 编译运行

```bash
knot hello.knot --run
```

### 示例

```knot
func main() -> I32 {
    x = 42
    y = x + 58
    return y   // → 退出码 100
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

## 命令行

```
knot <source.knot> [-o output.exe] [--run]
```

| 参数 | 说明 |
|------|------|
| `<source>` | Knot 源文件 |
| `-o, --output` | 输出可执行文件路径 |
| `--run` | 编译后立即运行 |
| `-h, --help` | 帮助信息 |
| `-V, --version` | 版本信息 |

## 文档

- [语言语法](SYNTAX_ZH-CN.md)
- 设计文档（待补充）

## 项目结构

```
src/
├── main.rs          # CLI 入口
├── lib.rs           # 库根
├── compiler.rs      # 编译管道
├── error.rs         # 错误类型
├── lexer/           # 词法分析
├── parser/          # 语法分析 + AST
│   ├── ast.rs       #   AST 定义
│   └── symbol.rs    #   符号表
├── semantic/        # 语义分析
│   ├── mod.rs       #   语句检查
│   └── check.rs     #   表达式类型检查
├── ir/              # 中间表示
│   ├── tac.rs       #   三地址码
│   └── lower.rs     #   AST → TAC
└── codegen/
    └── llvm.rs      # LLVM 代码生成
```

## 编译管道

```
源文件 → Lexer → Parser → Semantic → IR → LLVM → 可执行文件
```

## 许可

MIT
