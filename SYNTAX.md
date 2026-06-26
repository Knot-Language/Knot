# Knot Language Syntax

> [中文](SYNTAX_ZH-CN.md)

## Lexical

### Keywords

```
abstract  args     as       as!      assert
break     catch    class    continue delete
else      enum     false    for      func
if        import   in       kwargs   match
mixin     mut      new      null     operator
private   return   static   throw    true
try       while    wrap
```

### Literals

| Type | Format | Example |
|------|--------|---------|
| Boolean | `true` / `false` | `true` |
| Integer | Decimal / hex `0x` / binary `0b` / octal `0o`; suffix `i8`-`i64` `u8`-`u64` | `42`, `0xFF`, `0b1010`, `42i64` |
| Float | Suffix `f32` `f64` | `3.14`, `2.0f32` |
| String | Double-quoted `"..."`, supports interpolation `"Hello, {name}"` and escapes `\n` `\t` `\{` | `"hello world"` |
| Raw string | Single-quoted `'...'` or backtick `` `...` ``, no escapes | `'C:\path\to\file'` |
| Null | `null` | |
| Array | `[expr, ...]` | `[1, 2, 3]` |
| Dict | `{key: value, ...}` | `{"a": 1, b: 2}` |

### Operators

```
+  -  *  /  %  Arithmetic
<< >>         Shift
&  |  ^  ~    Bitwise
== != < > <= >= Comparison
&& || !       Logical
=             Assignment
..            Range
??            Null-coalesce
as as!        Type cast
.             Member access
::            Static access
@             Wrap invocation
() [] {}      Brackets
-> =>         Arrows
```

## Syntax

### Variables

```knot
x = 42           // Assignment = declaration, type inferred as I32
y: F64 = 3.14    // Explicit type annotation

// mut variables accept any type, type locks on first assignment
mut z = 42       // z is mut, type locked to I32
z = 43           // ✓ same type
// z = "hello"   // ❌ type already locked as I32

// Non-mut variables: type locked on first inference
a = 10           // inferred as I32
// a = "hi"      // ❌ type mismatch
```

### Functions

```knot
func add(a: I32, b: I32) -> I32 {
    return a + b
}

// Default parameter
func greet(name: String = "world") {
    print("Hello, {name}")
}

// Void return
func print(msg: String) {
    // ...
}

// Pack all positional args
func sumAll(args items: Array[I32]) -> I32 {
    total = 0
    for i in items {
        total = total + i
    }
    return total
}

// Pack all named args
func configure(kwargs opts: Map[String, Any]) {
    // ...
}

// Pack both
func handle(args items: Array[Any], kwargs opts: Map[String, Any]) {
    // ...
}
```

### Lambda

```knot
add = (a: I32, b: I32) -> a + b
run = () -> { print("hello") }
```

### Control Flow

```knot
// if / else
if x > 0 {
    return 1
} else if x == 0 {
    return 0
} else {
    return -1
}

// while
while x > 0 {
    x = x - 1
}

// for-in
for i in 0..10 {
    print(i)
}

// break 2 / continue 2 — jump multiple loop levels
while true {
    while true {
        if cond {
            break 2     // break out of two loops
        }
    }
}

// assert
assert x > 0
assert x > 0, "x must be positive"
```

### Match

```knot
// match as statement — branches must be blocks
match expr {
    1 => { print("one") }
    2 => { print("two") }
    else => { print("other") }
}

// match as expression — branches are values
result = match x {
    1 => "one"
    2 => "two"
    else => "other"
}
```

### Exceptions

```knot
try {
    risky_operation()
} catch e {
    print("error")
}

throw "something went wrong"
```

### Type Cast

```knot
x = value as I32       // Safe cast, returns null on failure
x = value as! I32      // Forced cast, throws on failure
```

### Classes

```knot
abstract class Printable {
    func print() { }
    abstract func format() -> String   // No body, subclasses must override
}

class A {
    func a() { print("a") }
    func c() { print("ac") }
}

class B {
    func b() { print("b") }
    func c() { print("bc") }
}

// mixin declared inside class body
class C {
    mixin A
    mixin B

    // A and B both have c(), must override
    func c() {
        A.c()       // Disambiguate by class name, calls A's c
        B.c()       // Calls B's c
    }
}

class Point {
    x: I32
    y: I32 = 0

    func new(x: I32, y: I32 = 0) {
        this.x = x
        this.y = y
    }

    func delete() { }

    func dist() -> F64 { return 0.0 }

    operator +(other: Point) -> Point {
        return Point::new(x + other.x, y + other.y)
    }

    // wrap: first param is the input value (name is irrelevant), at least 1 param required
    //       subsequent params are passed at @ call site, params may have defaults
    wrap into(target: I32 = 0) {
        x = target
        return this
    }
}

// Construction
p = Point::new(1, 2)

// Wrap invocation: @object.wrap_name(extra_args)
@p.into(10)

// Instance method call
p.dist()
```

### Enums

```knot
enum Color {
    Red
    Green
    Blue
}
```

### Imports

```knot
import "std.io" as io
import "utils.knot"
```

## Types

| Signed Int | Unsigned Int | Float | Other |
|-----------|-------------|-------|-------|
| `I8` `I16` `I32` `I64` | `U8` `U16` `U32` `U64` | `F32` `F64` | `String` `Bool` `Void` `Null` `Any` |

- Nullable: `I32?`
- Array: `Array[I32]`
- Dict: `Map[String, I32]`
- Dynamic: `Any` accepts any type

## Comments

```knot
// Single line
/* Multi
   line */
```
