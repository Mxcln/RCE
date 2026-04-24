# rubiks-alg Implementation Design

**Status:** Phase C 已完成 `57 OLL + 21 PLL` 公式库、catalog lookup 与 CLI / REPL 公式展示；当前尚未完成 `RandomState3x3`  
**Goal:** 作为 `rubiks-alg` 的权威设计与进度文档，覆盖公式目录、OLL/PLL case 识别、数据治理、随机打乱接口以及当前实现状态。  
**Non-goal for this document:** 本文件不直接生成代码，而是记录已经冻结的设计、当前已落地实现和后续补完边界。

## 结论

`rubiks-alg` 不再视为“预留接口”。

从本文件开始，`rubiks-alg` 的设计目标是：

- 直接可实施
- 关键接口不再保留 `TBD`
- 数据格式、目录结构、匹配语义、错误模型、测试标准全部冻结

当前已经完成的事：

- 已创建 `rubiks-alg` crate 并加入 workspace
- 已实现 `Auf`、`OllPattern`、`PllPattern`
- 已实现 `Catalog`、`AlgCatalog`、`LoadError`、`LookupError`
- 已实现 vendored `TOML` loader，以及 `Catalog::embedded()` / `Catalog::from_dir(...)`
- 已实现 OLL / PLL pattern 提取、前置条件校验和 AUF 预展开索引
- 已录入 `57 OLL + 21 PLL` 全量 vendored `TOML`
- 已实现 `TrainingFaceTurn`
- 已接入 `rubiks-cli` 子命令 `scramble [length]` 与 REPL 内置命令 `scramble [length]`
- 已接入 `rubiks-cli alg list <oll|pll>` 与 `rubiks-cli alg show <oll|pll> <case_id>`
- 已接入 REPL 内置命令 `alg list <oll|pll>` 与 `alg show <oll|pll> <case_id>`
- 已提供共享的终端友好公式输出格式，包括对齐列表和简化详情页

当前仍未完成的事：

- `RandomState3x3` 仍返回 `ScrambleError::UnsupportedMode`
- 尚未与 `rubiks-solver` 对接

## 职责边界

`rubiks-alg` 的职责固定为：

- case catalog
- OLL / PLL pattern extraction
- OLL / PLL matcher
- 公式元数据与默认公式选择
- vendored 数据加载
- scramble 生成

`rubiks-alg` 不负责：

- cubie 状态执行
- 扩展记法解析本身
- 物理朝向跟踪
- solver 搜索
- CLI / REPL 展示

依赖关系固定为：

```text
rubiks-alg → rubiks-core
```

`rubiks-alg` 直接依赖：

- `CubeState`
- `CubeStateParts`
- `MoveSequence`
- `ExtMoveSequence`
- `parse_notation()`
- `resolve_notation(..., Orientation::SOLVED)`

## 设计原则

### 1. case-first

必须区分：

- case 定义
- 公式实现

识别流程固定为：

- 先从 `CubeState` 命中 case
- 再从 case 选择公式

禁止：

- 从算法文本反推唯一 case
- 运行时通过“试所有公式”识别 case

### 2. 数据与代码分离

公式库主数据不硬编码在 Rust 源码里。

固定方案：

- 仓库内 vendored `TOML` 文件是唯一权威来源
- Rust 代码只负责加载、校验和索引

允许的加载入口：

- 编译时 embed
- 运行时从 `data/` 目录读取

但无论使用哪种入口，都必须共享同一份 vendored 数据文件。

### 3. 原始记法与执行表示分离

公式主数据以原始文本记法持久化：

- `notation: String`

运行时由 catalog 负责派生：

- `ExtMoveSequence`
- canonical `MoveSequence`

这些派生结果可以缓存，但不作为主数据 source of truth。

### 4. AUF 是正式公共语义

`lookup_oll()` / `lookup_pll()` 命中结果中的顶层对齐信息不再抽象保留为 `CaseAlignment`，而是直接冻结为：

- `Auf`

`Auf` 的语义固定为：

- `Identity`
- `U`
- `U2`
- `UPrime`

并且它表示：

- 为了把**当前状态**对齐到该 case 的 canonical 视角，在执行公式前需要先应用的 U 层对齐
- 若状态满足前置条件但实际上是 `OLL skip` / `PLL skip`，lookup 应返回 `Ok(None)`，而不是伪造一个 case

## 公开类型与接口

本节定义的是稳定设计目标，不代表当前仓库已经包含这些类型。

```rust
pub enum AlgorithmSourceKind {
    Imported,
    Transcribed,
    Original,
}

pub struct AlgorithmSource {
    pub kind: AlgorithmSourceKind,
    pub name: String,
    pub url: Option<String>,
    pub license: Option<String>,
    pub retrieved_at: Option<String>,
    pub notes: Option<String>,
}

pub struct Auf(/* opaque newtype over 0..=3 */);

pub struct OllPattern(/* opaque */);
pub struct PllPattern(/* opaque */);

pub struct AlgEntry {
    pub id: String,
    pub display_name: String,
    pub notation: String,
    pub is_default: bool,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub post_auf: Option<Auf>,
    pub source: AlgorithmSource,
}

pub struct OllCase {
    pub case_id: String,
    pub display_name: String,
    pub canonical_pattern: OllPattern,
    pub algorithms: Vec<AlgEntry>,
}

pub struct PllCase {
    pub case_id: String,
    pub display_name: String,
    pub canonical_pattern: PllPattern,
    pub algorithms: Vec<AlgEntry>,
}

pub struct CaseMatch<'a, T> {
    pub case: &'a T,
    pub auf: Auf,
}

pub enum LookupKind {
    Oll,
    Pll,
}

pub enum LookupError {
    PrerequisiteNotMet {
        lookup: LookupKind,
        requirement: &'static str,
    },
    AmbiguousCase {
        lookup: LookupKind,
        pattern_debug: String,
    },
    CatalogInvariant {
        message: String,
    },
}

pub enum LoadError {
    Parse {
        path: String,
        message: String,
    },
    DuplicateCaseId {
        case_id: String,
    },
    DuplicateAlgorithmId {
        algorithm_id: String,
    },
    MissingDefaultAlgorithm {
        case_id: String,
    },
    MultipleDefaultAlgorithms {
        case_id: String,
    },
    InvalidPattern {
        case_id: String,
        detail: String,
    },
    InvalidNotation {
        algorithm_id: String,
        notation: String,
        detail: String,
    },
    PatternCollision {
        family: LookupKind,
        pattern_debug: String,
        existing_case_id: String,
        duplicate_case_id: String,
    },
}

pub trait AlgCatalog {
    fn lookup_oll(&self, cube: &CubeState) -> Result<CaseMatch<'_, OllCase>, LookupError>;
    fn lookup_pll(&self, cube: &CubeState) -> Result<CaseMatch<'_, PllCase>, LookupError>;
}
```

### catalog 入口

`rubiks-alg` 应提供一个具体 catalog 实现，命名固定为：

```rust
pub struct Catalog { /* private */ }

impl Catalog {
    pub fn embedded() -> Result<Self, LoadError>;
    pub fn from_dir(path: impl AsRef<std::path::Path>) -> Result<Self, LoadError>;
}
```

约束：

- `Catalog::embedded()` 读取编译时内嵌的 vendored TOML
- `Catalog::from_dir(...)` 读取外部目录中的同结构 TOML
- 两种入口必须共享同一套解析、校验与索引逻辑
- `Catalog` 应实现 `AlgCatalog`

### scramble 接口

```rust
pub enum ScrambleMode {
    RandomState3x3,
    TrainingFaceTurn { length: usize },
}

pub enum ScrambleError {
    UnsupportedMode {
        mode: ScrambleMode,
    },
    InvalidLength {
        length: usize,
    },
}

pub trait ScrambleGenerator {
    fn generate(&self, mode: ScrambleMode) -> Result<MoveSequence, ScrambleError>;

    fn generate_with_rng<R: rand::Rng + ?Sized>(
        &self,
        mode: ScrambleMode,
        rng: &mut R,
    ) -> Result<MoveSequence, ScrambleError>;
}
```

约束：

- 首批实现必须支持 `TrainingFaceTurn`
- 首批实现可以对 `RandomState3x3` 返回 `ScrambleError::UnsupportedMode`
- 文档中仍然冻结 `RandomState3x3` 的长期技术路线

## AUF 语义

`Auf` 的内部编码固定为：

- `0 = Identity`
- `1 = U`
- `2 = U2`
- `3 = UPrime`

建议提供的行为：

- `Auf::IDENTITY`
- `Auf::inverse()`
- `Auf::compose(rhs)`
- `Auf::to_move_sequence()`
- `Auf::to_notation()`

`CaseMatch::auf` 的解释固定为：

- 如果对当前状态先应用 `auf`
- 则观察到的顶层模式与该 case 的 canonical pattern 对齐

因此，如果公式按 canonical 视角存储，则从命中状态执行默认公式的基本拼接形式是：

- `auf + alg`
- 对 PLL 可进一步拼接 `post_auf`

`lookup` 的索引构建必须预展开全部 4 个 U 对齐：

- 运行时不枚举 AUF
- 运行时只做一次 pattern 提取和一次 map 查询

构建索引时的规则固定为：

- 从 canonical pattern 出发生成 4 个 AUF 旋转后的观察模式
- 每个观察模式都写入索引
- 存储的返回值不是该旋转本身，而是其逆 AUF，因为返回值语义是“对当前状态需要先做什么”

## Pattern 提取与前置条件

Pattern 提取只观察 canonical `CubeState`，不接触 `Cube` 或 `Orientation`。

### 位置顺序

所有 last-layer 模式提取都使用以下固定顺序：

- U 层角块位置：`URF, UFL, ULB, UBR`
- U 层棱块位置：`UR, UF, UL, UB`

这些顺序直接对应当前 `rubiks-core` 中的 `CubeStateParts` 下标：

- 角块位置 `0..=3`
- 棱块位置 `0..=3`

### OLL 前置条件

`lookup_oll()` 的前置条件固定为 `F2L solved`。

文档中把它具体定义为：

- 角块位置 `4..=7` 必须分别放置 cubie `4..=7`
- 棱块位置 `4..=11` 必须分别放置 cubie `4..=11`
- 上述位置的朝向都必须为 `0`
- 角块位置 `0..=3` 必须只包含 cubie `0..=3`
- 棱块位置 `0..=3` 必须只包含 cubie `0..=3`

若不满足，`lookup_oll()` 必须返回：

- `LookupError::PrerequisiteNotMet { lookup: LookupKind::Oll, requirement: "F2L solved" }`

### OLL pattern 编码

`OllPattern` 对外保持 opaque，但内部编码在本设计中冻结为 `u16`：

- `corner_code = co[0] + 3*co[1] + 9*co[2] + 27*co[3]`
- `edge_code = eo[0] + 2*eo[1] + 4*eo[2] + 8*eo[3]`
- `key = corner_code * 16 + edge_code`

其中：

- `co[i]` 直接使用 `CubeStateParts.corner_orient[i]`
- `eo[i]` 直接使用 `CubeStateParts.edge_orient[i]`

编码范围固定为：

- `0..1295`

### PLL 前置条件

`lookup_pll()` 的前置条件固定为 `OLL solved`。

文档中把它具体定义为：

- 满足全部 `F2L solved` 条件
- U 层角块位置 `0..=3` 的朝向全部为 `0`
- U 层棱块位置 `0..=3` 的朝向全部为 `0`

若不满足，`lookup_pll()` 必须返回：

- `LookupError::PrerequisiteNotMet { lookup: LookupKind::Pll, requirement: "OLL solved" }`

### PLL pattern 编码

`PllPattern` 对外保持 opaque，但内部编码在本设计中冻结为 `u16`。

内部步骤固定为：

1. 读取角块位置 `0..=3` 上的 cubie id，它们必须是 `0..=3` 的一个排列
2. 读取棱块位置 `0..=3` 上的 cubie id，它们必须是 `0..=3` 的一个排列
3. 分别把角块排列和棱块排列通过 Lehmer code 编码为 `0..23`
4. 合成：

```text
key = corner_rank * 24 + edge_rank
```

编码范围固定为：

- `0..575`

### catalog 不变量

在首批全量数据下：

- 任意满足 OLL 前置条件的状态都必须命中唯一 OLL case
- 任意满足 PLL 前置条件的状态都必须命中唯一 PLL case

如果 loader 构建索引时发现多个 case 经 AUF 预展开后落到同一 pattern key，必须失败，不允许静默覆盖。

## 数据文件设计

### 文件格式

主数据格式固定为：

- `TOML`

文件粒度固定为：

- 每个 case 一个文件

目录结构固定为：

```text
crates/rubiks-alg/data/
├── oll/
│   ├── OLL01.toml
│   ├── ...
│   └── OLL57.toml
└── pll/
    ├── Aa.toml
    ├── ...
    └── Z.toml
```

约束：

- 文件名去掉 `.toml` 后必须等于 `case_id`
- family 由目录名决定，不再单独写字段

### case_id 与展示名

case 标识风格固定为：

- OLL: `OLL01` .. `OLL57`
- PLL: `Aa`, `Ab`, `E`, `F`, `Ga`, `Gb`, `Gc`, `Gd`, `H`, `Ja`, `Jb`, `Na`, `Nb`, `Ra`, `Rb`, `T`, `Ua`, `Ub`, `V`, `Y`, `Z`

`display_name` 固定为显式字段：

- 不从 `case_id` 推导
- 不依赖运行时格式化

### TOML schema

每个 case 文件的顶层字段固定为：

- `case_id`
- `display_name`

必须包含：

- `[pattern]`
- `[[algorithms]]`

#### OLL 文件中的 `[pattern]`

```toml
[pattern]
corners = [0, 1, 2, 0]
edges = [0, 1, 0, 1]
```

说明：

- `corners` 顺序固定为 `URF, UFL, ULB, UBR`
- `edges` 顺序固定为 `UR, UF, UL, UB`
- 数值直接对应 `CubeStateParts.corner_orient` / `edge_orient`

#### PLL 文件中的 `[pattern]`

```toml
[pattern]
corners = ["URF", "UFL", "ULB", "UBR"]
edges = ["UR", "UF", "UL", "UB"]
```

说明：

- 数组表示“该位置上应出现哪个 U 层 cubie”
- 顺序同样固定为 `URF, UFL, ULB, UBR` 与 `UR, UF, UL, UB`

#### `[[algorithms]]`

每条公式必须显式包含：

- `id`
- `display_name`
- `notation`
- `is_default`
- `tags`

可选字段：

- `notes`
- `post_auf`

`post_auf` 的文本表示固定为：

- `"I"`
- `"U"`
- `"U2"`
- `"U'"`

其中：

- `is_default = true` 的算法在每个 case 中必须恰好一条
- 绝不通过数组顺序隐式决定默认公式

#### `[algorithms.source]`

每条算法都必须记录 provenance。

固定字段：

- `kind`
- `name`

可选字段：

- `url`
- `license`
- `retrieved_at`
- `notes`

约束：

- `kind` 只能是 `imported` / `transcribed` / `original`
- `retrieved_at` 使用 `YYYY-MM-DD`
- 对 `imported` / `transcribed`，应优先填写 `url` 与 `retrieved_at`
- 对 `original`，允许 `url` 与 `retrieved_at` 为空

### 示例

```toml
case_id = "Ua"
display_name = "Ua Perm"

[pattern]
corners = ["URF", "UFL", "ULB", "UBR"]
edges = ["UF", "UL", "UB", "UR"]

[[algorithms]]
id = "ua-main-01"
display_name = "Standard Ua"
notation = "R U' R U R U R U' R' U' R2"
is_default = true
tags = ["2-sided", "speed"]
post_auf = "I"
notes = "Default Ua algorithm"

[algorithms.source]
kind = "transcribed"
name = "SpeedCubeDB"
url = "https://www.speedcubedb.com/"
license = "unknown"
retrieved_at = "2026-04-24"
notes = "Manually transcribed from reference material"
```

## 数据来源治理

数据来源策略在本设计中固定为：

- 许可明确且兼容的数据，可以直接导入
- 许可不明确但内容可参考的网站，允许人工转录
- 不允许把外部网站设计成构建期或运行时依赖

因此：

- 自动下载不是实现路径
- 自动抓取不是实现路径
- 网络访问不是实现路径

当前已知来源策略固定为：

- `ellishg/rubiks-algorithms` 可作为许可明确的参考来源
- SpeedCubeDB 可作为人工参考来源
- WCA / TNoodle 只作为 scramble 语义参考，不作为公式库数据源

这意味着首批全量 OLL / PLL 数据允许采用混合来源：

- 一部分来自可直接导入的开放数据
- 一部分来自人工转录的公共参考资料

但最终仓库中的唯一权威始终是 vendored TOML。

## 运行时加载与缓存

实现时固定采用以下流程：

1. 读取 vendored TOML
2. 解析为原始数据对象
3. 校验 case id / algorithm id / 默认公式 / provenance
4. 校验 `pattern`
5. 对每条算法执行 `parse_notation()`
6. 对解析结果执行 `resolve_notation(..., Orientation::SOLVED)`
7. 缓存 canonical `MoveSequence`
8. 构建 OLL / PLL AUF 预展开索引

要求：

- `notation` 解析失败应在加载阶段报错
- `resolve_notation(..., Orientation::SOLVED)` 结果必须只含 canonical 面转
- 运行时 `lookup` 不重复解析公式文本

缓存细节不是公共 API，但它属于正式实现要求。

## Scramble 设计

### 模式

`rubiks-alg` 的 scramble 模式固定为两类：

- `RandomState3x3`
- `TrainingFaceTurn { length }`

### `TrainingFaceTurn`

这是一类训练用随机面转序列，不是官方标准 scramble。

首批实现要求固定为：

- 输出 canonical 面转
- 只使用 `U R F D L B` 与 `'` / `2`
- 不允许连续两步作用于同一面
- `length == 0` 视为非法输入

首批明确不要求：

- 限制连续同轴三步
- 近似官方 random-state 分布

### `RandomState3x3`

这是官方 / 标准方向的长期目标接口。

技术路线固定为：

- 参考 WCA / TNoodle 的随机状态语义自主实现
- 不集成 TNoodle 作为外部运行依赖
- 不允许用简单面转随机串近似替代

但首批实现不要求落地：

- 允许返回 `ScrambleError::UnsupportedMode`

### RNG 入口

公开接口固定为双入口：

- `generate(...)`
- `generate_with_rng(...)`

语义：

- `generate(...)` 适合普通调用
- `generate_with_rng(...)` 适合测试、复现与基准

## 当前尚未完成的部分

当前实现已经覆盖 crate 骨架、loader、lookup、全量公式数据、CLI / REPL 公式展示和训练用 scramble，但以下工作仍待补完：

- 实现 `RandomState3x3`
- 与 `rubiks-solver` 做正式对接

## 测试与验收标准

当前基础实现已经覆盖其中一部分验收项，以下仍然作为完整完成标准：

### 数据加载

- 每个 case 文件可解析
- `case_id` 全局唯一
- `algorithm_id` 全局唯一
- 每个 case 恰有一个默认公式
- 所有公式都具有 provenance
- 所有 `notation` 都能通过 `parse_notation()`
- 所有公式都能通过 `resolve_notation(..., Orientation::SOLVED)`

### Pattern 与 lookup

- 每个 OLL case 的 canonical pattern 可稳定编码为唯一 `OllPattern`
- 每个 PLL case 的 canonical pattern 可稳定编码为唯一 `PllPattern`
- AUF 预展开后，4 个 U 对齐都能命中同一 case
- `CaseMatch::auf` 的语义与“先对齐再执行公式”一致
- 不满足 OLL / PLL 前置条件时返回 `PrerequisiteNotMet`
- 构建索引时出现 collision 必须失败，不允许静默覆盖

### 公式语义

- 每个 case 至少有一条默认公式
- 默认公式的 canonical 序列可回放
- 对 PLL，若定义了 `post_auf`，则 `auf + alg + post_auf` 应与 canonical 目标语义一致

### Scramble

- `TrainingFaceTurn` 输出长度正确
- 不出现连续同面
- 只输出 canonical face turn
- 固定 RNG 下结果可复现

### 文档一致性

- `rubiks-alg.md` 与总计划中的 `rubiks-alg` 段保持一致
- 术语、类型名、case 命名、数据来源策略必须一致

## 下一步最小里程碑

当前建议按以下顺序继续推进 Phase C：

1. 实现 `RandomState3x3`
2. 为公式库补充更多备选公式与标签
3. 在 CLI / REPL 中补充按公式 id 或标签过滤的浏览能力
4. 再评估与 `rubiks-solver` 的接口联动

其中：

- `TrainingFaceTurn` 已经可用，但它不是 WCA / TNoodle 意义上的标准随机状态打乱
