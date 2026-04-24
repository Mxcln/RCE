# rubiks-alg Reserved Interface Plan

**Status:** 不属于首版交付  
**Goal:** 为未来公式目录、case 识别与随机打乱保留稳定接口方向，但当前阶段不实现 crate。

## 当前结论

`rubiks-alg` 不进入首版实现。

原因：

- 首版目标是先把 `rubiks-core` 与 `rubiks-cli` 做成完整闭环
- 公式目录、case 识别与随机打乱都会引入额外数据模型复杂度
- 旧方案里“从算法文本反推 case”过于脆弱，不适合直接落地

因此本文件只定义未来的设计边界，不提供实施任务。

## 设计原则

### 1. 使用 case-first 模型

必须区分：

- case 定义
- 公式实现

不要把：

- `name | notation`

直接当作稳定的 case 数据来源。

### 2. 不从算法文本反推唯一 case 身份

原因：

- 同一 case 可以有多条公式
- 不同公式可能只是 AUF、视角或记法糖不同
- 数据文件中容易混入重复或等价公式

稳定的识别逻辑应直接依赖：

- 显式的 canonical pattern
- 显式的 case id

### 3. `rubiks-alg` 依赖 `rubiks-core`

未来：

```text
rubiks-alg → rubiks-core
```

不直接依赖 CLI。

## 建议数据结构

```rust
pub struct AlgEntry {
    pub id: String,
    pub name: String,
    pub notation: ExtMoveSequence,
}

pub struct OllPattern(/* private */);
pub struct PllPattern(/* private */);

pub struct OllCase {
    pub case_id: String,
    pub canonical_pattern: OllPattern,
    pub algorithms: Vec<AlgEntry>,
}

pub struct PllCase {
    pub case_id: String,
    pub canonical_pattern: PllPattern,
    pub algorithms: Vec<AlgEntry>,
}
```

说明：

- `AlgEntry` 表示一条公式，并保留展示用原始记法
- `OllCase` / `PllCase` 表示一个可识别 case
- 一个 case 可以拥有多条候选公式
- `OllPattern` / `PllPattern` 目前只作为占位抽象类型，暂不承诺具体编码格式或位宽
- 如需要 canonical `MoveSequence`，应在加载或运行时通过 `resolve_notation(..., Orientation::SOLVED)` 派生或缓存

## 建议接口

### case 命中结果

```rust
pub struct CaseAlignment {
    // exact fields TBD during implementation
}

pub struct CaseMatch<'a, T> {
    pub case: &'a T,
    pub alignment: CaseAlignment,
}
```

### catalog 接口

```rust
pub trait AlgCatalog {
    fn lookup_oll(&self, cube: &CubeState) -> Option<CaseMatch<'_, OllCase>>;
    fn lookup_pll(&self, cube: &CubeState) -> Option<CaseMatch<'_, PllCase>>;
}
```

说明：

- `lookup_pll()` 的前置条件仍然应该是“顶层朝向已完成”
- `lookup_oll()` / `lookup_pll()` 当前只是方向性接口草案；观察坐标、对齐语义以及是否需要额外 query object，留到真正实现阶段再正式编码
- 是否把这个前置条件编码进类型系统，留到实现阶段再决定
- `CaseAlignment` 当前只作为占位抽象类型；是否最终只需要 AUF，留到实现阶段再决定

## 数据文件方向

未来数据文件至少应包含：

- `case_id`
- `canonical_pattern`
- `algorithm_id`
- `display_name`
- `notation`

不要只保留：

- `name`
- `notation`

## 随机打乱

随机打乱属于 `rubiks-alg` 的职责，而不是 `rubiks-cli` 或 `rubiks-core` 的职责。

未来接口可参考：

```rust
pub trait ScrambleGenerator {
    fn generate(&self, length: usize) -> MoveSequence;
}
```

但当前阶段不实现。

## 实现前置条件

只有在以下条件满足后，才建议真正开始 `rubiks-alg`：

- `rubiks-core` API 稳定
- `parse_notation()` 语义稳定
- `resolve_notation(..., Orientation::SOLVED)` 语义稳定
- `ExtMoveSequence` 稳定
- `MoveSequence` 稳定
- `CubeState` 的顶层状态提取逻辑已验证清楚

## 当前阶段不做的事

- 不创建 OLL / PLL 数据文件
- 不创建公式目录实现
- 不创建 scramble 实现
- 不创建 CLI 接入

## 进入实现阶段时的完成标准

未来真正开始实现后，再使用以下标准：

- 数据文件显式定义 case，而不是从公式文本反推
- `lookup_oll()` / `lookup_pll()` 可稳定命中 case
- 支持一个 case 多条公式
- collision 不允许静默覆盖
- `canonical_pattern` 与 case 对齐信息不提前泄漏具体底层编码格式
