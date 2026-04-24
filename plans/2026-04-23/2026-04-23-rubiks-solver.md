# rubiks-solver Implementation Plan

**Status:** 进入文档先行的实现设计阶段  
**Current Focus:** 冻结公共类型与 `KociembaSolver` 的首批实现计划  
**Non-goal for this document:** 当前不实现 CFOP、通用 IDA* 搜索层、`SolverRegistry` 或 CLI `solve` 命令

## 当前结论

`rubiks-solver` 不再只保留“未来接口方向”。

从本文件开始，Phase D 的第一里程碑固定为：

- 先冻结公共求解接口
- 先实现一个可运行的 `KociembaSolver`
- 暂不实现 CFOP
- 暂不设计通用搜索框架

原因：

- 需要先验证 `solver` 的公共接口是否真的适合“分阶段 solver”和“平铺 solver”同时使用
- `KociembaSolver` 更适合作为第一个接口校验器，因为它天然返回一段完整解，而不是依赖人为 phase 拆分
- 过早实现 CFOP 会把文档和代码都绑到 `cross / f2l` 搜索设计上

## 职责边界

`rubiks-solver` 当前阶段负责：

- 求解器公共类型
- `Solver` trait
- `Solution` / `SolvePhase` / `SolveError`
- `SolveOptions`
- `KociembaSolver`
- `CubeState` 与 Kociemba cubestring 之间的转换
- 求解结果的解析、验证与错误映射

`rubiks-solver` 当前阶段不负责：

- CFOP
- LBL
- 通用 IDA* 抽象
- `SolverRegistry`
- CLI 命令接入
- OLL / PLL catalog 调度

## 依赖关系

当前第一里程碑建议依赖图为：

```text
rubiks-solver → rubiks-core
```

说明：

- `KociembaSolver` 的首批实现不需要 `rubiks-alg`
- 若未来开始实现 CFOP，并接入 OLL / PLL case lookup，再把依赖扩展为：

```text
rubiks-solver → rubiks-core, rubiks-alg
```

这样做的目的，是先让 `solver` 公共接口接受一个不依赖 catalog 的真实 solver 校验。

## 设计原则

### 1. solver 只操作 `CubeState`

solver 的输入输出固定为：

- 输入：`CubeState`
- 输出：canonical `MoveSequence`

而不是 `Cube`。

`Cube` 属于用户交互层；`solver` 不应感知物理朝向、宽转或旋转记法。

### 2. 公共接口必须同时容纳 phase solver 与 flat solver

CFOP 天然适合 `Cross / F2L / OLL / PLL` 形式的 phase 输出。  
Kociemba 更适合直接返回一段完整解。

因此 `Solution` 的正式语义固定为：

- `moves` 是最终权威执行结果
- `phases` 是可选诊断信息
- 没有稳定 phase 信息的 solver 可以返回空 `phases`

这意味着公共接口不能只靠 `Vec<SolvePhase>` 表达完整结果。

### 3. 不过早冻结共享搜索抽象

当前阶段不实现：

- 通用 IDA* trait
- 通用剪枝表框架
- 通用 phase 管线

只有当项目里至少出现第二个真实搜索型 solver 时，才重新评估是否值得抽象出共享搜索层。

### 4. Kociemba 只冻结 solver 边界，不冻结单一后端实现

`KociembaSolver` 的公共行为应当稳定，但它的具体后端可以是：

- 基于 C 实现的 `InProcess` 后端
- 基于可执行文件的 `ExternalProcess` 后端

当前文档明确同时支持两种后端，但默认工程方向是：

- 长期默认后端：`InProcess(C)`
- 开发期 fallback / 对拍后端：`ExternalProcess`

## 公共类型

### `SolveOptions`

```rust
pub struct SolveOptions {
    pub timeout: Option<std::time::Duration>,
    pub max_nodes: Option<u64>,
    pub diagnostics: bool,
}
```

语义约束：

- `timeout` 是求解预算，不承诺底层一定能强制中断，但 solver 必须尽量尊重
- `max_nodes` 为未来搜索型 solver 预留；`KociembaSolver` 可以忽略它
- `diagnostics` 表示调用方希望拿到尽可能完整的阶段与后端信息；忽略它不应导致求解失败

说明：

- 相比只定义 `solve(&self, cube: &CubeState)`，这里显式加入 `SolveOptions`，以避免 `KociembaSolver` 落地后再反向破坏接口

### `SolveError`

```rust
pub enum SolveError {
    InvalidState(CubeStateError),
    Unsolvable,
    ExhaustedBudget,
    BackendUnavailable {
        solver: &'static str,
        backend: &'static str,
    },
    BackendFailure {
        solver: &'static str,
        message: String,
    },
    Unsupported {
        solver: &'static str,
        feature: &'static str,
    },
}
```

语义约束：

- `InvalidState` 表示输入 `CubeState` 未通过 `validate()`
- `Unsolvable` 保留给未来更一般的求解语义；对于合法的 3x3 状态，它通常不应出现
- `ExhaustedBudget` 表示超时、深度上限或其他预算耗尽
- `BackendUnavailable` 表示当前配置的后端不存在或不可调用
- `BackendFailure` 表示后端返回了无法映射为正常解的错误
- `Unsupported` 表示某 solver 明确不支持某项能力

### `SolvePhase`

```rust
pub struct SolvePhase {
    pub name: &'static str,
    pub moves: MoveSequence,
}

impl SolvePhase {
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### `Solution`

```rust
pub struct Solution {
    pub solver_name: &'static str,
    pub moves: MoveSequence,
    pub phases: Vec<SolvePhase>,
}

impl Solution {
    pub fn total_moves(&self) -> &MoveSequence;
    pub fn total_len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

语义约束：

- `moves` 是正式求解结果
- `phases` 只用于附加结构化信息
- 若 `phases` 非空，它们按顺序拼接后必须与 `moves` 完全一致
- 若 solver 没有稳定 phase 语义，允许返回空 `phases`

### `Solver`

```rust
pub trait Solver {
    fn id(&self) -> &'static str;

    fn solve(
        &self,
        cube: &CubeState,
        options: &SolveOptions,
    ) -> Result<Solution, SolveError>;
}
```

约束：

- `solve()` 必须先验证输入状态
- 返回的解必须是 canonical `MoveSequence`
- 回放 `Solution::moves` 后必须得到 solved state
- 不允许依赖 `Cube` 的朝向语义

## `KociembaSolver`

### 目标

`KociembaSolver` 是 `rubiks-solver` 的第一个真实 solver，用来验证：

- 公共接口是否足够表达非 phase-first solver
- `CubeState -> facelets -> 两阶段后端 -> MoveSequence` 这条链路是否稳定
- solver 错误模型是否足够承载真实后端故障

### 公共类型

```rust
pub struct KociembaConfig {
    pub max_depth: u8,
    pub backend: KociembaBackendConfig,
}

pub enum KociembaBackendConfig {
    InProcess,
    ExternalProcess {
        program: std::path::PathBuf,
        args: Vec<String>,
    },
}

pub struct KociembaSolver {
    /* private */
}

impl KociembaSolver {
    pub fn new(config: KociembaConfig) -> Self;
}

impl Solver for KociembaSolver { /* ... */ }
```

说明：

- 当前不提供 `Auto` 后端选择，避免把后端探测与许可证策略悄悄写进默认行为
- `KociembaSolver::new()` 可以是无错误构造；后端可用性检查放在 `solve()` 时完成
- `InProcess` 指向 C 版本 Kociemba 内核
- `ExternalProcess` 可以指向 C CLI、Python CLI，或任意兼容 cubestring-in / move-text-out 协议的程序

### 内部后端边界

实现层建议保留一个非公共后端 trait：

```rust
trait TwoPhaseBackend {
    fn id(&self) -> &'static str;

    fn solve_facelets(
        &self,
        cubestring: &str,
        max_depth: u8,
        timeout: Option<std::time::Duration>,
    ) -> Result<String, BackendError>;
}
```

这样做的目的：

- 不把 C FFI 绑定写进公共接口
- 不把外部进程协议写进公共接口
- 便于同一套 `KociembaSolver` 对拍多个后端

补充约束：

- `InProcess` 与 `ExternalProcess` 都必须实现这同一个后端边界
- `KociembaSolver` 不感知底层到底是 C 函数调用还是子进程调用

### facelet 转换

`KociembaSolver` 不直接读取 cubie 数组，而是复用 `CubeState::to_facelets()`。

固定转换步骤：

1. 调用 `CubeState::to_facelets()`
2. 按 `U, R, F, D, L, B` 的 face 顺序展开 54 个 facelet
3. 每个 `Color` 使用 `U/R/F/D/L/B` 单字符输出
4. 生成 54 字符 cubestring

约束：

- cubestring 的顺序必须与 Kociemba 实现要求的顺序一致
- 不能通过 `Cube` 或 orientation 做额外重映射
- solved 状态必须生成：

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

### 求解流程

`KociembaSolver::solve()` 的固定流程建议为：

1. `cube.validate()`，失败则返回 `SolveError::InvalidState`
2. 将 `CubeState` 转成 cubestring
3. 调用配置的 Kociemba backend
4. 用 `parse_canonical_notation()` 解析返回结果
5. 将解析出的 `MoveSequence` 回放到输入状态副本上
6. 若未 solved，则返回 `SolveError::BackendFailure`
7. 构造 `Solution`

构造结果时：

- `solver_name` 固定为 `"kociemba"`
- `moves` 使用完整求解序列
- `phases` 当前建议返回空数组

说明：

- 当前不从 Kociemba 返回结果中反推出 “phase 1 / phase 2”
- 若未来某 backend 能稳定暴露 phase 边界，再评估是否补充 `phases`

### 错误映射

建议映射规则：

- backend 不存在、不可执行、动态库不可加载：
  - `SolveError::BackendUnavailable`
- backend 报超时或深度不足：
  - `SolveError::ExhaustedBudget`
- backend 返回格式非法或返回的解无法回放为 solved：
  - `SolveError::BackendFailure`
- 输入状态未通过 `CubeState::validate()`：
  - `SolveError::InvalidState`

说明：

- `ExternalProcess` 后端更容易保留原始错误文本
- `InProcess(C)` 统一通过一层薄 C 包装暴露更细粒度的状态码
- 两种后端都应尽量映射到一致的 solver 错误语义

## 两种后端的正式定位

### `InProcess(C)`

这是 `KociembaSolver` 的长期默认后端。

原因：

- 部署更简单
- 进程模型更稳定
- 没有额外进程启动和文本协议开销
- 更适合作为 CLI / REPL / GUI / benchmark 共用的默认求解路径

当前参考来源：

- `ref/kociemba/kociemba/ckociemba/include/search.h`
- `ref/kociemba/kociemba/ckociemba/search.c`

其中最关键的可复用边界是：

```c
char* solution(char* facelets, int maxDepth, long timeOut, int useSeparator, const char* cache_dir);
```

当前文档采用的实现方向是：

- 直接复用 C 版本两阶段求解内核
- 在 Rust 侧通过 FFI 调用一个稳定 C 接口
- Rust 负责：
  - `CubeState -> cubestring`
  - 调用 C 接口
  - 将返回文本解析为 `MoveSequence`
  - 回放验证

#### `InProcess(C)` 建议接口

当前实现方案已经固定为：不直接从 Rust FFI 调用 `search.c` 的原始 `solution(...)`，而是在 `rubiks-solver` 自己控制的薄包装层上定义更稳定的 C ABI。

推荐形态：

```c
typedef struct {
    int status;
    char* solution_text;
} rce_kociemba_result_t;

rce_kociemba_result_t rce_kociemba_solve(
    const char* facelets,
    int max_depth,
    long timeout_seconds,
    const char* cache_dir
);

void rce_kociemba_free_string(char* ptr);
```

推荐这么做的原因：

- 原始 `solution(...)` 把多种失败路径都折叠成 `NULL`
- Rust 侧直接绑定原始函数会丢失错误细节
- 自己加一层薄包装后，可以把：
  - 输入非法
  - 超时
  - max depth 不足
  - 内部错误
  分开编码成 `status`

#### `InProcess(C)` Rust 侧结构

```rust
pub(crate) struct InProcessBackend {
    cache_dir: std::path::PathBuf,
}

impl TwoPhaseBackend for InProcessBackend {
    fn id(&self) -> &'static str {
        "kociemba-inprocess-c"
    }

    fn solve_facelets(
        &self,
        cubestring: &str,
        max_depth: u8,
        timeout: Option<std::time::Duration>,
    ) -> Result<String, BackendError>;
}
```

Rust 侧职责：

- 把 Rust `&str` 转成 C 字符串
- 调用 FFI
- 把返回指针转回 `String`
- 负责释放 C 分配的内存

约束：

- FFI 层不暴露到公共 API
- `unsafe` 代码只集中在一个很小的模块里
- `KociembaSolver` 不直接写任何 `unsafe`

#### cache / prune table 处理

C 版本 `solution(...)` 需要 `cache_dir` 参数。

当前文档建议：

- `InProcess(C)` 后端显式持有 `cache_dir`
- cache 路径作为 `InProcess` 后端的内部配置，不进入顶层 `Solver` trait
- 首版可以使用 `rubiks-solver` 控制的固定 cache 目录

待实现时需要再确认：

- cache 是仓库内预生成文件、首次运行生成，还是运行时按用户目录缓存

### `ExternalProcess`

这是 `KociembaSolver` 的通用外部后端，也是开发期 fallback 与对拍后端。

适用场景：

- 快速接入现成可执行文件
- 与 `InProcess(C)` 做差分测试
- 当 `InProcess(C)` 尚未完成或不可用时提供兜底路径

当前允许的后端来源包括：

- `ref/kociemba` 的 C CLI
- Python 包或脚本封装的 `kociemba`
- 用户自备的任意兼容程序

#### `ExternalProcess` 协议

为了让 `KociembaSolver` 不绑定某个具体程序，`ExternalProcess` 后端只要求一个最小协议：

- stdin 或 argv 输入：一个 cubestring
- stdout 输出：一行 canonical move text
- 出错时：非零退出码，或 stderr / stdout 中的错误文本

当前首版建议优先采用的调用约定是：

- 命令行参数传入 cubestring
- stdout 返回解文本

原因：

- 与 `ref/kociemba/kociemba/ckociemba/solve.c` 现有用法最接近
- 更容易快速对接 C CLI 与 Python CLI

#### `ExternalProcess` Rust 侧结构

```rust
pub(crate) struct ExternalProcessBackend {
    program: std::path::PathBuf,
    args: Vec<String>,
}

impl TwoPhaseBackend for ExternalProcessBackend {
    fn id(&self) -> &'static str {
        "kociemba-external-process"
    }

    fn solve_facelets(
        &self,
        cubestring: &str,
        max_depth: u8,
        timeout: Option<std::time::Duration>,
    ) -> Result<String, BackendError>;
}
```

实现约束：

- `max_depth` 与 `timeout` 若底层程序不支持，必须在文档和错误中明确
- `ExternalProcess` 必须采集退出码、stdout、stderr
- 返回文本必须在 Rust 侧再做 `trim`、解析和回放验证

#### Python 在 `ExternalProcess` 中的定位

当前文档将 Python 版本明确定位为：

- 外部进程参考实现
- 对拍基线
- 调试辅助后端

而不是：

- 长期默认 `InProcess` 路线

原因：

- 当前项目主语言是 Rust
- Python 进程内嵌入会引入更重的运行时和打包复杂度
- 若需要 `InProcess`，C 版本比 Python 更适合直接桥接

### 进程内库 vs 外部进程

当前文档不冻结二者之一为默认实现，只冻结以下工程结论：

- `KociembaSolver` 必须通过统一 backend 边界访问两阶段求解器
- `InProcess(C)` 是长期默认后端
- `ExternalProcess` 是开发期 fallback、对拍工具和用户自备后端入口
- 两种后端都必须满足相同的 cubestring-in / canonical-moves-out 契约

## 建议模块结构

```text
crates/solver/
├── rubiks-solver/
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       └── types.rs
└── rubiks-solver-kociemba/
    ├── build.rs
    ├── csrc/
    │   ├── rce_kociemba_wrapper.c
    │   └── rce_kociemba_wrapper.h
    └── src/
        ├── lib.rs
        ├── facelets.rs
        ├── kociemba.rs
        └── backend/
            ├── mod.rs
            ├── external_process.rs
            ├── in_process.rs
            └── ffi/
                ├── mod.rs
                └── kociemba_c.rs
```

说明：

- `rubiks-solver` 只放公共 trait / types / errors
- `rubiks-solver-kociemba` 放 Kociemba 的 Rust 实现、C 包装和构建脚本
- `backend/in_process.rs` 放 C 后端适配
- `backend/external_process.rs` 放子进程后端
- `backend/ffi/` 只收纳 `unsafe` 绑定代码

## 首批实现任务

### Task 1：初始化 crate 与公共类型

目标：

- 创建 `rubiks-solver` crate
- 定义 `SolveOptions`
- 定义 `SolveError`
- 定义 `SolvePhase`
- 定义 `Solution`
- 定义 `Solver` trait

要求：

- 公共类型先稳定下来
- 不同时引入 CFOP 或搜索型 solver 抽象

### Task 2：实现 cubestring 编码

目标：

- 从 `CubeState::to_facelets()` 生成 Kociemba 兼容 cubestring

要求：

- solved state 编码固定
- 编码只依赖 `rubiks-core`
- 不引入额外的魔方状态表示

### Task 3：实现 `KociembaSolver` 外壳

目标：

- 实现配置解析
- 实现统一求解流程
- 实现返回结果解析与回放校验

要求：

- 后端输出必须经过 `parse_canonical_notation()`
- 不信任后端输出，必须回放验证

### Task 4：实现至少一个 backend

目标：

- 落地 `InProcess(C)` 与 `ExternalProcess` 两个 backend

说明：

- 实现顺序允许先做 `ExternalProcess` 再做 `InProcess(C)`
- 但默认工程方向仍是让 `InProcess(C)` 成为长期主路径

### Task 5：实现 `InProcess(C)` FFI 包装

目标：

- 为 C 版本 Kociemba 建立稳定的 FFI 边界
- 控制内存分配、字符串释放与错误映射

要求：

- `unsafe` 范围最小化
- Rust 只绑定项目自己控制的薄包装 C ABI
- 不让 Rust 直接绑定未经包装的 `search.c` 原始返回边界

### Task 6：补充测试

目标：

- 验证 cubestring 映射
- 验证解序列解析
- 验证回放后 solved
- 验证错误映射
- 验证两种 backend 对同一状态都能返回有效解
- 若条件允许，验证两种 backend 的解在 solved 校验上一致

## 当前明确不做的事

- 不实现 CFOP
- 不实现 `cross / f2l` 搜索
- 不实现 OLL / PLL catalog 驱动
- 不实现 `SolverRegistry`
- 不实现通用 IDA*
- 不在当前阶段接入 CLI `solve`

## 测试标准

### 公共类型

- `Solution::total_len()` 与 `moves.len()` 一致
- 当 `phases` 非空时，按序拼接等于 `moves`
- `SolvePhase::len()` 与内部 move 数一致

### cubestring 编码

- solved state 编码精确等于标准 solved cubestring
- 对已知状态样例，编码结果与参考实现一致

### `KociembaSolver`

- 对给定样例状态能返回可执行解
- 回放 `Solution::moves` 后 `is_solved() == true`
- 非法状态返回 `SolveError::InvalidState`
- 后端不可用时返回 `SolveError::BackendUnavailable`
- 后端返回异常文本时返回 `SolveError::BackendFailure`
- `InProcess(C)` 与 `ExternalProcess` 都能通过同一套回放验证

### 差分测试

如果条件允许，建议保留与 `ref/kociemba/` 的差分测试，用于验证：

- facelet 编码
- 解文本解析
- solved 校验链路

## 待确认事项

### 1. flat solver 的 `phases` 是否允许为空

当前文档采用的方案是：

- `Solution.moves` 为权威结果
- `KociembaSolver` 返回空 `phases`

如果后续 CLI 明确要求“每个 solver 都必须有可展示阶段”，再讨论是否为 flat solver 补一个合成 phase。
