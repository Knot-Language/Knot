# Knot Language Syntax Reference

> [中文](SYNTAX_ZH-CN.md)

## Table of Contents

- [1. Lexical Structure](#1-lexical-structure)
- [2. Comments](#2-comments)
- [3. Keywords](#3-keywords)
- [4. Literals](#4-literals)
- [5. Operators](#5-operators)
- [6. Variables & Types](#6-variables--types)
- [7. Functions](#7-functions)
- [8. Generics](#8-generics)
- [9. Control Flow](#9-control-flow)
- [10. Classes & OOP](#10-classes--oop)
- [11. Enums](#11-enums)
- [12. Exception Handling](#12-exception-handling)
- [13. Type Casting](#13-type-casting)
- [14. Module Imports](#14-module-imports)
- [15. Type System](#15-type-system)

---

## 1. Lexical Structure

### File Encoding

Source files must be UTF-8 encoded, without BOM.

### Whitespace

Spaces and tabs separate tokens and have no semantic meaning. Newlines (`\n`) serve as statement terminators. Newlines are ignored in the following contexts:

- Inside parentheses `()`, brackets `[]`, braces `{}`
- After operators (expressions may span multiple lines)

### Identifiers

Must start with a letter or underscore, followed by letters, digits, or underscores. Case-sensitive.

```
valid_name
_secret
data42
camelCase
PascalCase
```

---

## 2. Comments

Single-line comments start with `//` and extend to the end of the line. Multi-line comments are enclosed in `/* ... */` and may be nested.

```knot
// Single line comment

/*
   Multi line comment
   /* Nested comment */
*/
```

---

## 3. Keywords

The following are reserved keywords and cannot be used as identifiers:

```
abstract  Any      args     as       as!
assert    break    catch    class    continue
delete    else     enum     false    for
func      if       import   in       kwargs
match     mixin    mut      new      null
operator  private  return   static   throw
true      try      while    wrap
```

---

## 4. Literals

### Boolean

`true` and `false`, of type `Bool`.

### Integer

Supports decimal, hex `0x`, binary `0b`, and octal `0o` notation. Underscores may be used as visual separators. A suffix specifies the bit width:

| Suffix | Type |
|--------|------|
| `i8` | Signed 8-bit |
| `i16` | Signed 16-bit |
| `i32` (default) | Signed 32-bit |
| `i64` | Signed 64-bit |
| `u8` | Unsigned 8-bit |
| `u16` | Unsigned 16-bit |
| `u32` | Unsigned 32-bit |
| `u64` | Unsigned 64-bit |

```
42                   // I32
0xFF                 // hex 255
0b1010               // binary 10
0o777                // octal 511
1_000_000            // separator
42i64                // I64
```

### Float

Supports decimal point and scientific notation. Suffix specifies precision:

```
3.14                 // F64 (default)
2.0f32               // F32
1.5e10               // scientific
```

### String

Double-quoted strings `"..."` support escape sequences:

| Escape | Meaning |
|--------|---------|
| `\n` | Newline |
| `\t` | Tab |
| `\r` | Carriage return |
| `\\` | Backslash |
| `\"` | Double quote |
| `\{` | Left brace |

```knot
"Hello, World!"
"line1\nline2"
```

### Raw String

Single-quoted `'...'` or backtick `` `...` `` strings do NOT process escape sequences — ideal for paths and regex:

```knot
'C:\Users\name\file.txt'
`raw \n no escape`
```

### Null

`null` represents the absence of a value, type `Null`. Can be assigned to any `T?` nullable type.

### Array

Square brackets, comma-separated:

```knot
[1, 2, 3]
["a", "b"]
[]                   // empty array
```

### Dict

Curly braces, `key: value` pairs, comma-separated. Keys can be any expression:

```knot
{"name": "Knot", "year": 2026}
{a: 1, b: 2}
{}                   // empty dict
```

---

## 5. Operators

Listed from lowest to highest precedence:

| Prec | Operators | Assoc | Description |
|------|-----------|-------|-------------|
| 10 | `=` `+=` `-=` `*=` `/=` `%=` `&=` `\|=` `^=` | Right | Assignment & compound |
| 20 | `\|\|` | Left | Logical OR |
| 25 | `\|` | Left | Bitwise OR |
| 27 | `^` | Left | Bitwise XOR |
| 29 | `&` | Left | Bitwise AND |
| 30 | `&&` | Left | Logical AND |
| 35 | `<<` `>>` | Left | Shift |
| 40 | `==` `!=` | Left | Equality |
| 50 | `<` `>` `<=` `>=` | Left | Comparison |
| 55 | `..` | Left | Range |
| 60 | `??` | Left | Null-coalesce |
| 65 | `as` `as!` | Left | Type cast |
| 70 | `+` `-` | Left | Add / Subtract |
| 80 | `*` `/` `%` | Left | Multiply / Divide / Modulo |
| 85 | `-x` `!x` `~x` `++x` `--x` | Right(prefix) | Unary |
| 90 | `x.y` `x::y` `x()` `x[]` `x++` `x--` | Left(postfix) | Call / Access |

Additional symbols:

| Symbol | Description |
|--------|-------------|
| `->` | Function return type marker |
| `=>` | Match branch arrow |
| `@` | Wrap invocation |
| `..` | Range operator |

---

## 6. Variables & Types

### Declaration via Assignment

Knot uses **assignment as declaration**. A variable is declared on its first assignment and its type is inferred:

```knot
x = 42                 // inferred as I32
y = 3.14               // inferred as F64
name = "Knot"          // inferred as String
flag = true            // inferred as Bool
```

### Type Annotation

Explicit type annotation overrides inference:

```knot
count: I64 = 0         // explicit I64
pi: F32 = 3.14         // explicit F32
```

### Type Locking

Once a non-`mut` variable's type is determined, subsequent assignments must match:

```knot
a = 10                 // locked to I32
a = 20                 // ✓ valid
// a = "hello"         // ❌ type mismatch, compile error
```

Numeric types have implicit compatibility (I32 can widen to F64, etc.).

### mut Variables

Variables declared with `mut` can hold different types over their lifetime. The old value is automatically freed on reassignment:

```knot
mut x = 42             // currently I32
x = "hello"            // now String, old I32 freed
x = true               // now Bool
```

### Nullable Types

Append `?` to a type name to allow `null`. `T?` values cannot be directly assigned to `T`:

```knot
maybe: I32? = null
maybe = 42

// value: I32 = maybe     // ❌ compile error
value = maybe as! I32     // ✓ forced cast
value = maybe ?? 0        // ✓ null-coalesce
```

---

## 7. Functions

### Definition

`func` defines a function. Parameters go in `()`. Return type goes after `->`. No return type means `Void`:

```knot
func add(a: I32, b: I32) -> I32 {
    return a + b
}

func nothing() {          // equivalent to -> Void
    print("side effect")
}
```

### Invocation

```knot
sum = add(3, 5)
nothing()
```

### Default Parameters

Parameters may specify defaults. Defaulted parameters may be omitted at the call site:

```knot
func greet(name: String = "world", times: I32 = 1) {
    // ...
}

greet()                  // name="world", times=1
greet("Knot")            // name="Knot", times=1
greet("Knot", 3)         // name="Knot", times=3
```

Default parameters must be provided contiguously from right to left.

### Variadic Parameters (args / kwargs)

`args` packs all positional arguments into an array. `kwargs` packs all named arguments into a dict. They are mutually exclusive with regular parameters:

```knot
func sumAll(args items: Array[I32]) -> I32 {
    total = 0
    for i in items { total = total + i }
    return total
}

func config(kwargs opts: Map[String, Any]) {
    // ...
}
```

### Return

`return` ends function execution and provides a value. Bare `return` returns void:

```knot
func early(x: I32) -> I32 {
    if x < 0 {
        return 0
    }
    return x * 2
}
```

### Lambda

`(params) -> expression` or `(params) -> { block }` creates an anonymous function. Captured variables are **copied by value**:

```knot
double = (x: I32) -> x * 2
run = () -> { print("hello") }

result = double(5)       // 10
```

---

## 8. Generics

Generics allow functions, classes, and enums to accept type parameters,
enabling parametric polymorphism.

> Note: generics are fully parsed by the compiler but monomorphization is
> not yet implemented by the backend. Defining generic parameters will
> emit a compiler warning.

### Generic Functions

Type parameters are declared in `[T, U, ...]` after the function name:

```knot
func identity[T](x: T) -> T {
    return x
}

func pair[K, V](key: K, val: V) -> Map[K, V] {
    return {key: val}
}

result = identity(42)      // T inferred as I32
result = identity("hello") // T inferred as String
```

Type arguments are inferred automatically from the call-site argument types.

### Generic Classes

Classes may declare type parameters after the class name, usable in fields
and methods:

```knot
class Box[T] {
    value: T

    func new(v: T) {
        this.value = v
    }

    func get() -> T {
        return this.value
    }
}

intBox = Box::new(42)
strBox = Box::new("hello")
```

### Generic Enums

Enums can also carry type parameters:

```knot
enum Option[T] {
    Some
    None
}
```

### Generic Methods

Methods within a class may independently declare type parameters:

```knot
class Util {
    static func map[T, U](arr: Array[T], f: (T) -> U) -> Array[U] {
        // ...
    }
}
```

---

## 9. Control Flow

### if / else

Any non-zero expression is truthy. `if`-`else` chains:

```knot
if x > 0 {
    print("positive")
} else if x == 0 {
    print("zero")
} else {
    print("negative")
}
```

`if` can be used as an expression. An `else` branch is required in expression position:

```knot
abs = if x >= 0 { x } else { -x }
```

### while

Pre-condition loop:

```knot
while x > 0 {
    x = x - 1
}
```

### for-in

Iterates over ranges, arrays, or dicts:

```knot
for i in 0..10 {         // 0 through 9
    print(i)
}

for item in [1, 2, 3] {
    print(item)
}
```

### match

Multi-branch pattern matching. Statement form requires blocks. Expression form accepts values:

```knot
// Statement form
match x {
    1 => { print("one") }
    2 => { print("two") }
    else => { print("other") }
}

// Expression form
label = match x {
    1 => "one"
    2 => "two"
    else => "other"
}
```

The `else` branch is the default. There is no fall-through.

### break / continue

`break` exits the nearest loop. `continue` skips to the next iteration. An optional integer specifies the number of loop levels to break out of:

```knot
while true {
    while true {
        if done {
            break 2      // break two levels
        }
        if skip {
            continue     // skip one iteration
        }
    }
}
```

### assert

Assertions are enforced in **all compilation modes** (no debug/release distinction). Failing an assert throws a runtime error:

```knot
assert x > 0
assert x > 0, "x must be positive"
```

---

## 10. Classes & OOP

### Class Definition

`class` defines a class. The body may contain fields, methods, static methods, operator overloads, wraps, and mixins:

```knot
class Point {
    x: I32               // field
    y: I32 = 0           // field with default
}
```

### Instantiation

Use `::new` to invoke the constructor:

```knot
p = Point::new(10, 20)
```

### Constructor (new)

`func new` is the constructor. It implicitly returns the class type. Do NOT use `static` or a return type annotation — the compiler will warn:

```knot
class Point {
    x: I32

    func new(x: I32) {
        this.x = x
    }
}
```

### Destructor (delete)

`func delete` is the destructor, called automatically when the object is freed. No parameters, no return type:

```knot
class Resource {
    func delete() {
        cleanup()
    }
}
```

### Instance Methods & `this`

Inside an instance method, `this` refers to the current object. Access fields via `this.field`:

```knot
class Point {
    x: I32
    func get_x() -> I32 {
        return this.x
    }
}
```

### Static Methods

`static func` defines a class-level method. Call with `ClassName::method()`:

```knot
class Math {
    static func abs(x: I32) -> I32 {
        return if x >= 0 { x } else { -x }
    }
}

result = Math::abs(-5)   // 5
```

### mixin

`mixin` copies all fields and methods from a source class into the target class. Declared inside the class body:

```knot
class A {
    func a() { print("a") }
    func c() { print("ac") }
}

class B {
    func b() { print("b") }
    func c() { print("bc") }
}

class C {
    mixin A
    mixin B

    func c() {            // Must override: both A and B have c()
        A.c()             // Disambiguate by class name
        B.c()
    }
}
```

When multiple mixins provide methods with the same signature, the target class must explicitly override. Use `ClassName.method()` to disambiguate. The target class's own methods take highest priority.

### Abstract Classes

`abstract class` cannot be instantiated; it can only be used as a mixin source:

```knot
abstract class Printable {
    abstract func format() -> String   // No body
    func print() { }
}
```

Abstract methods use `abstract func` without a body. Subclasses must implement them.

### Access Control

`private` restricts a member to within the current class:

```knot
class Secret {
    private code: I32

    private func encrypt() {
        // ...
    }
}
```

### Operator Overloading

`operator` inside a class defines custom behavior for operators. Supports `+` `-` `*` `/` `%` `==` `!=` `<` `>` etc.:

```knot
class Vector {
    x: I32
    y: I32

    operator +(other: Vector) -> Vector {
        return Vector::new(x + other.x, y + other.y)
    }
}
```

### wrap

`wrap` defines a function that can **only be invoked via `@` syntax** — it cannot be called directly. `@` applies to the declaration on the next line; the wrapped entity is automatically injected as the first parameter. Extra arguments are passed via `@name(...)`.

**Top-level wrap**

```knot
wrap wrapperWithArgument(funcIn, argument) {
    print(funcIn(argument))
    return funcIn
}

@wrapperWithArgument(111)
func test(number) -> I32 {
    return number + 42
}
// Output: 153 (111 + 42)
// Desugars to: test = wrapperWithArgument(test, 111)
```

Without extra args:

```knot
wrap debug(input) {
    print("debugging...")
    return input
}

@debug
func hello() { return "world" }
// Desugars to: hello = debug(hello)
```

**Class-level wrap** — first param is `this`, second is the wrapped entity:

```knot
class Range {
    left: I32
    right: I32

    wrap binarySearch(check) {
        // this = Range object
        // check = function below @
    }
}

@range.binarySearch
func isValid(mid: I32) -> Bool {
    return mid * mid <= 100
}
// Desugars to: range.binarySearch(range, isValid)
```

**Key rules**:

- `wrap` cannot be called explicitly — only via `@`
- `@name` decorates the next declaration, injecting it as the first parameter
- `@name(args)` passes extra args to subsequent wrap parameters
- In a class wrap, the first param is `this`, the second is the wrapped entity
- Compile-time inlined — zero overhead
```

---

## 11. Enums

`enum` defines an enumeration type. Variants are separated by newlines, not commas:

```knot
enum Color {
    Red
    Green
    Blue
}

c = Color.Red
```

---

## 12. Exception Handling

### throw

`throw` raises any value as an exception:

```knot
throw "error message"
throw 404
```

### try / catch

`try`-`catch` handles exceptions. `catch` may specify a capture variable,
with optional type filtering:

```knot
try {
    risky()
} catch e {
    print(e)
}
```

Multiple catch branches filter by type:

```knot
try {
    mightFail()
} catch e: String {
    print("string error: " + e)
} catch e: I32 {
    print("error code: " + e)
}
```

`throw` stores the exception value in a global register and jumps to the
nearest catch handler. Single-level try/catch only; nesting is not yet
supported.

---

## 13. Type Casting

### as (Safe Cast)

Returns `null` if the cast fails:

```knot
x = value as I32
```

### as! (Forced Cast)

Throws if the cast fails:

```knot
x = value as! I32
```

### Null-Coalesce

`??` returns the right-hand side if the left is `null`:

```knot
x = maybe ?? 0
```

---

## 14. Module Imports

### import

`import` loads other files. Supports string paths and module paths:

```knot
import "std/io.knot" as io
import "utils.knot"
```

### Project Configuration

`Knot.toml` at the project root declares metadata and dependencies:

```toml
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.knot"

[dependencies]
```

### Entry Point

An executable must contain `func main() -> I32`. The exit code is specified by the return value:

```knot
func main() -> I32 {
    return 0
}
```

---

## 15. Type System

### Primitive Types

| Category | Types | Notes |
|----------|-------|-------|
| Signed int | `I8` `I16` `I32` `I64` | Default integer literal → I32 |
| Unsigned int | `U8` `U16` `U32` `U64` | |
| Float | `F32` `F64` | Default float literal → F64 |
| String | `String` | Immutable UTF-8 |
| Boolean | `Bool` | `true` / `false` |
| Null | `Null` | Sole value `null` |
| Void | `Void` | No return value |
| Dynamic | `Any` | Accepts any type |

### Compound Types

| Type | Syntax | Notes |
|------|--------|-------|
| Nullable | `T?` | May be `null` |
| Array | `Array[T]` | Dynamic length |
| Dict | `Map[K, V]` | Key-value pairs |
| Class | `ClassName` | User-defined class |

### Type Inference Rules

- `42` → `I32`
- `3.14` → `F64`
- `"hello"` → `String`
- `true` / `false` → `Bool`
- `null` → `Null`
- Array `[1, 2, 3]` → `Array[I32]`
- Binary operation: result type is the wider of the two operands

### Any Dynamic Type

The `Any` keyword represents a dynamic type that can hold any value and is
compatible with all types:

```knot
mut x: Any = 42
x = "hello"
x = true

func process(data: Any) {
    // data can be anything
}
```

Values of type `Any` should typically be cast to a concrete type via `as`
or `as!` before use. `Any` is a reserved keyword.
