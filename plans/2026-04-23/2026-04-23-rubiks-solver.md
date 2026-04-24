# rubiks-solver Reserved Interface Plan

**Status:** 不属于首版交付  
**Goal:** 为未来求解器 crate 保留稳定接口方向，但当前阶段不实现搜索框架、CFOP 或其他 solver。

## 当前结论

`rubiks-solver` 不进入首版实现。

原因：

- `rubiks-core` 与 `rubiks-cli` 还需要先形成完整闭环
- solver 是项目中最复杂、最不稳定的部分
- 过早设计通用搜索引擎和 CFOP 分阶段实现，会让首版边界失真

因此本文件只保留未来接口方向，不提供当前实施任务。

## 设计原则

### 1. solver 只操作 `CubeState`

未来求解器的输入输出都应基于：

- `CubeState`
- canonical `MoveSequence`

而不是 `Cube`。

`Cube` 属于用户交互层，不属于 solver 输入模型。

### 2. 先稳定公共接口，再选择具体算法

当前阶段先保留：

- `Solver`
- `SolveError`
- `Solution`
- `SolvePhase`

但不承诺：

- CFOP 是第一个实现
- 通用 IDA* 是最终共享抽象
- 一定需要 `SolverRegistry`

这些都应在第一个真实 solver 开始实现时再确定。

### 3. 不要过早泛化搜索引擎

是否抽出共享搜索层，取决于未来是否真的出现多个 solver 复用相同抽象。  
在那之前，不建议先写一个“通用 IDA* 框架”。

## 建议接口

```rust
pub enum SolveError {
    Unsolvable,
    Unsupported,
}

pub struct SolvePhase {
    pub name: &'static str,
    pub moves: MoveSequence,
}

impl SolvePhase {
    pub fn len(&self) -> usize;
}

pub struct Solution {
    pub solver_name: &'static str,
    pub phases: Vec<SolvePhase>,
}

impl Solution {
    pub fn total_moves(&self) -> MoveSequence;
    pub fn total_len(&self) -> usize;
}

pub trait Solver {
    fn name(&self) -> &'static str;
    fn solve(&self, cube: &CubeState) -> Result<Solution, SolveError>;
}
```

说明：

- 当前阶段不把 timeout 写入稳定接口
- 只有当 solver 请求对象真正引入预算或取消语义后，才考虑加入 timeout 相关错误
- 当前 `Solver` trait 只用于描述未来的大致边界，不视为已经冻结的稳定公共契约；真实 solver 开始实现时，允许根据预算、取消、诊断信息等需要调整签名

## 可选接口

只有当项目里同时存在多个可运行 solver 时，才考虑引入：

```rust
pub struct SolverRegistry {
    solvers: Vec<Box<dyn Solver>>,
}
```

如果最终只有一个 solver，或者 CLI 根本不需要动态注册，就没必要引入这层抽象。

## 未来实现顺序建议

当 `rubiks-core` 稳定后，再重新评估 solver 路线：

### 方案 A：先做 reference solver

优点：

- 快速拿到“从 scramble 到 solved”的完整闭环
- 便于验证 CLI 和状态引擎

### 方案 B：先做 LBL

优点：

- 比完整 CFOP 搜索更容易拆阶段
- 更适合作为第一个真实 solver

### 方案 C：最后再做 CFOP

原因：

- cross / f2l 搜索设计复杂
- 依赖较多启发式与 case 规则
- 更适合作为后续增强，不适合作为首版交付

## 对 `rubiks-alg` 的依赖

如果未来 solver 需要查 OLL / PLL catalog，则依赖关系应为：

```text
rubiks-solver → rubiks-core, rubiks-alg
```

但在 `rubiks-alg` 尚未稳定之前，不建议正式开始 solver 实现。

## 当前阶段不做的事

- 不实现 CFOP
- 不实现 IDA*
- 不实现 `SolverRegistry`
- 不实现 CLI `solve` 命令
- 不把 solver 写进首版测试闭环

## 进入实现阶段时的前置条件

建议至少满足以下条件后，再开始真正的 solver 工作：

- `rubiks-core` API 稳定
- `CubeState::validate()` 稳定
- `parse_notation()` 语义稳定
- `resolve_notation(..., Orientation::SOLVED)` 语义稳定
- `rubiks-alg` 若需要参与 OLL/PLL 查表，则其 case 模型已稳定

## 进入实现阶段时的完成标准

未来真正开始 solver 实现后，再使用以下标准：

- 对给定样例 scramble 能返回真实可执行解
- 回放 `Solution::total_moves()` 后 `is_solved() == true`
- 非法状态返回 `SolveError::Unsolvable`
- phase 统计与 move 总数一致
