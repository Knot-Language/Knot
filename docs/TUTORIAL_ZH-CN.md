# Knot 教程：从零开始系统编程

> [English](TUTORIAL.md)

跟着这篇教程，你很快就能用 Knot 写出自己的程序。不用怕，Knot 的语法
很简洁，如果你写过 JavaScript、Python 或 C，上手会非常快。

我们边写边学，每一节都有能跑的代码。

---

## 目录

1. [Hello, Knot!](#1-hello-knot)
2. [变量和基本类型](#2-变量和基本类型)
3. [函数](#3-函数)
4. [控制流](#4-控制流)
5. [类和对象](#5-类和对象)
6. [枚举](#6-枚举)
7. [错误处理](#7-错误处理)
8. [泛型](#8-泛型)
9. [Mixin](#9-mixin)
10. [Wrap 和装饰器](#10-wrap-和装饰器)
11. [模块和导入](#11-模块和导入)
12. [Extern——调用 C 代码](#12-extern让-knot-调用-c-代码)
13. [综合示例](#13-综合示例来个完整的)
14. [下一步](#14-下一步)

---

## 1. Hello, Knot!

### 把环境搭起来

```bash
git clone https://github.com/your/knot
cd knot
cargo build --release
```

> 需要电脑上装了 `clang`。如果 clang 不在默认路径里，设置一下环境变量：
> `KNOT_CLANG=/path/to/clang`

### 三行代码，跑起来

新建一个文件叫 `hello.knot`，写入：

```knot
import "std/io.knot" as io

func main() -> I32 {
    io.write_i32("Hello, Knot!")
    return 0
}
```

然后：

```bash
set KNOT_STD=D:\path\to\knot\std     # Windows 先设环境变量
knot run hello.knot
```

看到屏幕上出现 `Hello, Knot!` 就成功了。

### 这些代码在干嘛？

- `import "std/io.knot" as io` — 引入标准库的 I/O 模块
- `func main() -> I32` — 程序入口，返回 32 位整数
- `io.write_i32(...)` — 打印字符串到屏幕
- `return 0` — 返回 0 表示正常退出

> **关于 `print()`**：Knot 没有内置 `print`。后续示例中 `print(x)` 是
> `io.write_i32(x)` 的简写。需先 `import "std/io.knot" as io` 并设
> `KNOT_STD` 环境变量。详见[模块导入](#12-模块和导入)。

注意：Knot **不用写分号**。换一行就是语句结束，跟 Python 一样。

---

## 2. 变量和基本类型

### 直接赋值就等于声明

Knot 不需要 `let`、`var`、`const` 这些词。你第一次给一个名字赋值，它就自动
变成变量了，类型也让编译器自己猜：

```knot
age = 25           // 编译器一看 25 就知道是 I32（整数）
price = 9.99       // 有小数点，那肯定是 F64（浮点数）
name = "小明"      // 带引号的就是 Array[Char]
ok = true          // true/false 就是 Bool（布尔）
nothing = null     // null 表示"啥也没有"
```

**什么是 I32、F64？** 就当它们是"整数"和"小数"的高级叫法。
I32 = 32 位整数（大概正负 21 亿范围），F64 = 64 位浮点数（更精确的小数）。
日常写代码你用 `42` 和 `3.14` 就够了，编译器会自动处理。

### 想要精确控制类型？加上标注就行

```knot
count: I64 = 0       // 我要一个 64 位的大整数
ratio: F32 = 3.14    // 我要一个 32 位的浮点数（省点内存）
```

### 变量一旦定了类型，就不能乱塞别的东西

```knot
a = 10      // 好的，编译器记住了：a 是 I32
a = 20      // 没问题，20 也是整数
// a = "hi" // ❌ 编译不通过！a 是整数，不能突然变成字符串
```

这个设计很有用——帮你提前拦住低级错误，不用等到运行时才发现。

### 有时候确实需要"空"

万一变量可能没值呢？加个 `?`：

```knot
maybe: I32? = null      // "我可能是个整数，也可能啥也不是"

maybe = 42              // 现在有值了

// x: I32 = maybe       // ❌ 不行！编译器说：万一 maybe 是 null 呢？
x = maybe ?? 0          // ✅ 如果 maybe 是 null，就用 0
x = maybe as! I32       // ✅ 我确定它有值，强制取出来（如果是 null 就崩）
```

### Knot 里你能用的"字"全家福

| 说人话 | 类型 | 怎么写的 |
|--------|------|---------|
| 整数 | `I8` `I16` `I32` `I64` | 写 `42` 默认就是 `I32` |
| 非负整数 | `U8` `U16` `U32` `U64` | 加后缀，比如 `42u32` |
| 小数 | `F32` `F64` | 写 `3.14` 默认就是 `F64` |
| 文本 | `Array[Char]` | `"hello"` |
| 单字符 | `Char` | `'a'` |
| 真假 | `Bool` | `true` / `false` |
| 空 | `Null` | `null` |


### 数字的写法花样挺多

```knot
42              // 普通十进制
0xFF            // 十六进制 = 255（前缀 0x）
0b1010          // 二进制 = 10（前缀 0b）
0o777           // 八进制 = 511（前缀 0o）
1_000_000       // 100 万，下划线只是看着舒服，编译器会忽略
42i64           // 显式指定：这是 64 位的
3.14            // 小数默认 64 位
2.0f32          // 显式指定：这是 32 位的
1.5e10          // 科学计数法 = 150 亿
```

### 字符字面量

单引号包单个字符就是 `Char` 类型（单字节），也支持转义：

```knot
'A'
'\n'        // 换行符
'0'         // 数字 0
```

### 字符串有"加工版"和"原味版"

双引号里的 `\n` 会变成换行、`\t` 会变成制表符，这叫"转义"。
字符串底层其实是 `Array[Char]`（所以你可以像数组一样用下标）：

```knot
"你好\n世界"     // 输出两行
```

**反引号字符串**（`` `...` ``）不处理转义，原样保留。写 Windows 路径和正则的神器：

```knot
`C:\Users\小明\文件.txt`
`no \n escape here`
```

**单引号**看情况：一个字符就是 `Char`，多个字符就当原始字符串（跟反引号一样）：

```knot
'A'                          // Char 类型
'C:\Users\name\file.txt'     // 原始字符串（多字符 → 不转义）
```

---

## 3. 函数

### 把一段操作打包成函数

```knot
func add(a: I32, b: I32) -> I32 {
    return a + b
}

func sayHi(name: String) {
    print("嗨，" + name + "！")
}
```

规律很简单：
- `func` 开头
- 括号里写参数（名字: 类型）
- `->` 后面写返回什么类型
- 不写 `->` 就是"不返回东西"

### 调一下试试

```knot
result = add(3, 5)   // 8
sayHi("小红")        // 输出：嗨，小红！
```

### 有些参数可以不传——给个默认值

```knot
func greet(name: String = "世界", times: I32 = 1) {
    for i in 0..times {
        print("你好，" + name + "！")
    }
}

greet()                // 你好，世界！（只输出一次）
greet("Knot")          // 你好，Knot！
greet("Knot", 3)       // 你好，Knot！× 3 次
```

注意：有默认值的参数得放在**最后面**，不能跳着写默认值。

### 参数个数不确定怎么办？用 args

```knot
func sumAll(args items: Array[I32]) -> I32 {
    total = 0
    for i in items {
        total = total + i
    }
    return total
}

sumAll(1, 2, 3, 4, 5)   // 15 —— 随便传几个都行
```

`args` 把传进来的所有参数打包成一个数组，你想传几个传几个。

### 随手写个小函数——Lambda

有时候只需要用一个临时函数，不想正经起名定义，那就用箭头：

```knot
double = (x: I32) -> x * 2       // 输入 x，返回 x*2
run = () -> { print("搞定") }    // 无参数，执行一个语句块

double(5)   // 10
```

> Lambda 里用到外面的变量时是**拷贝一份**进去的，所以改外面的值不会影响
> lambda 里的副本。

### return 就是"拿结果走人"

```knot
func abs(x: I32) -> I32 {
    if x >= 0 {
        return x    // 正数直接返回
    }
    return -x       // 负数取反再返回
}
```

函数碰到 `return` 就结束，后面代码不跑了。如果函数不返回值（`Void`），写个
光秃秃的 `return;` 就可以提前溜。

---

## 4. 控制流——让程序有脑子

### if/else：分情况处理

```knot
if x > 0 {
    print("正数")
} else if x == 0 {
    print("零")
} else {
    print("负数")
}
```

条件必须是 Bool 类型。

`if` 还能当成表达式用，直接"算出"一个值：

```knot
label = if score >= 60 { "及格" } else { "不及格" }
```

当表达式用时，**必须有 else**，不然没法确定值。

### while：不满足条件就停

```knot
x = 10
while x > 0 {
    print(x)
    x = x - 1
}
// 输出 10, 9, 8, ... 一直到 1
```

### for-in：遍历一堆东西

这是 Knot 里最常用的循环，可以遍历范围或数组：

```knot
// 从 0 数到 9（0..10 不包含 10）
for i in 0..10 {
    print(i)
}

// 遍历数组
for name in ["张三", "李四", "王五"] {
    print(name)
}

// 遍历字典（同时拿到键和值）
scores = {"语文": 90, "数学": 85}
for subject, score in scores {
    print(subject + "：" + score + "分")
}
```

### match：多路选择，优雅

```knot
match x {
    1 => { print("一") }
    2 => { print("二") }
    else => { print("其他") }
}
```

也可以当表达式用：

```knot
chinese = match x {
    1 => "一"
    2 => "二"
    else => "其他"
}
```

`else` 是兜底的分支，必须写。分支之间不会"掉到下一行"，不用写 break。

### break 和 continue：跳出去 / 下一圈

```knot
while true {
    while true {
        if done {
            break 2      // 跳出两层循环
        }
        if skip {
            continue     // 跳过本轮，开始下一轮
        }
    }
}
```

`break 2` 中的数字就是你想跳出几层，默认 `break` 只跳一层。

### assert：我说啥就是啥，不对就崩

```knot
assert x > 0
assert x > 0, "x 必须大于 0，现在不对哦"
```

断言是给程序员用的——你对某些条件非常确信的时候写上去，万一错了程序当场报错，
帮你快速定位问题。跟 debug/release 模式无关，永远生效。

---



## 5. 类和对象

### 把数据和操作打包在一起

```knot
class Point {
    x: I32
    y: I32

    func new(x: I32, y: I32) {
        this.x = x
        this.y = y
    }

    func distance() -> F64 {
        return sqrt(this.x * this.x + this.y * this.y)
    }
}
```

- `x: I32` 和 `y: I32` 是字段——就是存在对象里的数据
- `func new(...)` 是构造函数——`Point::new(3, 4)` 这行就是调它来创建对象的。
  **不要**给构造函数加 `static` 或写返回类型。
- `this` 指向"当前这个对象自己"。想访问自己的字段或方法，就用 `this.xxx`

### 创建对象

```knot
p = Point::new(3, 4)
d = p.distance()    // 5.0 —— 勾三股四弦五！
```

### 静态方法：属于这个类，不属某个对象

```knot
class Math {
    static func abs(x: I32) -> I32 {
        return if x >= 0 { x } else { -x }
    }
}

result = Math::abs(-5)   // 5
```

静态方法不用创建对象就能调用，适合放工具函数。

### 析构函数：对象销毁时自动打扫

```knot
class Resource {
    handle: I32

    func delete() {
        close(this.handle)    // 对象不用了，关掉句柄
    }
}
```

`func delete()` 就是析构函数，不需要参数和返回类型。对象被释放的时候编译器
会自动帮你调它。

### private：有些东西不想让外面碰

```knot
class Secret {
    private code: I32

    private func encrypt() -> I32 {
        return this.code ^ 0xFF     // 异或加密
    }

    func getCode() -> I32 {
        return this.encrypt()       // 外面调这个方法可以间接访问
    }
}
```

`private` 的东西只有这个类自己能用，外面看不到也改不了。

### 重载运算符：让你的类也能 `+` `-` `==` `<<`

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
v3 = v1 + v2          // Vector::new(4, 6) —— 实际调用了 Vector__op_plus
equals = v1 == v2     // false
```

支持 `+` `-` `*` `/` `%` `==` `!=` `<` `>` `<=` `>=` `<<` `>>` `&` `|` `^`。

---

## 6. 枚举

枚举就是"只能从这几个里面选一个"：

```knot
enum Color {
    Red
    Green
    Blue
}

c = Color.Red
```

枚举值之间**换行分隔**，不用写逗号。还可以带泛型：

```knot
enum Option[T] {
    Some
    None
}
```

---

## 7. 错误处理

### throw：出事了，往外抛

```knot
throw "文件打不开"
throw 404
throw MyError::new("连接断了")
```

throw 可以抛任何东西——字符串、数字、对象都行。

### try/catch：接住它

```knot
try {
    mightFail()
} catch e {
    print("出错了：" + e)
}
```

还可以按类型分别处理：

```knot
try {
    riskyOperation()
} catch e: String {
    print("字符串错误：" + e)
} catch e: I32 {
    print("错误码：" + e)
}
```

try 块里一抛异常，程序立马跳到最近匹配的 catch 分支，try 里剩余代码不跑了。

---

## 8. 泛型

泛型就是"不写死类型，用的时候再定"。编译器会在每个具体使用时自动生成
对应的真实代码，所以性能零损耗。

### 泛型函数

```knot
func identity[T](x: T) -> T {
    return x             // 不管 T 是啥，原样返回
}

a = identity(42)         // T 自动推断为 I32
b = identity("hello")    // T 自动推断为 String

// 泛型函数也可以接受多个类型参数：
func pair[K, V](key: K, val: V) -> (K, V) {
    return (key, val)
}
c = pair("score", 100)   // K=String, V=I32
```

`[T]` 里的 T 是占位符，调用时编译器根据你传的参数自动填充。

### 泛型类

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

一个 `Box` 类可以装任何类型，不用每种类型写一个类。

### 泛型方法

方法自己也可以带泛型参数，独立于类的泛型：

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

## 9. Mixin——不用继承也能复用代码

`mixin` 能把一个类里的字段和方法全部"粘贴"到另一个类中：

```knot
class Logger {
    func log(msg: String) {
        print("[LOG] " + msg)
    }
}

class App {
    mixin Logger      // 把 Logger 的东西全部搬进来

    func run() {
        this.log("应用启动")   // 直接就能用 Logger 的方法
    }
}
```

### 两个 mixin 有同名方法怎么办？

那就自己重写，明确说调用谁的：

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

    func shared() {           // 重写来解决冲突
        A.shared()
        B.shared()
    }
}
```

不重写的话编译器会报错——它不知道你想用哪个。

### 抽象类：只定义接口，不能创建实例

```knot
abstract class Printable {
    abstract func format() -> String    // 只声明不实现

    func print() {
        println(this.format())
    }
}

class Report {
    mixin Printable

    func format() -> String {          // 必须实现
        return "报告内容..."
    }
}
```

抽象类只能被 mixin，不能 `new`。抽象方法没函数体，子类必须实现。
优先级从高到低：自己的方法 > mixin 的方法 > mixin 的 mixin 的方法。

---

## 10. Wrap 与 @ 装饰器——给函数穿衣服

`wrap` 是一种**只能用 `@` 调用的特殊函数**。`@` 放在一个函数前面，就会把
那个函数当作参数传给 wrap。

最简单的例子：

```knot
wrap debug(input) {
    print("debug：函数被调用了")
    return input
}

@debug
func hello() {
    return "world"
}
// 这等价于：hello = debug(hello)
// 原来的 hello 函数被 debug 包装了一下，但名字不变
```

### 还能传额外参数

```knot
wrap withArg(target, value) {
    print("用 " + value + " 包了一下")
    return target
}

@withArg("magic")
func worker() {
    // ...
}
// 等价于：worker = withArg(worker, "magic")
```

### 类里面的 wrap

类里定义的 wrap，第一个参数固定是 `this`，第二个才是被装饰的东西：

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
// 等价于：range.binarySearch(range, isValid)
```

### 三条规矩

- `wrap` 不能像普通函数那样直接调用——只能用 `@`。
- `@name` 修下一行的声明，被修饰的东西自动变成第一个参数。
- `@name(args)` 把额外参数传给 wrap 的后续参数。

零运行时开销——编译期就全部展开好了。

---

## 11. 模块和导入

### 引入标准库

Knot 自带标准库 `std/`。用 `KNOT_STD` 环境变量指定路径：

```bash
set KNOT_STD=D:\dev\knot\std        # Windows
export KNOT_STD=/path/to/knot/std   # Linux/macOS
```

然后引入 I/O 模块：

```knot
import "std/io.knot" as io

func main() -> I32 {
    cout = io.Cout::new()
    cout << "Hello " << "World\n"   // 流式输出（operator <<）
    io.write_i32(42)                // 输出整数
    name = io.read_line()           // 读取一行
    return 0
}
```

### 引入别的文件

```knot
import "utils.knot"           // 直接用文件路径
import "std/io.knot" as io    // 加个别名，避免名字冲突
import math                   // 模块名，编译器会从搜索路径找
```

### 有别名 vs 无别名

不加 `as`，导入的东西直接混进当前作用域，好像本来就在这个文件里：

```knot
import "utils.knot"

result = compute(42)   // 直接用 utils.knot 里定义的 compute
```

加上 `as`，就得通过别名访问：

```knot
import "utils.knot" as utils

result = utils.compute(42)
```

### 项目配置文件

项目根目录放一个 `Knot.toml`：

```toml
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.knot"

[dependencies]
```

### 创建新项目一把梭

```bash
knot new myapp
```

自动生成 `Knot.toml` 和 `src/main.knot`，开箱即用。

### 安全设计

导入在编译期完成（不是运行时加载），而且**禁止 `..` 路径穿越**——你没法
`import "../secret.knot"`。编译器读入目标文件后直接内联到当前文件，
全部在编译期搞定。

---

## 12. Extern——让 Knot 调用 C 代码

Knot 是系统编程语言，跟 C 打交道是基本操作。你在 Knot 里写签名，在 `.c` 文件
里写实现，编译器自动帮你把两边连起来。

### 外部函数

```knot
extern func puts(s: I8*) -> I32
extern func strlen(s: I8*) -> I64
extern func sin(x: F64) -> F64
```

- `extern func` 只声明函数签名，**不写函数体**。
- C 侧在**同名** `.c` 文件中实现（比如 `utils.knot` 对应 `utils.c`）。
- 编译器自动找到并编译 `.c` 文件，最后一起链接。

### 外部类

把 C 的 struct 声明为 Knot 的外部类。Knot 侧只能通过指针访问，不会自己创建：

```knot
extern class File {
    fd: I32
}

extern func fopen(path: I8*, mode: I8*) -> File
extern func fclose(f: File) -> I32
```

C 侧写法（引入 `knot.h`）：

```c
#include "knot.h"

typedef struct { knot_i32 fd; } File;
```

### 类型对照表（knot.h）

编译器自带 `knot.h` 头文件，C 代码里 include 它就能用对应类型：

| Knot | C |
|------|---|
| `I8`~`I64` | `int8_t`~`int64_t` |
| `U8`~`U64` | `uint8_t`~`uint64_t` |
| `F32` / `F64` | `float` / `double` |
| `Char` | `char` |
| `Bool` | `int32_t` |
| `I8*` / `T*` | `const char*` / 指针 |
| 外部类 | 指向同名结构体的指针 |

### 举个栗子

`utils.knot`：

```knot
extern func strlen(s: I8*) -> I64
```

`utils.c`：

```c
#include "knot.h"
#include <string.h>

knot_i64 strlen(const char* s) {
    return (knot_i64)strlen(s);
}
```

把 `utils.c` 放在 `utils.knot` 旁边就行，别的不用管——编译器搞定。

---

## 13. 综合示例——来个完整的

下面是一个小项目，把前面学的东西串起来：

```knot
// --- 抽象 Logger：谁混入了它就能打日志 ---
abstract class Logger {
    abstract func prefix() -> String

    func log(msg: String) {
        print(this.prefix() + msg)
    }
}

// --- 一个泛型仓库：可以装任意类型的数据 ---
class Repository[T] {
    mixin Logger

    private items: Array[T]

    func new() {
        this.items = []
    }

    func prefix() -> String {
        return "[仓库] "
    }

    func add(item: T) {
        this.items.push(item)
        this.log("已添加：" + item)
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

// --- timed wrap：自动测量执行时间 ---
wrap timed(target) {
    start = now()
    result = target()
    elapsed = now() - start
    print("耗时 " + elapsed + "ms")
    return result
}

// --- 数据处理，用 @timed 自动计时 ---
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

// --- 入口 ---
func main() -> I32 {
    result = processData()
    assert result == 9, "不对哦，结果是 " + result
    return 0
}
```

### 这个例子用了哪些知识点

| 特性 | 示例 |
|------|------|
| 类 | `class Repository[T]` |
| 泛型 | `Repository[T]`, `Array[T]` |
| Mixin | `mixin Logger` |
| 抽象类 | `abstract class Logger` |
| 字段 | `private items: Array[T]` |
| 构造函数 | `func new()` |
| 私有成员 | `private items` |
| 可空类型 | `T?` |
| 空值合并 | `??` |
| For-in 循环 | `for i in 0..repo.count()` |
| Wrap | `wrap timed(target)` |
| @ 装饰器 | `@timed` |
| 断言 | `assert result == 9, "..."` |
| 入口点 | `func main() -> I32` |

---

## 14. 下一步

### 还要看什么

- **[语言语法参考](SYNTAX_ZH-CN.md)**——每个关键字、每个运算符的完整说明，
  包含优先级表和类型系统细节。写代码时当字典翻就行。
- **Knot 源代码**——编译器本身就在 `knot/src/` 下。解析器
  (`src/parser/mod.rs`) 和代码生成 (`src/codegen/llvm.rs`) 都是不错的切入口。
- **VS Code 插件**——安装 `knot-vscode` 扩展，获得语法高亮、错误提示、
  自动补全和悬停文档。

### 开工

```bash
knot new myapp           # 创建项目
knot build src/main.knot # 编译
knot run src/main.knot   # 编译 + 运行
```

### 参与进来

Knot 是 MIT 开源的。任何贡献、bug 报告、功能建议都欢迎——
直接去 GitHub 仓库提 issue 或 PR 就好。

---

> **小贴士**：写代码的时候把[语法参考](SYNTAX_ZH-CN.md)开着当速查手册，
> 事半功倍。
