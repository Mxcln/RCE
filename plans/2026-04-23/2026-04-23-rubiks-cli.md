# rubiks-cli V1 Implementation Plan

**Goal:** 实现首版用户入口 crate，包含基础子命令、交互式 REPL 与 ASCII 展开图渲染。

**Architecture:** `rubiks-cli` 依赖 `rubiks-core`，并通过 `rubiks-alg` 接入训练用随机打乱。CLI 层持有 `Cube`，通过 `Cube::apply_notation()` 或 `Cube::apply_canonical_sequence()` 驱动状态变化，通过 `Cube::to_facelets()` 驱动渲染。若后续需要动画、逐步回放或公式展示，应优先依赖原始输入文本或 `ExtMoveSequence`，而不是尝试从 canonical `MoveSequence` 反推用户记法。

**Tech Stack:** Rust，依赖 `rubiks-core`，并允许为命令行解析与 REPL 引入少量工具型第三方依赖，如 `clap`、`rustyline`

## 首版范围

首版只实现：

- `rubiks new`
- `rubiks apply "<notation>"`
- `rubiks scramble [length]`
- `rubiks repl`

REPL 内置命令：

- `reset`
- `history`
- `show`
- `validate`
- `scramble [length]`
- `help`

当前仍不实现：

- `solve`
- `alg list`

其中当前 `scramble` 使用的是 `rubiks-alg::TrainingFaceTurn`，还不是 `RandomState3x3`。

## 依赖关系

```text
rubiks-cli → rubiks-core, rubiks-alg
```

不依赖：

- `rubiks-solver`

## 建议模块结构

```text
crates/rubiks-cli/src/
├── main.rs
├── render.rs
└── repl.rs
```

## 子命令设计

### `rubiks new`

行为：

- 创建还原状态 `Cube::solved()`
- 渲染并输出 ASCII 展开图

### `rubiks apply "<notation>"`

行为：

- 从还原状态开始
- 应用一段扩展记法
- 输出最终 ASCII 展开图

用途：

- 快速验证 `rubiks-core` 的 parser、orientation 与渲染链路

### `rubiks repl`

行为：

- 启动交互模式
- 维护一个 `ReplState`
- 每次输入后渲染当前魔方

### `rubiks scramble [length]`

行为：

- 使用 `TrainingScrambleGenerator`
- 默认长度为 `25`
- 输出 `scramble: ...`
- 从 solved 状态应用这串 canonical moves
- 渲染打乱后的 ASCII 展开图

说明：

- 这是训练用随机面转序列
- 当前不等价于 WCA / TNoodle 的随机状态打乱

## REPL 设计

### 状态对象

```rust
pub struct ReplState {
    pub cube: Cube,
    pub history: Vec<String>,
}
```

### `handle_input()` 语义

普通输入：

- 视为一段扩展记法
- 调用 `cube.apply_notation()`
- 成功后记录到 `history`

说明：

- `history` 首版可直接保留原始输入文本
- 如果后续需要逐步动画、公式高亮或语义化回放，可进一步保留解析后的 `ExtMoveSequence`

内置命令：

- `reset`：回到还原状态并清空历史
- `history`：显示输入历史
- `show`：重新渲染当前状态
- `validate`：对 `cube.validate()` 的结果进行显示
- `scramble [length]`：重置到 solved，生成并应用训练用随机打乱，清空旧 history 后记录新 scramble
- `help`：显示帮助

建议 API：

```rust
pub enum ReplEvent {
    Render,
    Print(String),
    PrintAndRender(String),
    Exit,
}

pub enum ReplError {
    InvalidCommand(String),
    InvalidNotation(String),
}

pub fn handle_input(&mut self, line: &str) -> Result<ReplEvent, ReplError>;
```

### 交互约束

- REPL 里只操作 `Cube`
- 不直接暴露 `CubeState` 给用户
- 当前只接入训练用 `scramble`
- 不做 solver 或公式浏览入口

## ASCII 渲染

### 输出来源

- 输入：`Cube::to_facelets()`
- 输出：终端可读的展开图字符串

### 建议提供两个版本

- `ascii(cube: &Cube) -> String`
- `ascii_plain(cube: &Cube) -> String`

用途：

- `ascii()` 用于终端彩色输出
- `ascii_plain()` 用于测试和非 ANSI 环境

### 布局

```text
        U U U
        U U U
        U U U
L L L   F F F   R R R   B B B
L L L   F F F   R R R   B B B
L L L   F F F   R R R   B B B
        D D D
        D D D
        D D D
```

## 任务拆分

### Task 1：初始化 crate

**Files:**

- `crates/rubiks-cli/Cargo.toml`
- `crates/rubiks-cli/src/main.rs`
- `crates/rubiks-cli/src/render.rs`
- `crates/rubiks-cli/src/repl.rs`

依赖：

- `rubiks-core`
- `rubiks-alg`

### Task 2：实现渲染模块

**Files:**

- `crates/rubiks-cli/src/render.rs`

内容：

- `ascii_plain()`
- 可选的 ANSI 彩色 `ascii()`

要求：

- solved 状态下输出布局稳定
- 测试不依赖 ANSI 色值

### Task 3：实现 `new` 与 `apply` 子命令

**Files:**

- `crates/rubiks-cli/src/main.rs`

内容：

- `new`
- `apply`
- `scramble`

说明：

- `apply` 直接从 solved cube 出发
- `scramble` 通过 `rubiks-alg` 生成训练用随机面转序列

### Task 4：实现 REPL 状态机

**Files:**

- `crates/rubiks-cli/src/repl.rs`

内容：

- `ReplState`
- `handle_input()`
- `run()`

要求：

- 将“读取用户输入”和“处理命令”分离
- `handle_input()` 返回带语义的事件类型，而不是直接返回渲染后的字符串
- 核心逻辑可在单元测试中直接调用

### Task 5：端到端验证

验证流程：

- `cargo run -p rubiks-cli -- new`
- `cargo run -p rubiks-cli -- apply "R U R' U'"`
- `cargo run -p rubiks-cli -- scramble`
- `cargo run -p rubiks-cli -- repl`

## 未来扩展预留

后续仍可继续新增：

- `rubiks solve`
- `rubiks alg list`

那时再决定：

- 子命令命名
- 输出格式
- phase 展示方式

当前仍建议避免把训练用 `scramble` 误写成标准随机状态打乱。

## 完成标准

- `cargo build -p rubiks-cli` 无错误
- `cargo test -p rubiks-cli` 全部通过
- `cargo clippy -p rubiks-cli` 无警告
- `rubiks new`、`rubiks apply` 和 `rubiks scramble` 可正常运行
- REPL 可正确响应 move 输入与内置命令，包括 `scramble [length]`
