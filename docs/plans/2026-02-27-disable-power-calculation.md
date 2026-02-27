# 2026-02-27 禁用电力计算 + 输入结构升级（执行计划与落地）

## 1. 目标与约束（最终确认）

- 目标 1：支持禁用电力计算，使求解器可作为纯配平计算器使用。
- 目标 2：AIC 输入改为 `[power]` 结构，同时支持外部生产电力与外部耗电。
- 目标 3：Stage2 目标从离散模式改为 3 个正交权重值。
- 目标 4：依赖电力计算的输出整体封装为 `Option<struct>`，Web 前端不展示该块。
- 兼容性目标：支持顶层 `version`；未指定时按最新版本解析。

## 2. 最终输入/输出规格

### 2.1 顶层版本

```toml
version = 2
```

- 未指定 `version` 时，默认使用 `2`（当前最新）。
- 指定非 `2` 的版本会报错。

### 2.2 电力配置 `[power]`

```toml
[power]
enabled = true
external_production = 200
external_consumption = 0
```

```toml
[power]
enabled = false
```

规则：

1. `enabled = true`：
- `external_production`：非负整数，默认 `200`
- `external_consumption`：非负整数，默认 `0`
2. `enabled = false`：不允许出现 `external_production` / `external_consumption` 字段。
3. 为兼容旧输入，解析时接受 `enternal_consumption` 作为别名；规范输出统一使用 `external_consumption`。

### 2.3 Stage2 目标 `[objective]`

```toml
[objective]
# all of them are optional. (pos f64)
# 0 present - no stage2 optimization at all
# 1 present - optimize the target
# >= 2 present - optimize weighted
min_machines = 0.1
max_power_slack = 1
max_money_slack = 1
```

规则：

1. 三个字段均为可选、非负浮点数。
2. 有效目标数量（值大于 0 的字段个数）语义：
- `0`：不做阶段二优化（`stage2 = stage1`）
- `1`：执行对应单目标优化
- `>=2`：执行加权目标优化（按权重归一化）
3. `power.enabled = false` 时，`max_power_slack` 必须为 `0`（或缺省）。

### 2.4 输出语义

- `StageSolution` 的电力相关字段改为：
  - `power: Option<PowerSummary>`
- `power.enabled = true`：返回 `Some(PowerSummary)`
- `power.enabled = false`：返回 `None`
- Web 前端不展示 `power` 块（即使后端返回 `Some`）

## 3. 改造前行为调查结论

1. 求解器默认总是建模电力平衡，无法作为纯配平使用。
2. Stage2 使用离散目标模式（含 weighted 例外分支），不是 3 维正交输入。
3. 输出层与前端默认展示平铺电力字段。
4. About 页描述了“可只做配平”的使用方式，但输入与求解行为未完整支撑。

## 4. 需要变更的行为（已落地）

1. `end_io`（schema + 解析）
- 新增 `version`、`[power]`、`[objective]` 解析结构与校验。
- `power.enabled=false` 禁止携带电力数值字段。
- 兼容 `enternal_consumption` 别名。
- 增加 `power.enabled=false` 与 `objective.max_power_slack` 的交叉校验。

2. `end_model`（输入模型）
- 引入 `PowerConfig::{Disabled, Enabled{...}}`。
- 引入 `Stage2Weights { min_machines, max_power_slack, max_money_slack }`。
- 通过 `active_target_count()` 统一判定 Stage2 分支。

3. `end_opt`（求解器）
- 电力启用时：保留热能池与电力约束，并使用 `external_production`/`external_consumption`。
- 电力禁用时：移除电力相关变量与约束，仅保留配平核心约束。
- Stage2 按“有效目标数量”自动选择：`0/1/>=2` 三种路径。
- 电力禁用场景下阻止 power slack 目标。

4. `end_model::optimization` / `end_web` DTO / `end_report`
- 新增 `PowerSummary` 并改为 `Option` 输出。
- API `SummaryDto` 改为 `power: Option<PowerSummaryDto>`。
- CLI 报告按 `power` 是否存在决定是否渲染电力段落。

5. Web（输入/结果）
- 编辑器改为：
  - 电力开关 + 外部发电 + 外部耗电
  - Stage2 三个正交权重输入
- AIC TOML 导入/导出改为新格式，同时兼容旧字段读取。
- 结果面板移除电力展示。

6. 测试
- Rust：schema/解析/solver/report 回归用例已同步新语义。
- Web：草稿动作、TOML 读写、状态与 e2e 已同步新 UI 与新字段。

## 5. 兼容策略

1. 解析兼容：
- 省略 `version` 时按 `2` 解析。
- 旧 `external_power_consumption_w` 仍可从 Web 侧导入映射到新结构。
- 旧 Stage2 枚举形式可映射为新 `objective` 权重。

2. 输出统一：
- 导出 TOML 统一写新格式（`version=2` + `[power]` + `[objective]`）。

## 6. 验证记录

已执行并通过：

- `cargo make done`
- `scripts/build_web_wasm.sh`
- `cd web && npm run check`
- `cd web && npm run test:e2e`

注：e2e 中旧选择器 `#external-power` 已更新为 `#power-external-production`。

## 7. TODO（当前状态）

- [x] 支持禁用电力计算
- [x] 增加外部生产电力输入
- [x] Stage2 改为 3 个正交权重
- [x] 电力输出 `Option` 化并从 Web 结果隐藏
- [x] 增加版本字段并在缺省时采用最新版本
- [x] 完成 Rust 与 Web 回归验证
