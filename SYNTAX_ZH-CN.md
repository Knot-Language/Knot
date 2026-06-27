# Knot 语言语法参考

> [English](SYNTAX.md)

## 目录

- [1. 词法结构](#1-词法结构)
- [2. 注释](#2-注释)
- [3. 关键字](#3-关键字)
- [4. 字面量](#4-字面量)
- [5. 运算符](#5-运算符)
- [6. 变量与类型](#6-变量与类型)
- [7. 函数](#7-函数)
- [8. 泛型](#8-泛型)
- [9. 控制流](#9-控制流)
- [10. 类与面向对象](#10-类与面向对象)
- [11. 枚举](#11-枚举)
- [12. 异常处理](#12-异常处理)
- [13. 类型转换](#13-类型转换)
- [14. 模块导入](#14-模块导入)
- [15. 类型系统](#15-类型系统)

---

## 1. 词法结构

### 文件编码

源文件必须使用 UTF-8 编码，无 BOM。

### 空白

空格、制表符用于分隔 token，无实际意义。换行 `\n` 作为语句结束符。以下上下文中换行将被忽略：

- 括号 `()`、`[]`、`{}` 内部
- 操作符之后（表达式可跨行）

### 标识符

字母或下划线开头，后续可包含字母、数字、下划线。大小写敏感。

```
valid_name
_secret
data42
camelCase
PascalCase
```

### 命名规范

Knot 采用以下命名约定（编译器不强制，但强烈推荐）：

| 类别 | 风格 | 示例 |
|------|------|------|
| 类名、枚举名、文件名 | **PascalCase**（大驼峰） | `HttpClient`, `Point2D`, `main.knot` |
| 函数名、方法名 | **camelCase**（小驼峰） | `getUser`, `parseInt`, `toString` |
| 字段名、变量名 | **camelCase**（小驼峰） | `firstName`, `itemCount`, `isReady` |
| Wrap 名 | **camelCase**（小驼峰） | `debugLog`, `withTransaction` |
| 常量 | **UPPER_SNAKE_CASE** | `MAX_SIZE`, `DEFAULT_PORT` |

统一原因：类与实例通过大小写即可区分——`Point` 是类，`point` 是变量；
`Point.getX()` 一眼看出 `Point` 是类名、`getX` 是方法。

### 注释

单行注释从 `//` 到行尾。多行注释以 `/*` 开始、`*/` 结束，可嵌套。

```knot
// 这是单行注释

/*
   这是多行注释
   /* 可以嵌套 */
*/
```

---

## 2. 关键字

以下为保留关键字，不可用作标识符：

```
abstract  args     as       as!
assert    break    catch    class    continue
delete    else     enum     false    for
func      if       import   in       kwargs
match     mixin    new      null
operator  private  return   static   throw
true      try      while    wrap
```

---

## 3. 字面量

### 布尔

`true` 和 `false`，类型为 `Bool`。

### 整数

支持十进制、十六进制 `0x`、二进制 `0b`、八进制 `0o`。

可在数字间插入 `_` 提高可读性。可加后缀指定宽度：

| 后缀 | 类型 |
|------|------|
| `i8` | 有符号 8 位 |
| `i16` | 有符号 16 位 |
| `i32` (默认) | 有符号 32 位 |
| `i64` | 有符号 64 位 |
| `u8` | 无符号 8 位 |
| `u16` | 无符号 16 位 |
| `u32` | 无符号 32 位 |
| `u64` | 无符号 64 位 |

```
42                   // I32
0xFF                 // 十六进制 255
0b1010               // 二进制 10
0o777                // 八进制 511
1_000_000            // 下划线分隔
42i64                // 显式 I64
```

### 浮点

支持小数点和科学计数法。后缀指定精度：

```
3.14                 // F64 (默认)
2.0f32               // F32
1.5e10               // 科学计数
```

### 字符串

双引号字符串 `"..."` 支持以下转义序列：

| 转义 | 含义 |
|------|------|
| `\n` | 换行 |
| `\t` | 制表符 |
| `\r` | 回车 |
| `\\` | 反斜杠 |
| `\"` | 双引号 |
| `\{` | 左大括号 |

```knot
"Hello, World!"
"line1\nline2"
```

### 原始字符串

单引号 `'...'` 或反引号 `` `...` `` 中的字符不处理转义，适合写路径和正则：

```knot
'C:\Users\name\file.txt'
`raw \n no escape`
```

### 空值

`null` 表示空值，类型为 `Null`。可赋值给任意 `T?` 可空类型。

### 数组

方括号包裹，逗号分隔：

```knot
[1, 2, 3]
["a", "b"]
[]                   // 空数组
```

### 字典

花括号包裹，`key: value` 对，逗号分隔。key 可以是任意表达式：

```knot
{"name": "Knot", "year": 2026}
{a: 1, b: 2}
{}                   // 空字典
```

---

## 4. 运算符

按优先级从低到高排列：

| 优先级 | 运算符 | 结合性 | 说明 |
|--------|--------|--------|------|
| 10 | `=` `+=` `-=` `*=` `/=` `%=` `&=` `\|=` `^=` | 右 | 赋值和复合赋值 |
| 20 | `\|\|` | 左 | 逻辑或 |
| 25 | `\|` | 左 | 位或 |
| 27 | `^` | 左 | 位异或 |
| 29 | `&` | 左 | 位与 |
| 30 | `&&` | 左 | 逻辑与 |
| 35 | `<<` `>>` | 左 | 位移 |
| 40 | `==` `!=` | 左 | 相等 |
| 50 | `<` `>` `<=` `>=` | 左 | 比较 |
| 55 | `..` | 左 | 范围 |
| 60 | `??` | 左 | 空值合并 |
| 65 | `as` `as!` | 左 | 类型转换 |
| 70 | `+` `-` | 左 | 加减 |
| 80 | `*` `/` `%` | 左 | 乘除模 |
| 85 | `-x` `!x` `~x` `++x` `--x` | 右(前缀) | 一元 |
| 90 | `x.y` `x::y` `x()` `x[]` `x++` `x--` | 左(后缀) | 调用/访问 |

额外的运算符：

| 符号 | 说明 |
|------|------|
| `->` | 函数返回类型标记 |
| `=>` | match 分支匹配 |
| `@` | wrap 调用 |
| `..` | 范围运算符 |

---

## 5. 变量与类型

### 声明与推断

Knot 采用**赋值即声明**机制。变量首次被赋值时自动声明，类型由编译器推断：

```knot
x = 42                 // 推断为 I32
y = 3.14               // 推断为 F64
name = "Knot"          // 推断为 String
flag = true            // 推断为 Bool
```

### 类型标注

可在变量名后标注类型以覆盖推断：

```knot
count: I64 = 0         // 显式标注为 I64
pi: F32 = 3.14         // 标注为 F32
```

### 类型锁定

变量的类型一旦确定，后续赋值必须同类型：

```knot
a = 10                 // 锁定为 I32
a = 20                 // ✓ 合法
// a = "hello"         // ❌ 类型不匹配，编译错误
```

数值类型之间有隐式兼容（`I32` 可隐式转换到 `F64` 等）。

### 可空类型

类型名后加 `?` 表示可为 `null`。`T?` 类型的值不可直接赋值给 `T`，须显式检查或转换：

```knot
maybe: I32? = null
maybe = 42

// value: I32 = maybe     // ❌ 类型不匹配
value = maybe as! I32     // ✓ 强制转换
value = maybe ?? 0        // ✓ 空值合并
```

---

## 6. 函数

### 定义

`func` 关键字定义函数。参数列表在 `()` 内，返回类型在 `->` 后。无返回类型视为 `Void`：

```knot
func add(a: I32, b: I32) -> I32 {
    return a + b
}

func nothing() {          // 等价于 -> Void
    print("side effect")
}
```

### 调用

函数调用使用 `name(args)` 语法：

```knot
sum = add(3, 5)
nothing()
```

### 默认参数

参数可指定默认值。有默认值的参数在调用时可省略：

```knot
func greet(name: String = "world", times: I32 = 1) {
    // ...
}

greet()                  // name="world", times=1
greet("Knot")            // name="Knot", times=1
greet("Knot", 3)         // name="Knot", times=3
```

默认参数必须从右向左连续提供，不可跳过。

### 打包参数 (args/kwargs)

`args` 将所有位置参数打包为数组。`kwargs` 将所有命名参数打包为字典。二者互斥，且不能与普通参数共存：

```knot
func sumAll(args items: Array[I32]) -> I32 {
    total = 0
    for i in items { total = total + i }
    return total
}

```

### 返回值

`return` 语句结束函数执行并返回值。`return` 不带值时表示返回空：

```knot
func early(x: I32) -> I32 {
    if x < 0 {
        return 0
    }
    return x * 2
}
```

### Lambda（匿名函数）

`(参数) -> 表达式` 或 `(参数) -> { 语句块 }` 创建匿名函数。Lambda 捕获的变量是**值拷贝**：

```knot
double = (x: I32) -> x * 2
run = () -> { print("hello") }

result = double(5)       // 10
```


---

## 8. 泛型

泛型允许函数、类、枚举接受类型参数，实现参数化多态。

> 注意：泛型语法已被解析器完整支持，但编译器后端尚未实现单态化展开。定义泛型参数会触发编译警告。

### 泛型函数

函数名后可用 [T, U, ...] 声明泛型参数：

```knot
func identity[T](x: T) -> T {
    return x
}

func pair[K, V](key: K, val: V) -> Map[K, V] {
    return {key: val}
}

result = identity(42)      // T 推断为 I32
result = identity("hello") // T 推断为 String
```
当调用泛型函数时，类型参数由编译器根据实参类型自动推断。

### 泛型类

类名后可声明泛型参数，用于字段和方法类型：

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
### 泛型枚举

枚举也可带泛型参数：

```knot
enum Option[T] {
    Some
    None
}
```

### 泛型方法

类中方法可独立声明泛型参数（在方法名后）：

```knot
class Util {
    static func map[T, U](arr: Array[T], f: (T) -> U) -> Array[U] {
        // ...
    }
}
```

---

## 9. 控制流

### if / else

条件可以是任意表达式，非零为真。`if`-`else` 链：

```knot
if x > 0 {
    print("positive")
} else if x == 0 {
    print("zero")
} else {
    print("negative")
}
```

`if` 也可以作为表达式使用。此时 `else` 分支必须存在：

```knot
abs = if x >= 0 { x } else { -x }
```

### while

前置条件循环：

```knot
while x > 0 {
    x = x - 1
}
```

### for-in

遍历范围、数组或字典：

```knot
for i in 0..10 {         // 0 到 9
    print(i)
}

for item in [1, 2, 3] {
    print(item)
}
```

### match

多分支模式匹配。语句形式的分支必须用块。表达式形式的分支可以是值：

```knot
// 语句形式
match x {
    1 => { print("one") }
    2 => { print("two") }
    else => { print("other") }
}

// 表达式形式
label = match x {
    1 => "one"
    2 => "two"
    else => "other"
}
```

`else` 分支为默认兜底，无穿透。

### break / continue

`break` 结束最近循环，`continue` 跳到下次迭代。后面跟数字指定跳出的循环层数：

```knot
while true {
    while true {
        if done {
            break 2      // 跳出两层
        }
        if skip {
            continue     // 跳一层
        }
    }
}
```

### assert

断言在**所有编译模式**下生效（无 debug/release 区分）。断言失败抛出运行时错误：

```knot
assert x > 0
assert x > 0, "x must be positive"
```

---

## 10. 类与面向对象

### 类定义

`class` 关键字定义类。类体可包含字段、方法、静态方法、操作符重载、wrap 和 mixin：

```knot
class Point {
    x: I32               // 字段声明
    y: I32 = 0           // 带默认值的字段
}
```

### 实例化

使用 `::new` 调用构造器：

```knot
p = Point::new(10, 20)
```

### 构造器 (new)

`func new` 是构造器。隐式返回类类型，不需要也不应写 `static` 或返回类型（写了会警告）：

```knot
class Point {
    x: I32

    func new(x: I32) {
        this.x = x
    }
}
```

### 析构器 (delete)

`func delete` 是析构器，对象释放时自动调用。不需要参数和返回类型：

```knot
class Resource {
    func delete() {
        cleanup()
    }
}
```

### 实例方法与 this

实例方法内部 `this` 指向当前对象。访问自身字段必须用 `this.field`：

```knot
class Point {
    x: I32
    func get_x() -> I32 {
        return this.x
    }
}
```

### 静态方法

`static func` 定义类级别方法。调用时用 `ClassName::method()` 语法：

```knot
class Math {
    static func abs(x: I32) -> I32 {
        return if x >= 0 { x } else { -x }
    }
}

result = Math::abs(-5)   // 5
```

### mixin

`mixin` 将源类的所有字段和方法复制到目标类中。在类体内部声明：

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

    func c() {            // 必须重写：A 和 B 都有 c()
        A.c()             // 用类名消歧
        B.c()
    }
}
```

多个 mixin 提供同签名方法时，目标类必须显式重写。在重写中可用 `ClassName.method()` 消歧。目标类中的同名方法优先级最高。

### 抽象类

`abstract class` 不可直接实例化，只能作为 mixin 源使用：

```knot
abstract class Printable {
    abstract func format() -> String   // 抽象方法无体
    func print() { }
}
```

抽象方法用 `abstract func` 声明，不能写方法体，子类必须实现。

### 访问控制

`private` 关键字限制成员仅在当前类内可访问：

```knot
class Secret {
    private code: I32

    private func encrypt() {
        // ...
    }
}
```

### 操作符重载

`operator` 关键字在类内定义操作符行为。支持 `+` `-` `*` `/` `%` `==` `!=` `<` `>` 等：

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

`wrap` 是一种**只能用 `@` 语法调用的特殊函数**。不能像普通函数那样显式调用。`@` 修饰下一行的声明，被修饰的东西自动成为第一个参数。后续参数通过 `@name(...)` 传入。

**顶层 wrap**

```knot
wrap wrapperWithArgument(funcIn, argument) {
    print(funcIn(argument))
    return funcIn
}

@wrapperWithArgument(111)
func test(number) -> I32 {
    return number + 42
}
// 输出: 153 (111 + 42)
// 等价于: test = wrapperWithArgument(test, 111)
```

无额外参数时：

```knot
wrap debug(input) {
    print("debugging...")
    return input
}

@debug
func hello() { return "world" }
// 等价于: hello = debug(hello)
```

**类内 wrap** — 第一个参数是 `this`，第二个参数是被修饰的东西：

```knot
class Range {
    left: I32
    right: I32

    wrap binarySearch(check) {
        // this = Range 对象
        // check = @ 下方定义的函数
    }
}

@range.binarySearch
func isValid(mid: I32) -> Bool {
    return mid * mid <= 100
}
// 等价于: range.binarySearch(range, isValid)
```

**核心规则**：

- `wrap` 不能显式调用，只能通过 `@` 使用
- `@name` 修饰下一行声明，被修饰者自动成为第一个参数
- `@name(args)` 额外参数传给 wrap 的后续参数
- 类内 wrap 第一个参数是 `this`，第二个是被修饰者
- 编译期内联，零开销

---

## 11. 枚举

`enum` 关键字定义枚举类型。枚举值换行分隔，无逗号：

```knot
enum Color {
    Red
    Green
    Blue
}

c = Color.Red
```

---

## 12. 异常处理

### throw

`throw` 抛出任意类型的值：

```knot
throw "error message"
throw 404
```

### try / catch

`try`-`catch` 捕获异常。`catch` 后可指定捕获变量，可选类型过滤：

```knot
try {
    risky()
} catch e {
    print(e)
}
```

多个 catch 分支按类型过滤：

```knot
try {
    mightFail()
} catch e: String {
    print("字符串异常: " + e)
} catch e: I32 {
    print("错误码: " + e)
}
```

`throw` 将异常值存入全局异常寄存器并跳转到最近的 catch 标签。`catch` 块通过 `CatchEntry` 加载异常值。当前仅支持单层 try/catch，不支持嵌套。

---

## 13. 类型转换

### as（安全转换）

返回 `null` 如果转换失败：

```knot
x = value as I32
```

### as!（强制转换）

转换失败时抛出异常：

```knot
x = value as! I32
```

### 空值合并

`??` 运算符：左值为 `null` 时返回右值：

```knot
x = maybe ?? 0
```

---

## 14. 模块导入

### import

`import` 加载其他文件。支持字符串路径和模块路径：

```knot
import "std/io.knot" as io
import "utils.knot"
```

### 项目配置

项目根目录下的 `Knot.toml` 声明项目元信息和依赖：

```toml
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.knot"

[dependencies]
```

### 入口函数

可执行程序必须包含 `func main() -> I32` 作为入口，退出码通过返回值指定：

```knot
func main() -> I32 {
    return 0
}
```

---

## 15. 类型系统

### 基础类型

| 类别 | 类型 | 说明 |
|------|------|------|
| 有符号整数 | `I8` `I16` `I32` `I64` | 默认整数字面量 → `I32` |
| 无符号整数 | `U8` `U16` `U32` `U64` | |
| 浮点 | `F32` `F64` | 默认浮点字面量 → `F64` |
| 字符串 | `String` | 不可变 UTF-8 |
| 布尔 | `Bool` | `true` / `false` |
| 空 | `Null` | 唯一值 `null` |
| 无返回 | `Void` | 函数无返回值 |

### 复合类型

| 类型 | 语法 | 说明 |
|------|------|------|
| 可空 | `T?` | 可为 `null` |
| 数组 | `Array[T]` | 动态长度 |
| 字典 | `Map[K, V]` | 键值对 |
| 类类型 | `ClassName` | 自定义类 |

### 类型推断规则

- `42` → `I32`
- `3.14` → `F64`
- `"hello"` → `String`
- `true` / `false` → `Bool`
- `null` → `Null`
- 数组 `[1, 2, 3]` → `Array[I32]`
- 运算结果取操作数中较宽的类型
