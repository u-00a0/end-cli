# 2026-02-23 Stage2 目标自定义改动总结

## 背景

为匹配最新模型设计，Stage2 需要支持可配置优化目标（不仅是最少机器），并将该能力暴露到 `aic.toml` / CLI / 前端输入链路。

---

## 本次改动内容

### 1) 模型层：引入 Stage2 可配置目标

在 `end_model` 中新增 Stage2 目标类型：

- `min_machines`
- `max_power_slack`
- `max_money_slack`
- `weighted { alpha, beta, gamma }`

并挂到 `AicInputs`：

- 增加 `stage2_objective()` 访问器
- 通过 `AicInputs::builder(...).stage2_objective(...)` 设置
- 默认值为 `min_machines`

相关文件：

- `crates/end_model/src/aic_input.rs`
- `crates/end_model/src/lib.rs`

---

### 2) IO/TOML：支持 `aic.toml` 的 `[stage2]` 配置

`aic.toml` 新增配置段：

```toml
[stage2]
objective = "min_machines" # min_machines | max_power_slack | max_money_slack | weighted
alpha = 1.0
beta = 1.0
gamma = 1.0
```

实现点：

- schema 增加 `Stage2Toml` 与 `Stage2ObjectiveToml`
- objective 字符串解析与校验
- `alpha/beta/gamma` 非负浮点解析与默认值
- `resolve_aic` 将 schema 映射到 `AicInputs.stage2_objective`

相关文件：

- `crates/end_io/src/schema.rs`
- `crates/end_io/src/aic.rs`
- `crates/end_io/src/aic.toml`（内置默认模板）
- `aic.toml`（工作区示例）

---

### 3) 求解器：实现 Stage2 多目标流程

`run_two_stage` 的 Stage2 分支改为根据 `aic.stage2_objective()` 选择：

- `MinMachines`
- `MaxPowerSlack`
- `MaxMoneySlack`
- `Weighted`

实现细节：

- `max_money_slack` / `weighted`：启用虚拟销量变量 `q_tilde` 与 money slack
- `max_power_slack` / `weighted`：启用 power slack 变量
- 统一保持 Stage1 真实收益 floor 约束（真实销量收益不退化）
- `weighted` 先分别求 `max_power_slack` 与 `max_money_slack` 的 Stage2 最优值，作为归一化尺度后再求加权目标

相关文件：

- `crates/end_opt/src/solver.rs`

---

### 4) 前端：提供 Stage2 目标可编辑能力

前端草稿与编辑器支持 Stage2 配置：

- 类型系统新增 `stage2` 字段（objective + alpha/beta/gamma）
- TOML 解析/导出支持 `[stage2]`
- 编辑面板新增 Stage2 目标选择
- 仅在 `weighted` 目标下显示权重输入框
- 本地草稿持久化/恢复包含 Stage2 字段

相关文件：

- `web/src/lib/types.ts`
- `web/src/lib/aic.ts`
- `web/src/lib/editor-actions.ts`
- `web/src/lib/draft-actions.ts`
- `web/src/lib/draft-storage.ts`
- `web/src/components/EditorPanel.svelte`
- `web/src/App.svelte`
- `web/src/lib/aic.test.ts`
- `web/src/lib/draft-actions.test.ts`

---

## 冗余字段收敛（后续修正）

根据代码审阅意见，已删除 `StageSolution.power_slack_w`，统一以 `power_margin_w` 表达电力余量：

- 删除字段：`crates/end_model/src/optimization.rs`
- 求解器归一化读取改为 `power_margin_w as f64`：`crates/end_opt/src/solver.rs`

说明：

- 对外展示/报告/DTO 本来就使用 `power_margin_w`
- 删除后语义更清晰，避免重复状态

---

## 兼容性与行为说明

- 不配置 `[stage2]` 时默认行为与旧版本保持一致：`min_machines`
- `weighted` 模式下，`alpha/beta/gamma` 支持非负浮点
- `max_money_slack` 与 `weighted` 会引入虚拟成交额变量（不影响真实收益 floor）

---

## 验证结果

已执行并通过：

- `cargo make done`
- `scripts/build_web_wasm.sh`
- `cd web && npm run check`
- `cd web && npm run test:console`

结果：通过（Rust tests 全绿，前端类型检查与 console 测试通过）。
