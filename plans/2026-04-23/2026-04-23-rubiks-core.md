# rubiks-core V1 Implementation Plan

**Goal:** 实现首版魔方引擎核心 crate，包含 canonical 状态表示、状态合法性校验、基础面转、扩展记法解析、朝向追踪与 facelet 输出。

**Architecture:** 使用 Kociemba cubie 模型存储角块/棱块的排列与朝向。`CubeState` 负责 canonical 状态；`Cube` 负责物理朝向与用户记法。`MoveSequence` 是 canonical 执行 IR，`ExtMoveSequence` 是保留用户输入语义的记法 IR。当前阶段不引入 solver 坐标表、剪枝表或求解逻辑。

**Tech Stack:** Rust（无第三方依赖）

## 核心设计约束

### 1. canonical 与 physical 语义必须分开

`CubeState`：

- 只接受 canonical 基础面转
- 只暴露 `Move` / `MoveSequence`
- 未来供 `rubiks-alg` / `rubiks-solver` 使用
- 是整个引擎的领域核心数据结构

`Cube`：

- 面向 CLI / REPL / GUI
- 处理扩展记法与整魔方朝向
- 不暴露语义模糊的 `apply_sequence()`
- 不是新的领域核心，只是 `CubeState + Orientation` 的受控包装

扩展记法相关逻辑采用两阶段模型：

- `parse_notation()` 只解析用户写了什么，输出 lossless 的 `ExtMoveSequence`
- `resolve_notation()` 再结合当前 `Orientation`，把扩展记法翻译为 canonical 执行计划

### 2. `CubeState` 使用私有字段与受控构造

`CubeState` 的内部数组不应直接对外暴露。

公共构造入口只保留：

- `CubeState::solved()`
- `CubeState::try_from_parts(parts)`

建议配套提供：

```rust
pub struct CubeStateParts {
    pub corner_perm: [u8; 8],
    pub corner_orient: [u8; 8],
    pub edge_perm: [u8; 12],
    pub edge_orient: [u8; 12],
}
```

语义约束：

- 外部调用者通过 `try_from_parts()` 构造任意状态
- `try_from_parts()` 必须先经过 `validate()`
- 如实现内部确实需要跳过校验，可提供 `pub(crate)` 的 unchecked 构造，但它不属于公共契约

### 3. `validate()` 是正式公共契约

必须提供：

```rust
pub fn validate(&self) -> Result<(), CubeStateError>;
pub fn is_valid(&self) -> bool;
```

覆盖内容至少包括：

- corner orientation 取值范围
- edge orientation 取值范围
- corner permutation 合法性
- edge permutation 合法性
- corner orientation sum mod 3
- edge orientation sum mod 2
- corner / edge parity 一致性

### 4. 朝向表使用“生成到 `src/`”的落地方式

正式代码中，`tools/gen_orientation_tables.py` 是朝向表的唯一权威来源。  
它应生成：

- `tools/gen_orientation_tables.py`
- 输出：`crates/rubiks-core/src/orientation_tables.rs`

约束：

- `crates/rubiks-core/src/orientation_tables.rs` 是提交到仓库的生成文件
- 构建与测试直接依赖 `crates/rubiks-core/src/orientation_tables.rs`
- 不在 `cargo build` 过程中动态生成朝向表
- CI 应重新运行生成器，并检查生成后的 `orientation_tables.rs` 没有差异

## 目标 API

### `CubeState`

```rust
pub struct CubeState {
    corner_perm:   [u8; 8],
    corner_orient: [u8; 8],
    edge_perm:     [u8; 12],
    edge_orient:   [u8; 12],
}

pub struct CubeStateParts {
    pub corner_perm: [u8; 8],
    pub corner_orient: [u8; 8],
    pub edge_perm: [u8; 12],
    pub edge_orient: [u8; 12],
}

impl CubeState {
    pub fn solved() -> Self;
    pub fn try_from_parts(parts: CubeStateParts) -> Result<Self, CubeStateError>;
    pub fn parts(&self) -> CubeStateParts;
    pub fn is_solved(&self) -> bool;
    pub fn reset(&mut self);

    pub fn validate(&self) -> Result<(), CubeStateError>;
    pub fn is_valid(&self) -> bool;

    pub fn apply_move(&mut self, mv: Move);
    pub fn apply_sequence(&mut self, seq: &MoveSequence);

    pub fn to_facelets(&self) -> [[Color; 9]; 6];
}
```

### move 相关类型

```rust
pub enum Face { U, R, F, D, L, B }
pub enum Direction { CW, CCW, Double }

pub struct Move {
    pub face: Face,
    pub dir: Direction,
}

pub struct MoveSequence(pub Vec<Move>);

pub struct ExtMoveSequence(pub Vec<ExtMove>);
```

### 扩展记法与朝向

```rust
pub enum Slice { M, S, E }
pub enum Axis { X, Y, Z }

pub enum ExtMove {
    Face(Face, Direction),
    Slice(Slice, Direction),
    Rotation(Axis, Direction),
    Wide(Face, Direction),
}

pub struct Orientation(u8);

pub fn parse_canonical_notation(
    input: &str,
) -> Result<MoveSequence, ParseError>;

pub fn parse_notation(
    input: &str,
) -> Result<ExtMoveSequence, ParseError>;

pub struct ResolvedStep {
    pub input: ExtMove,
    pub canonical: Vec<Move>,
    pub orientation_before: Orientation,
    pub orientation_after: Orientation,
}

pub struct ResolvedSequence {
    pub source: ExtMoveSequence,
    pub steps: Vec<ResolvedStep>,
    pub flattened: MoveSequence,
    pub final_orientation: Orientation,
}

pub fn resolve_notation(
    seq: &ExtMoveSequence,
    orientation: Orientation,
) -> ResolvedSequence;
```

说明：

- `parse_notation()` 只负责保留用户输入语义，不处理朝向
- `parse_canonical_notation()` 只解析 canonical 记法文本，输出 `MoveSequence`
- `resolve_notation()` 负责结合当前 `Orientation` 生成 canonical 执行结果；对于合法的 `ExtMoveSequence` 它不返回错误
- UI、REPL history、公式展示优先使用 `ExtMoveSequence`
- `CubeState`、solver、差分测试只使用 `MoveSequence`

### `Cube`

```rust
pub struct Cube {
    state: CubeState,
    orientation: Orientation,
}

impl Cube {
    pub fn solved() -> Self;
    pub fn from_state(state: CubeState) -> Self;
    pub fn from_state_with_orientation(
        state: CubeState,
        orientation: Orientation,
    ) -> Self;

    pub fn state(&self) -> &CubeState;
    pub fn orientation(&self) -> Orientation;
    pub fn into_state(self) -> CubeState;

    pub fn reset(&mut self);
    pub fn reset_orientation(&mut self);
    pub fn is_solved(&self) -> bool;
    pub fn validate(&self) -> Result<(), CubeStateError>;
    pub fn is_valid(&self) -> bool;

    pub fn apply_canonical_sequence(&mut self, seq: &MoveSequence);
    pub fn apply_ext_sequence(&mut self, seq: &ExtMoveSequence);
    pub fn apply_canonical_notation(&mut self, input: &str) -> Result<(), ParseError>;
    pub fn apply_notation(&mut self, input: &str) -> Result<(), ParseError>;
    pub fn to_facelets(&self) -> [[Color; 9]; 6];
}
```

语义约束：

- `CubeState` 才是领域核心；solver / alg / 差分测试应直接依赖 `CubeState`
- `Cube` 不公开暴露内部字段，也不提供通用 `state_mut()`
- `Cube` 不暴露语义模糊的 `apply_sequence()`，而是显式区分 `apply_canonical_sequence()` 与 `apply_ext_sequence()`
- `Cube::validate()` / `Cube::is_valid()` 只是委托给内部 `CubeState`
- `Cube::is_solved()` 只反映 cubie 状态，和 `orientation` 无关；纯 `x/y/z` 旋转不会让已解状态变成未解
- `Cube::from_state()` 默认使用 `Orientation::SOLVED`
- `Cube::into_state()` 用于把交互层对象显式降回核心状态
- 如需修改核心状态，应通过 `CubeState` 自身 API，或显式替换整个 `CubeState`
- `Cube::apply_canonical_sequence()` 只提交 canonical `MoveSequence`，不读取也不更新 `orientation`
- `Cube::apply_canonical_*()` 属于 canonical frame 下的高级入口，不承诺等价于“当前物理视角下用户看到的那一面被转动”；面向用户输入时应优先使用 `apply_ext_*()`
- `Cube::apply_ext_sequence()` 负责对 `ExtMoveSequence` 调用 `resolve_notation()`，然后提交 canonical moves 并更新 `orientation`；它不返回错误
- `Cube::apply_canonical_notation()` 只解析 canonical 记法文本，然后调用 `apply_canonical_sequence()`
- `Cube::apply_notation()` 的内部流程是 `parse_notation() -> apply_ext_sequence()`；`ParseError` 只来自文本解析阶段

## 推荐模块结构

```text
crates/rubiks-core/src/
├── lib.rs
├── cube.rs
├── moves.rs
├── color.rs
├── notation.rs
├── orientation.rs
├── orientation_tables.rs   # 由脚本生成
└── oriented.rs
```

## 任务拆分

### Task 1：初始化 crate 与模块骨架

**Files:**

- `Cargo.toml`
- `crates/rubiks-core/Cargo.toml`
- `crates/rubiks-core/src/lib.rs`

要求：

- workspace 首版只包含 `rubiks-core` 和 `rubiks-cli`
- `rubiks-core` 不依赖其他 crate

### Task 2：基础枚举与 `CubeState`

**Files:**

- `crates/rubiks-core/src/cube.rs`
- `crates/rubiks-core/src/moves.rs`

内容：

- `Corner` / `Edge`
- `Face` / `Direction`
- `CubeState::solved()`
- `CubeState::try_from_parts()`
- `CubeState::parts()`
- `is_solved()`
- `reset()`

### Task 3：`validate()` 与错误类型

**Files:**

- `crates/rubiks-core/src/cube.rs`

内容：

- `CubeStateError`
- `validate()`
- `is_valid()`

这一任务必须在未来任何 solver 设计之前稳定下来。

### Task 4：`Move` / `MoveSequence` / `ExtMoveSequence`

**Files:**

- `crates/rubiks-core/src/moves.rs`
- `crates/rubiks-core/src/cube.rs`

内容：

- `Move::inverse()`
- `MoveSequence::inverse()`
- `MoveSequence::len()`
- `MoveSequence::to_notation()`
- `ExtMoveSequence::len()`
- `ExtMoveSequence::to_notation()`
- `CubeState::apply_sequence()`

说明：

- `mirror()` 暂不放入首版必须项
- 如果后续确实需要，再基于明确镜像语义单独设计

### Task 5：基础面转与 move tables

**Files:**

- `crates/rubiks-core/src/cube.rs`

内容：

- `CubieMove`
- `MOVE_6`
- `CubeState::apply_move()`

要求：

- 首版允许通过重复应用 CW 基础转处理 `Double` / `CCW`
- 未来若需要性能优化，再加 `MOVE_18`

### Task 6：颜色与 `to_facelets()`

**Files:**

- `crates/rubiks-core/src/color.rs`
- `crates/rubiks-core/src/cube.rs`

内容：

- `Color`
- `CubeState::to_facelets()`

约束：

- 返回 canonical frame 下的 `[[Color; 9]; 6]`
- 面顺序固定为 `U, R, F, D, L, B`

### Task 7：生成朝向表并实现 `Orientation`

**Files:**

- `tools/gen_orientation_tables.py`
- `crates/rubiks-core/src/orientation_tables.rs`
- `crates/rubiks-core/src/orientation.rs`

内容：

- `ORIENTATION_FRAMES`
- `FACE_REMAP`
- `FACE_DIR_FLIP`
- `ROTATION_TABLE`
- `FACELET_REMAP`
- `Orientation::after_rotation()`
- `Orientation::remap_move()`
- `Orientation::remap_facelets()`

要求：

- `Orientation::SOLVED = Orientation(0)`
- `FACE_DIR_FLIP` 当前应全为 `false`
- `tools/gen_orientation_tables.py` 是唯一权威来源
- `crates/rubiks-core/src/orientation_tables.rs` 必须提交到仓库
- CI 应重新运行生成器，并检查 `orientation_tables.rs` 无差异

### Task 8：扩展记法解析

**Files:**

- `crates/rubiks-core/src/notation.rs`

内容：

- `ParseError`
- `parse_token()`
- `parse_canonical_notation()`
- `parse_notation()`
- `resolve_ext_move()`
- `resolve_notation()`

要求：

- 在本任务开始前，`Orientation` 必须已完成
- `parse_canonical_notation()` 只接受 canonical 记法，不产生 `ExtMoveSequence`
- `parse_notation()` 输出必须保持用户输入语义，不提前降级为 canonical moves
- `resolve_notation()` 输出必须始终是 canonical `MoveSequence`
- `resolve_notation()` 只负责编译 `ExtMoveSequence`，不直接修改 `CubeState`
- `resolve_notation()` 对合法的 `ExtMoveSequence` 是总函数，不返回 `Result`
- 测试中必须使用 `Orientation::SOLVED`，不要使用裸整数

### Task 9：实现 `Cube`

**Files:**

- `crates/rubiks-core/src/oriented.rs`

内容：

- `Cube::solved()`
- `Cube::apply_canonical_sequence()`
- `Cube::apply_ext_sequence()`
- `Cube::apply_canonical_notation()`
- `Cube::apply_notation()`
- `Cube::to_facelets()`
- `Cube::reset()`
- `Cube::is_solved()`
- `Cube::validate()`
- `Cube::is_valid()`

要求：

- `Cube::apply_canonical_sequence()` 直接委托给内部 `CubeState::apply_sequence()`
- `Cube::apply_ext_sequence()` 接收已解析的 `ExtMoveSequence`，内部通过 `resolve_notation()` 驱动，且不返回错误
- `Cube::apply_canonical_notation()` 只接受 canonical 记法文本，不读取当前 `orientation`
- `Cube::apply_notation()` 通过 `parse_notation()` + `apply_ext_sequence()` 驱动
- `Cube` 不暴露语义模糊的 `apply_sequence()`
- `Cube` 的 `state` / `orientation` 字段保持私有，只暴露只读访问器与受控构造
- `Cube::is_solved()` 只取决于 `state`，不取决于 `orientation`

### Task 10：测试与差分校验

建议测试覆盖：

- `validate()` 的各类非法状态
- 每个基本面转四次回到初始状态
- move + inverse 回到初始状态
- solved `to_facelets()` 输出正确配色
- `parse_notation()` 的 round-trip
- `parse_canonical_notation("R U R' U'")` 直接输出对应 `MoveSequence`
- `parse_notation("r")` 保留为 `ExtMove::Wide(R, ...)`
- `resolve_notation()` 能把 `r` / `M` / `x` 正确展开为 canonical moves 和 orientation 变化
- `Cube::apply_canonical_sequence()` 不改变 `orientation`
- `Cube::apply_ext_sequence()` 对合法 `ExtMoveSequence` 不产生错误，并正确更新 `state` / `orientation`
- `Cube::apply_canonical_notation("R U R' U'")` 与直接解析后的 `MoveSequence` 执行结果一致
- `Cube::apply_notation("x")` 后 orientation 更新正确
- `Cube::apply_notation("x")` 后 `is_solved() == true`
- M/S/E / 宽转的分解等价性
- orientation 表各行是合法排列
- x/y/z 能生成全部 24 个朝向

如果条件允许，增加与 `ref/kociemba/` 的差分测试：

- 随机序列执行后的 cubie 状态一致
- `to_facelets()` 与参考输出一致

## 完成标准

- `cargo test -p rubiks-core` 全部通过
- `cargo clippy -p rubiks-core` 无警告
- `CubeState` / `Cube` 的 canonical / physical 语义边界清晰
- `parse_notation()` 能无损保留扩展记法语义
- `resolve_notation()` 正确把扩展记法翻译为 canonical `MoveSequence`
- CLI 可直接基于 `rubiks-core` 实现渲染与交互
