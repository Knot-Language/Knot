# Knot Tutorial: Learn Systems Programming, The Fun Way

> [中文](TUTORIAL_ZH-CN.md)

This guide will get you writing real Knot programs fast. Knot's syntax is
deliberately minimal — if you've used JavaScript, Python, or C, you'll feel
right at home. No jargon dumps, no walls of theory. Just code that runs.

---

## Table of Contents

1. [Hello, Knot!](#1-hello-knot)
2. [Variables and Basic Types](#2-variables-and-basic-types)
3. [Functions](#3-functions)
4. [Control Flow](#4-control-flow)
5. [Arrays and Dictionaries](#5-arrays-and-dictionaries)
6. [Classes and Objects](#6-classes-and-objects)
7. [Enums](#7-enums)
8. [Error Handling](#8-error-handling)
9. [Generics](#9-generics)
10. [Mixins](#10-mixins)
11. [Wraps and Decorators](#11-wraps-and-decorators)
12. [Modules and Imports](#12-modules-and-imports)
13. [Extern — Call C from Knot](#13-extern--call-c-from-knot)
14. [Putting It All Together](#14-putting-it-all-together)
15. [Next Steps](#15-next-steps)

---

## 1. Hello, Knot!

### Get Set Up

```bash
git clone https://github.com/your/knot
cd knot
cargo build --release
```

> You need `clang` on your system. If it's not on PATH, set
> `KNOT_CLANG=/path/to/clang`.

### Three Lines to Running Code

Make a file called `hello.knot`:

```knot
import "std/io.knot" as io

func main() -> I32 {
    io.print_str("Hello, Knot!")
    return 0
}
```

Then:

```bash
set KNOT_STD=D:\path\to\knot\std     # Windows — set this first
knot run hello.knot
```

If you see `Hello, Knot!` on screen, you're in business.

### What That Code Actually Does

- `import "std/io.knot" as io` — brings in the standard library I/O module
- `func main() -> I32` — "Hey compiler, here's my entry point. When we're done,
  give the OS a 32-bit integer."
- `io.print_str(...)` — prints a string to the terminal.
- `return 0` — tells the OS "all good" (0 = success, anything else = error).

> **About `print()`**: Knot has no built-in `print`. Throughout this tutorial,
> `print(x)` is shorthand for `io.print_str(x)`. You need to
> `import "std/io.knot" as io` and set the `KNOT_STD` environment variable first.
> See [Modules and Imports](#12-modules-and-imports) for details.

Notice there are **no semicolons**. A newline means "statement done" — like
Python, but with curly braces. Clean and clear.

---

## 2. Variables and Basic Types

### Assign = Declare

In Knot you don't type `let`, `var`, or `const`. The first time you assign a
value to a name, that's a variable declaration. The compiler figures out the
type for you:

```knot
age = 25           // whole number → I32
price = 9.99       // has a decimal → F64
name = "Alice"     // in quotes → String
ok = true          // true/false → Bool
nothing = null     // null → Null
```

Think of it like labelled boxes. You drop a value in, the compiler peeks at it
and labels the box with the right type.

### Want to Be Explicit? Sure.

```knot
count: I64 = 0       // "I want a 64-bit integer, please"
ratio: F32 = 3.14    // "32-bit float is enough for me"
```

### Once a Variable Has a Type, That's It

```knot
a = 10      // a is now I32, locked and loaded
a = 20      // still I32, still fine
// a = "hi" // NOPE — compiler stops you right here
```

This catches a whole category of bugs before they ever run. You never
accidentally put a string where a number belongs.

### Making Room for "Nothing"

Sometimes a value might be missing. Append `?` to the type:

```knot
maybe: I32? = null     // "I might have a number, or I might not"

maybe = 42             // now it does

// x: I32 = maybe      // ❌ compiler says "what if it's null?"
x = maybe ?? 0         // ✅ "if null, use 0 instead"
x = maybe as! I32      // ✅ "I swear it's not null — crash if I'm wrong"
```

### Primitive Types Cheat Sheet

| What you mean | Type | How to write it |
|---------------|------|-----------------|
| Whole number | `I8` `I16` `I32` `I64` | `42` defaults to `I32` |
| Unsigned int | `U8` `U16` `U32` `U64` | `42u32` (add suffix) |
| Decimal | `F32` `F64` | `3.14` defaults to `F64` |
| Text | `Array[Char]` (a.k.a. `String`) | `"hello"` |
| Single character | `Char` | `'a'` |
| Yes/No | `Bool` | `true` / `false` |
| Nothing | `Null` | `null` |


### Number Tricks

```knot
42              // plain decimal
0xFF            // hex = 255 (prefix 0x)
0b1010          // binary = 10 (prefix 0b)
0o777           // octal = 511 (prefix 0o)
1_000_000       // one million (underscores are just for your eyes)
42i64           // explicitly 64-bit
3.14            // 64-bit float by default
2.0f32          // explicitly 32-bit
1.5e10          // scientific notation = 15 billion
```

### Character Literals

A single character in single quotes gets type `Char` (one byte). Escape sequences
work here too:

```knot
'A'
'\n'        // newline character
'0'         // the digit zero
```

### Strings: Escaped vs. Raw

Double-quoted strings produce escaped text — `\n` becomes a newline, `\t` a tab.
Under the hood, a string is `Array[Char]` (yes, you can index it like an array):

```knot
"Hello\nWorld"     // prints on two lines
```

**Backtick strings** (`` `...` ``) keep everything literal — no escape processing.
Perfect for Windows paths and regex:

```knot
`C:\Users\Alice\file.txt`
`raw \n here`
```

**Single quotes** are context-sensitive: one character → `Char` literal;
multiple characters → raw string (same as backticks):

```knot
'A'                          // Char
'C:\Users\name\file.txt'     // raw string (multi-char → no escape)
```

---

## 3. Functions

### Define One

```knot
func add(a: I32, b: I32) -> I32 {
    return a + b
}

func sayHi(name: String) {
    print("Hi, " + name + "!")
}
```

Pattern: `func` keyword → name → (params) → `->` return type → { body }.
Skip the `->` if the function doesn't return anything.

### Call One

```knot
result = add(3, 5)   // 8
sayHi("Bob")         // prints "Hi, Bob!"
```

### Default Values for Parameters

Make a parameter optional by giving it a fallback:

```knot
func greet(name: String = "world", times: I32 = 1) {
    for i in 0..times {
        print("Hello, " + name + "!")
    }
}

greet()                // uses defaults: prints once with "world"
greet("Knot")          // "world" → "Knot", still prints once
greet("Knot", 3)       // prints three times
```

Rule: parameters with defaults go on the **right**. You can't have a default
parameter left of a non-default one.

### Variable Number of Arguments? Use args

```knot
func sumAll(args items: Array[I32]) -> I32 {
    total = 0
    for i in items {
        total = total + i
    }
    return total
}

sumAll(1, 2, 3, 4, 5)   // 15 — pass as many as you want
```

`args` bundles every argument you pass into a single array.

### Lambdas: Quick, Throwaway Functions

```knot
double = (x: I32) -> x * 2        // in, out
runner = () -> { print("done") } // no params, runs a block

double(5)   // 10
```

Lambdas capture outside variables **by value** — they get a snapshot at creation
time, so changing the original later won't affect the copy inside.

### return: Take the Result and Leave

```knot
func abs(x: I32) -> I32 {
    if x >= 0 {
        return x    // already positive, done
    }
    return -x        // flip it and done
}
```

A bare `return;` (no value) works in `Void` functions when you want to bail out
early.

---

## 4. Control Flow — Making Decisions

### if/else

```knot
if x > 0 {
    print("positive")
} else if x == 0 {
    print("zero")
} else {
    print("negative")
}
```

Any non-zero value is "truthy". You can just write `if flag { ... }` instead of
`if flag == true { ... }`.

`if` is also an **expression** — it produces a value:

```knot
label = if score >= 60 { "pass" } else { "fail" }
```

When used as an expression, `else` is mandatory — otherwise what would the value
be?

### while

```knot
x = 10
while x > 0 {
    print(x)
    x = x - 1
}
// counts down 10, 9, 8, ... 1
```

### for-in: The Workhorse Loop

Works with ranges, arrays, and dicts:

```knot
// 0 through 9 (0..10 excludes 10)
for i in 0..10 {
    print(i)
}

// over an array
for name in ["Alice", "Bob", "Charlie"] {
    print(name)
}

// over a dict — get both key and value
scores = {"math": 90, "english": 85}
for subject, score in scores {
    print(subject + ": " + score)
}
```

### match: Multi-Way Branching, Clean

```knot
// Statement form — each branch runs code
match x {
    1 => { print("one") }
    2 => { print("two") }
    else => { print("something else") }
}

// Expression form — each branch returns a value
chinese = match x {
    1 => "一"
    2 => "二"
    else => "其他"
}
```

The `else` branch catches everything else. No fall-through — no `break` needed.

### break and continue

```knot
while true {
    while true {
        if done {
            break 2      // escapes two levels of loops
        }
        if skip {
            continue     // skip to next iteration of inner loop
        }
    }
}
```

`break 2` means "break out of 2 nested loops". Plain `break` just exits the
nearest one.

### assert: "This Must Be True"

```knot
assert x > 0
assert x > 0, "x must be positive!"
```

Assertions run in **all builds** — there's no debug/release distinction. They're
your sanity checks. If one fails, the program stops immediately with a clear
message. Great for catching logic errors early.

---

## 5. Arrays and Dictionaries

### Arrays: Ordered List of Same-Type Things

```knot
nums = [1, 2, 3, 4, 5]
names = ["Alice", "Bob", "Charlie"]
empty = []

first = nums[0]          // 1
nums[0] = 99             // now [99, 2, 3, 4, 5]
count = nums.length      // 5
```

`[1, 2, 3]` has type `Array[I32]`. Every element must match.

### Dictionaries: Key → Value Lookup

```knot
config = {"host": "localhost", "port": 8080}
caps = {a: 1, b: 2}     // bare keys work too

port = config["port"]    // 8080
config["debug"] = true   // add or update
```

`{"a": 1}` has type `Map[String, I32]`.

### The Range Operator `..`

```knot
for i in 0..5 {   // 0, 1, 2, 3, 4 (stops before 5)
    print(i)
}
```

`a..b` is a range from a (inclusive) to b (exclusive).

---

## 6. Classes and Objects

### Bundle Data and Behaviour

```knot
class Point {
    x: I32
    y: I32

    // constructor — called when you write Point::new(3, 4)
    func new(x: I32, y: I32) {
        this.x = x
        this.y = y
    }

    func distance() -> F64 {
        return sqrt(this.x * this.x + this.y * this.y)
    }
}
```

The bits to remember:
- `x: I32` and `y: I32` are fields — data stored on each instance
- `func new(...)` is the constructor. Don't mark it `static` and don't add a
  return type — the compiler handles that.
- `this` means "this specific instance". Use `this.xxx` to reach fields and
  methods.

### Create and Use Objects

```knot
p = Point::new(3, 4)
d = p.distance()    // 5.0 — 3-4-5 triangle!
```

### Static Methods: Belong to the Class, Not an Instance

```knot
class Math {
    static func abs(x: I32) -> I32 {
        return if x >= 0 { x } else { -x }
    }
}

result = Math::abs(-5)   // 5
```

No object needed to call a static method — perfect for utility functions.

### Destructors: Clean Up When Done

```knot
class Resource {
    handle: I32

    func delete() {
        close(this.handle)    // compiler calls this automatically
    }
}
```

`func delete()` is the destructor. No args, no return type. The compiler
frees the object and calls `delete()` at a compile-time-determined point.

### private: Keep Your Internals to Yourself

```knot
class Secret {
    private code: I32

    private func encrypt() -> I32 {
        return this.code ^ 0xFF
    }

    func getCode() -> I32 {
        return this.encrypt()   // public method can call private ones
    }
}
```

`private` members are only visible inside the class. The outside world sees
nothing.

### Operator Overloading: Make `+`, `==`, and `<<` Work on Your Types

```knot
class Vector {
    x: I32
    y: I32

    func new(x: I32, y: I32) {
        this.x = x
        this.y = y
    }

    operator +(other: Vector) -> Vector {
        return Vector::new(this.x + other.x, this.y + other.y)
    }

    operator ==(other: Vector) -> Bool {
        return this.x == other.x && this.y == other.y
    }
}

v1 = Vector::new(1, 2)
v2 = Vector::new(3, 4)
v3 = v1 + v2          // Vector { x: 4, y: 6 } — actually calls Vector__op_plus
areSame = v1 == v2    // false

// << is great for I/O-style chaining:
class Cout {
    operator <<(s: I8*) -> Cout {
        // built-in C printf under the hood
        return this           // return self to chain: cout << "a" << "b"
    }
}
```

You can overload `+` `-` `*` `/` `%` `==` `!=` `<` `>` `<=` `>=` `<<` `>>` `&` `|` `^`.
Each operator can only have one overload per class (no type-based overloading).

---

## 7. Enums

An enum is "pick exactly one from this list":

```knot
enum Color {
    Red
    Green
    Blue
}

c = Color.Red
```

Variants are **newline-separated** (no commas). They can also be generic:

```knot
enum Option[T] {
    Some
    None
}
```

---

## 8. Error Handling

### throw: Something Went Wrong, Bail Out

```knot
throw "file not found"
throw 404
throw MyError::new("connection dropped")
```

You can throw literally any value — strings, numbers, objects, whatever makes
sense.

### try/catch: Handle the Chaos Gracefully

```knot
try {
    mightFail()
} catch e {
    print("caught: " + e)
}
```

Filter catches by type:

```knot
try {
    riskyOperation()
} catch e: String {
    print("string error: " + e)
} catch e: I32 {
    print("error code: " + e)
}
```

When `throw` fires, execution jumps to the nearest matching `catch` block.
Everything in the `try` after the throw point is skipped.

---

## 9. Generics — Write Once, Use with Any Type

Generics let you say "T can be any type, we'll figure it out later". The
compiler generates the actual code for each concrete type at compile time —
zero runtime cost.

### Generic Functions

```knot
func identity[T](x: T) -> T {
    return x              // whatever T is, just hand it back
}

func pair[K, V](key: K, val: V) -> Map[K, V] {
    return {key: val}
}

a = identity(42)          // compiler infers T = I32
b = identity("hello")     // compiler infers T = String
c = pair("score", 100)    // K = String, V = I32
```

The `[T]` is a placeholder. You never explicitly write `identity[I32](42)` —
the compiler figures it out from the argument you pass.

### Generic Classes

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

intBox = Box::new(42)         // Box[I32]
strBox = Box::new("hello")    // Box[String]
```

One `Box` class, works for any type. No code duplication.

### Generic Methods

Methods can have their own type parameters, separate from the class:

```knot
class Util {
    static func map[T, U](arr: Array[T], f: (T) -> U) -> Array[U] {
        result: Array[U] = []
        for item in arr {
            result.push(f(item))
        }
        return result
    }
}

doubled = Util::map([1, 2, 3], (x: I32) -> x * 2)   // [2, 4, 6]
```

---

## 10. Mixins — Share Code Without Inheritance Hell

`mixin` copies all fields and methods from one class into another. Think of it
as "paste everything from this class into that class," compile-time:

```knot
class Logger {
    func log(msg: String) {
        print("[LOG] " + msg)
    }
}

class App {
    mixin Logger       // now App has everything Logger has

    func run() {
        this.log("app started")   // straight from Logger
    }
}
```

### When Two Mixins Clash

If two mixins bring methods with the same name, you **must override** in the
target class and specify which one you mean:

```knot
class A {
    func action() { print("A") }
    func shared() { print("A.shared") }
}

class B {
    func action() { print("B") }
    func shared() { print("B.shared") }
}

class C {
    mixin A
    mixin B

    func shared() {           // resolve the conflict
        A.shared()
        B.shared()
    }
}
```

Without the override, the compiler says "I don't know which one you want" and
refuses to build.

### Abstract Classes: Blueprints Only

`abstract class` can't be instantiated. It's meant to be mixed in. Abstract
methods have no body — the mixing class must provide the implementation:

```knot
abstract class Printable {
    abstract func format() -> String   // declaration only, no body

    func print() {
        println(this.format())
    }
}

class Report {
    mixin Printable

    func format() -> String {          // must implement this
        return "Report contents..."
    }
}
```

Priority order: your own methods > mixin methods > mixin's mixin methods.

---

## 11. Wraps and @ Decorators

A `wrap` is a function you can **only call with `@`**. `@` goes before a
declaration and passes that declaration as the wrap's first argument.

### Simplest Wrap

```knot
wrap debug(input) {
    print("debug: entering function")
    return input
}

@debug
func hello() {
    return "world"
}
// Behind the scenes: hello = debug(hello)
// The original hello is wrapped by debug, but keeps the same name
```

### Wraps with Extra Arguments

```knot
wrap withArg(target, value) {
    print("wrapping with " + value)
    return target
}

@withArg("magic")
func worker() {
    // ...
}
// Behind the scenes: worker = withArg(worker, "magic")
```

### Wraps Inside a Class

When a wrap lives in a class, the first parameter is `this`, the second is the
wrapped entity:

```knot
class Range {
    left: I32
    right: I32

    wrap binarySearch(check) {
        while this.left < this.right {
            mid = (this.left + this.right) / 2
            if check(mid) {
                this.left = mid + 1
            } else {
                this.right = mid
            }
        }
        return this.left
    }
}

range = Range::new(0, 100)

@range.binarySearch
func isValid(mid: I32) -> Bool {
    return mid * mid <= 100
}
// Behind the scenes: range.binarySearch(range, isValid)
```

### The Rules in Plain English

- You cannot call a `wrap` directly — it only works through `@`.
- `@name` decorates the next line, which becomes the first argument.
- `@name(extra)` passes extra arguments to the wrap.
- Everything is inlined at compile time — zero overhead.

---

## 12. Modules and Imports

### Pull in Other Files

```knot
import "utils.knot"           // file path, relative to this file
import "std/io.knot" as io    // with a namespace alias
import math                   // module name, resolved by compiler
```

### Importing the Standard Library

Knot ships with a standard library `std/`. Point `KNOT_STD` to it first:

```bash
set KNOT_STD=D:\dev\knot\std        # Windows
export KNOT_STD=/path/to/knot/std   # Linux/macOS
```

Then pull in the I/O module:

```knot
import "std/io.knot" as io

func main() -> I32 {
    cout = io.Cout::new()
    cout << "Hello " << "World\n"   // stream-style output (operator <<)
    io.write_i32(42)                // output an integer
    name = io.read_line()           // read a line from stdin
    return 0
}
```

### Alias vs. No Alias

No alias: everything from the imported file lands directly in your scope,
as if you'd written it right here:

```knot
import "utils.knot"

result = compute(42)   // compute was defined in utils.knot
```

With alias: access through the namespace:

```knot
import "utils.knot" as utils

result = utils.compute(42)
```

### Project Config

A `Knot.toml` at your project root:

```toml
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.knot"

[dependencies]
```

### Scaffold a New Project

```bash
knot new myapp
```

Boom — you get `Knot.toml` and `src/main.knot` ready to go.

### Safety Note

Imports happen at compile time. Path traversal (`..`) is **blocked** — you
can't sneak around the file system. The compiler reads and inlines everything
in one shot.

---

## 13. Extern — Call C from Knot

Knot is a systems language, so talking to C is a first-class feature. You
declare the signature in Knot, write the implementation in a `.c` file — the
compiler links them together automatically.

### Extern Functions

```knot
extern func puts(s: I8*) -> I32
extern func strlen(s: I8*) -> I64
extern func sin(x: F64) -> F64
```

- `extern func` declares the signature only — **no body**.
- The C side implements it in a file with the **same name** (e.g. `utils.c` for
  `utils.knot`).
- The compiler auto-detects and compiles the `.c` file, links everything at the
  end.

### Extern Classes

You can declare C structs as extern classes. Knot only uses them through
pointers — it never creates them directly:

```knot
extern class File {
    fd: I32
}

extern func fopen(path: I8*, mode: I8*) -> File
extern func fclose(f: File) -> I32
```

C side (`knot.h`):

```c
#include "knot.h"

typedef struct { knot_i32 fd; } File;
```

### Type Mapping (knot.h)

The compiler ships a `knot.h` header. Include it in your C files and use these
mappings:

| Knot | C |
|------|---|
| `I8`–`I64` | `int8_t`–`int64_t` |
| `U8`–`U64` | `uint8_t`–`uint64_t` |
| `F32` / `F64` | `float` / `double` |
| `Char` | `char` |
| `Bool` | `int32_t` |
| `I8*` / `T*` | `const char*` / pointer |
| `Array[Char]` | `const char*` |
| Extern class | pointer to struct |

### Quick Example

`utils.knot`:

```knot
extern func strlen(s: I8*) -> I64
```

`utils.c`:

```c
#include "knot.h"
#include <string.h>

knot_i64 strlen(const char* s) {
    return (knot_i64)strlen(s);
}
```

Just drop `utils.c` next to `utils.knot`. The compiler handles the rest.

---

## 14. Putting It All Together

Here's a real example using classes, generics, mixins, error handling, and
decorators — all in one file:

```knot
// --- Logger mixin: anything that mixes this gets log() for free ---
abstract class Logger {
    abstract func prefix() -> String

    func log(msg: String) {
        print(this.prefix() + msg)
    }
}

// --- Generic data store: holds items of any type ---
class Repository[T] {
    mixin Logger

    private items: Array[T]

    func new() {
        this.items = []
    }

    func prefix() -> String {
        return "[Repo] "
    }

    func add(item: T) {
        this.items.push(item)
        this.log("added: " + item)
    }

    func get(index: I32) -> T? {
        if index < 0 || index >= items.length {
            return null
        }
        return this.items[index]
    }

    func count() -> I32 {
        return this.items.length
    }
}

// --- timed wrap: automatically measures how long a function takes ---
wrap timed(target) {
    start = now()
    result = target()
    elapsed = now() - start
    print("took " + elapsed + "ms")
    return result
}

// --- Data processing function, automatically timed ---
@timed
func processData() -> I32 {
    repo = Repository::new()
    repo.add("alpha")
    repo.add("beta")

    total = 0
    for i in 0..repo.count() {
        item = repo.get(i) ?? "default"
        total = total + item.length
    }
    return total
}

// --- Entry point ---
func main() -> I32 {
    result = processData()
    assert result == 9, "unexpected result: " + result
    return 0
}
```

### What This Example Uses

| Feature | Where |
|---------|-------|
| Classes | `class Repository[T]` |
| Generics | `Repository[T]`, `Array[T]` |
| Mixin | `mixin Logger` |
| Abstract class | `abstract class Logger` |
| Fields | `private items: Array[T]` |
| Constructor | `func new()` |
| Private | `private items` |
| Nullable | `T?` |
| Null-coalesce | `??` |
| For-in | `for i in 0..repo.count()` |
| Wrap | `wrap timed(target)` |
| `@` | `@timed` |
| Assert | `assert result == 9, "..."` |
| Entry point | `func main() -> I32` |

---

## 15. Where to Go from Here

### Keep Reading

- **[Language Syntax Reference](SYNTAX.md)** — the full reference: every
  keyword, every operator with precedence, the complete type system. Keep it
  open as a cheatsheet while coding.
- **Compiler Source** — the compiler itself lives in `knot/src/`. The parser
  (`src/parser/mod.rs`) and codegen (`src/codegen/llvm.rs`) are great entry
  points for understanding how the language works under the hood.
- **VS Code Extension** — grab the `knot-vscode` extension for highlighting,
  diagnostics, autocompletion, and hover info.

### Start Building

```bash
knot new myapp           # create project
knot build src/main.knot # compile
knot run src/main.knot   # compile + run
```

### Get Involved

Knot is MIT-licensed. Bug reports, feature ideas, pull requests — all welcome
on the GitHub repo.

---

> **Pro tip**: Keep the [Syntax Reference](SYNTAX.md) open next to your editor.
> It's a faster way to look up operators and types than scrolling through the
> tutorial.
