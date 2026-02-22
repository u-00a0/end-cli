# 参照 Catalog 的 generativity，改造 AicInputs 与 logistics（#plan，2026-02-22）

## 目标与约束（严格表述）

目标：

1. 让 `AicInputs` 的 outpost 索引与输入实例在类型层绑定，消除“跨 AIC 混用 outpost id”。
2. 让 `end_opt` 的 logistics 结构与输入实例保持同源类型约束，减少不必要的运行时缺失检查。
3. 将 `SupplyNodeId/DemandNodeId/LogisticsNodeId` 绑定到优化结果品牌，避免跨场景/跨结果误配。
4. 复用现有 `Catalog<'id>` 的 generativity 设计风格：边界构造 token，内部只流动强类型 id。

约束：

1. 保持 `parse, don't validate`，优先在边界解析出强类型，不在业务路径做兜底式校验。
2. 调用处业务代码不引入 `unsafe`/`expect`；若内部有 `unsafe`，必须由类型不变量支撑。
3. 跨 crate `Error` 保持 `'static` 传播能力，不直接携带带生命周期的 id 到错误边界。
4. 本次仅产出调查+计划，不做实质性代码改造。
5. 本轮先保持 `inputs.outpost()` 返回 `Option`，强约束 API 后续单独评估。

## 基线调查结果

1. `Catalog` 已完成品牌化：`ItemId/FacilityId/RecipeId/PowerRecipeId` 都是 `<'id>` branded id。
   - 位置：`crates/end_model/src/catalog/model/types.rs`
   - 构造入口：`Catalog::builder(guard)`，`guard` 来自 `generativity::make_guard!`。
2. `AicInputs` 目前只对 item 维度带 `'id`，`OutpostId` 仍是裸 `u32`。
   - 位置：`crates/end_model/src/aic_input.rs:11`
   - 现状 API：`AicInputs<'id>::outpost(&self, id: OutpostId) -> Option<&OutpostInput<'id>>`
3. `end_opt` 的 outpost 相关字段都依赖裸 `OutpostId`，类型上无法阻止“同 catalog 下跨 AIC 混用”。
   - 位置：`crates/end_opt/src/types.rs:23`、`crates/end_opt/src/types.rs:35`、`crates/end_opt/src/types.rs:136`
4. `logistics` 的节点 id（`SupplyNodeId/DemandNodeId/LogisticsNodeId`）均为裸 `u32` newtype。
   - 位置：`crates/end_opt/src/types.rs:78`、`crates/end_opt/src/types.rs:91`、`crates/end_opt/src/types.rs:104`
5. `logistics` 测试存在“独立构造 AIC 仅为拿一个 `OutpostId`”的辅助路径，说明当前类型系统允许跨输入复用 outpost id。
   - 位置：`crates/end_opt/src/logistics.rs:892`
6. `end_report/end_web` 当前错误类型直接携带 `OutpostId/LogisticsNodeId`，若这些 id 品牌化后会影响 `'static` 错误传播。
   - 位置：`crates/end_report/src/error.rs:12`、`crates/end_report/src/error.rs:15`、`crates/end_web/src/error.rs:19`、`crates/end_web/src/error.rs:24`

## 已做基线验证（只读调查）

1. `cargo test -q -p end-model aic_parse_rejects_duplicate_outpost_keys`：通过。
2. `cargo test -q -p end-opt logistics_plan_is_deterministic_for_same_stage_solution`：通过。
3. `cargo test -q -p end-model cross_catalog_ids_do_not_typecheck`：通过（`trybuild` compile_fail 生效）。

## 设计选项（含已确认结论）

### 选项 A（已确认，推荐）：AIC + Result 双新增品牌生命周期（`'sid` + `'rid`）

核心思路：

1. 将 `OutpostId` 改为 `OutpostId<'sid>`，由 `AicInputs` 持有的 scenario token mint。
2. `AicInputs` 改为双生命周期：`AicInputs<'cid, 'sid>`。
3. `run_two_stage` 增加 `Guard<'rid>` 入参，`OptimizationResult` 升级为 `<'cid, 'sid, 'rid>`。
4. `DemandSite::OutpostSale` / `OutpostValue` / `OutpostSaleQty` 等 outpost 关联类型改用 `OutpostId<'sid>`。
5. `SupplyNodeId/DemandNodeId/LogisticsNodeId` 升级为 `<'rid>` 品牌化 id。
6. `load_aic/load_aic_from_str` 增加 `Guard<'sid>` 参数；边界调用方显式创建第二个 guard。

优点：

1. 类型层真正防止“同 catalog 下跨 AIC 混用 outpost id”。
2. 类型层防止“同一 scenario 下跨结果混用 logistics node id”。
3. 风格与 `Catalog` generativity 一致，长期维护更清晰。

代价：

1. 生命周期会扩散到 `end_io/end_opt/end_report/end_web/end_cli`，改动面中等偏大。
2. API 会新增两处 guard 透传（AIC guard + result guard），调用代码更显式。

### 选项 B：只引入 `'sid`（不引入 `'rid`）

核心思路：

1. `OutpostId<'sid>` 与 `AicInputs<'cid, 'sid>` 落地。
2. `SupplyNodeId/DemandNodeId/LogisticsNodeId` 继续保持非 result 品牌化。

优点：

1. 改动小于选项 A。
2. 可快速封堵跨 AIC 的 outpost 混用问题。

缺点：

1. 不能防止“同一 scenario 下跨结果混用 logistics node id”。
2. 无法完整满足“跨场景/跨结果误配都要封堵”的目标。

### 选项 C：仅做 logistics 局部整理，不引入新的 generativity 维度

核心思路：

1. 保持 `OutpostId` 现状。
2. 只改 `logistics` 内部容器与错误路径（例如保留边界错误，清理局部实现细节）。

优点：

1. 成本最低。
2. 对外 API 影响最小。

缺点：

1. 不解决你这次关心的“参照 Catalog 做 generativity 化”目标。

## 推荐执行方案（对应选项 A）

### P0：模型层品牌化骨架（end_model）

- [ ] `OutpostId` 改为 `OutpostId<'sid>`，并提供 `as_u32/index`。
- [ ] `AicInputs` 改为 `AicInputs<'cid, 'sid>`，内部持有 scenario brand。
- [ ] `AicInputs::parse` 改签名：仅接收 `Guard<'sid>`（不引入 `Catalog<'cid>` 入参，避免模型层耦合 IO 解析上下文）。
- [ ] `outposts_with_id()` 返回 `(OutpostId<'sid>, &OutpostInput<'cid>)`。
- [ ] 增加 `trybuild` 证据：跨 AIC 混用 `OutpostId` 编译失败。

### P1：IO 边界与调用入口（end_io/end_cli/end_web）

- [ ] `load_aic/load_aic_from_str` 改为接收 `Guard<'sid>` 并产出 `AicInputs<'cid, 'sid>`。
- [ ] CLI/Web 调用点创建第二个 guard（catalog guard + aic guard 分离）。
- [ ] `default_aic_toml` 内部如需验证 parse，使用局部 guard，不向外泄漏生命周期。
- [ ] `end_io` 相关单测与测试辅助函数签名同步到 `AicInputs<'cid, 'sid>`。

### P2：优化器与物流类型联动（end_opt）

- [ ] `run_two_stage` 增加 `Guard<'rid>` 入参，`OptimizationResult` 升级到 `<'cid, 'sid, 'rid>`。
- [ ] `solve_stage`/`StageSolution` 保持与业务语义一致：仅携带必要生命周期，避免无意义引入 `'rid`。
- [ ] `OutpostValue`、`OutpostSaleQty`、`DemandSite::OutpostSale` 等 outpost 相关结构升级到 `OutpostId<'sid>`。
- [ ] `SupplyNodeId/DemandNodeId/LogisticsNodeId` 升级为 `<'rid>`，并同步更新 `ItemFlowEdge/LogisticsEdge/LogisticsNode/ItemSubproblem` 的字段签名。
- [ ] `logistics.rs` 内部容器（`DenseNodeMap`、`BTreeMap` key、排序/索引流程）跟随新 id 类型改造，确保不再依赖裸 `u32` 作为跨函数接口。
- [ ] 增加 compile-fail 证据：跨 scenario 混用 `OutpostId`、跨 result 混用上述 NodeId 均编译失败（可放 `trybuild` 或 experiments）。

### P3：展示层与错误边界（end_report/end_web）

- [ ] 跟随新签名透传生命周期（`Catalog/AicInputs/OptimizationResult` 的新参数）。
- [ ] 保持公共错误 `'static`：`MissingOutpost`/`MissingLogisticsNode` 等错误字段统一降级为 `u32`/`String`，不暴露 branded id。
- [ ] 本轮保留 `inputs.outpost()` 的 `Option` 与对应 `MissingOutpost` 分支；后续在不变量稳定后再评估收窄。

### P4：测试与回归用例迁移

- [ ] `end_io/tests/aic_validation.rs` 与测试 helper 升级到新签名。
- [ ] `end_opt/tests/two_stage_regression.rs` 与 `logistics.rs` 内测升级到 `sid/rid` 方案。
- [ ] `end_report/tests/report_render.rs` 升级到 `AicInputs/OptimizationResult` 新签名。
- [ ] 保持现有 trybuild compile-fail 套件可复现，并新增跨 AIC / 跨 result 误配证据。

### P5：回归验证

- [ ] `cargo make done`
- [ ] `scripts/build_web_wasm.sh`
- [ ] `web` 下 `npm run check`
- [ ] `web` 下 `npm run test:console`

## 已确认决策

1. 按推荐的 **选项 A（`catalog + scenario + result`，即 `cid + sid + rid`）** 执行。
2. 本轮先保持 `inputs.outpost()` 为 `Option`，将 infallible 收敛留到后续专项。

## 结论

从当前代码看，`Catalog` 的 generativity 模板已经成熟，可直接复用到 AIC/outpost 维度并扩展到 result 维度。若目标是“类型层彻底防误用（跨 AIC + 跨 result）”，必须选择带 `rid` 的选项 A；选项 B/C 只能部分改进。
