# Knot 语言语法

> [English](SYNTAX.md)

## 词法

### 关键字

```
abstract  args     as       as!      assert
break     catch    class    continue delete
else      enum     false    for      func
if        import   in       kwargs   match
mixin     mut      new      null     operator
private   return   static   throw    true
try       while    wrap
```

### 字面量

| 类型 | 格式 | 示例 |
|------|------|------|
| 布尔 | `true` / `false` | `true` |
| 整数 | 十进制 / 十六进制 `0x` / 二进制 `0b` / 八进制 `0o`; 后缀 `i8`-`i64` `u8`-`u64` | `42`, `0xFF`, `0b1010`, `42i64` |
| 浮点 | 后缀 `f32` `f64` | `3.14`, `2.0f32` |
| 字符串 | 双引号 `"..."`，支持插值 `"Hello, {name}"` 和转义 `\n` `\t` `\{` | `"hello world"` |
| 原始字符串 | 单引号 `'...'` 或反引号 `` `...` ``，不处理转义 | `'C:\path\to\file'` |
| 空 | `null` | |
| 数组 | `[expr, ...]` | `[1, 2, 3]` |
| 字典 | `{key: value, ...}` | `{"a": 1, b: 2}` |

### 运算符

```
+  -  *  /  %  算术
<< >>         位移
&  |  ^  ~    位运算
== != < > <= >= 比较
&& || !       逻辑
=             赋值
..            范围
??            空合并
as as!        类型转换
.             成员访问
::            静态访问
@             wrap 调用
() [] {}      括号
-> =>         箭头
```

## 语法

### 变量

```knot
x = 42           // 赋值即声明，类型推断为 I32
y: F64 = 3.14    // 显式类型标注

// mut 变量可接受任意类型，类型锁定后不可更改
mut z = 42       // z 是 mut，类型为 I32
z = 43           // ✓ 同类型
// z = "hello"   // ❌ 类型已锁定为 I32

// 未用 mut 声明的变量，类型一旦推断确定不可变更
a = 10           // 推断为 I32
// a = "hi"      // ❌ 类型不匹配
```

### 函数

```knot
func add(a: I32, b: I32) -> I32 {
    return a + b
}

// 默认参数
func greet(name: String = "world") {
    print("Hello, {name}")
}

// 无返回值
func print(msg: String) {
    // ...
}

// 打包所有位置参数
func sumAll(args items: Array[I32]) -> I32 {
    total = 0
    for i in items {
        total = total + i
    }
    return total
}

// 打包所有命名参数
func configure(kwargs opts: Map[String, Any]) {
    // ...
}

// 位置 + 命名参数同时打包
func handle(args items: Array[Any], kwargs opts: Map[String, Any]) {
    // ...
}
```

### 匿名函数 (Lambda)

```knot
add = (a: I32, b: I32) -> a + b
run = () -> { print("hello") }
```

### 控制流

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

// break 2 / continue 2  跳多层循环
while true {
    while true {
        if cond {
            break 2     // 跳出两层
        }
    }
}

// assert
assert x > 0
assert x > 0, "x must be positive"
```

### match

```knot
// match 作为语句——分支必须是大括号块
match expr {
    1 => { print("one") }
    2 => { print("two") }
    else => { print("other") }
}

// match 作为表达式——分支是值
result = match x {
    1 => "one"
    2 => "two"
    else => "other"
}
```

### 异常

```knot
try {
    risky_operation()
} catch e {
    print("error")
}

throw "something went wrong"
```

### 类型转换

```knot
x = value as I32       // 安全转换，失败返回 null
x = value as! I32      // 强制转换，失败抛异常
```

### 类

```knot
abstract class Printable {
    func print() { }
    abstract func format() -> String   // 无方法体，子类必须重写
}

class A {
    func a() { print("a") }
    func c() { print("ac") }
}

class B {
    func b() { print("b") }
    func c() { print("bc") }
}

// mixin 在类体内声明
class C {
    mixin A
    mixin B

    // A 和 B 都有 c()，必须重写
    func c() {
        A.c()       // 用类名消歧，调用 A 的 c
        B.c()       // 调用 B 的 c
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

    // wrap: 第一个参数是输入值（名字任意），不能少于一个参数
    //       后续参数在 @ 调用时传入，参数可有默认值
    wrap into(target: I32 = 0) {
        x = target
        return this
    }
}

// 构造实例
p = Point::new(1, 2)

// wrap 调用：@对象.wrap名(额外参数)
@p.into(10)

// 实例方法用 . 调用
p.dist()
```

### 枚举

```knot
enum Color {
    Red
    Green
    Blue
}
```

### 导入

```knot
import "std.io" as io
import "utils.knot"
```

## 类型

| 有符号整数 | 无符号整数 | 浮点 | 其他 |
|-----------|----------|------|------|
| `I8` `I16` `I32` `I64` | `U8` `U16` `U32` `U64` | `F32` `F64` | `String` `Bool` `Void` `Null` `Any` |

- 可空类型：`I32?`
- 数组类型：`Array[I32]`
- 字典类型：`Map[String, I32]`
- 动态类型：`Any` 可接受任意类型

## 注释

```knot
// 单行注释
/* 多行
   注释 */
```
