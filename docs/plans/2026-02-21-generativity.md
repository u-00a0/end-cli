# 在本项目里落地 generativity 的可行性、方案和 TODO（2026-02-21 决策更新）

## 目标与约束（严格表述）

目标：

1. 让 `ItemId` / `FacilityId` / `RecipeId` / `PowerRecipeId` 在类型层面绑定到同一个 `Catalog` 来源，消除“跨 catalog 混用 id”的语义漏洞。
2. 消除 `crates/end_opt/src/solver.rs` 里 `item_balance[item.index()]` 这类潜在 panic 点。
3. 去掉 `validate_aic_item_ids` 这种“边界检查式 validate”，改成“边界 parse 后得到更强类型”。
4. 去掉 `crates/end_opt/src/logistics.rs` 业务代码里的 `unsafe`（当前在 `205` 行）以及 `299`/`309` 行不必要的分支错误处理。

约束：

1. 业务代码优先“parse, don't validate”。
2. 不把 `unsafe` 暴露在业务路径调用处。
3. 若采用 generativity，要显式处理 `Guard` 不能逃逸作用域的 caveat（见 `docs/blogs/generativity/arhan.md`）。
4. 本轮接受在公共 API 暴露 `'id`，不采用 closure/HRTB 封装 brand。
5. `end_io/end_opt/end_report/end_web/end_cli` 的错误链路需要保持 `'static` 可传播：`branded id` 不直接进入跨 crate `Error` 的 `#[source]` 链，必要时降级为 `key/u32` 文本信息。
6. “跨 catalog 不可混用”的 `compile_fail` 证据必须进入 workspace 回归（不是只放在 `experiments` 里）。
7. 本轮品牌化范围限定为“catalog 来源一致性”；`AicInputs` 与 `OptimizationResult` 的同源品牌化不在本轮范围内（保留必要运行时检查）。

## 已确认决策（2026-02-21）

1. 采用 **方案 A（全量 branded lifetime）** 作为执行路径。
2. `AicInputs::new` **不保留**；`AicInputs` 仅保留 parse API（从 `Catalog<'id>` 边界解析得到）。
3. doctest 纳入 CI：`cargo make done` 必须覆盖 doctest（不仅 clippy + nextest）。
4. 跨 crate 错误链路保持 `'static`：公共 `Error` 不直接携带 branded id，统一降级为 `u32/key/String` 等 `'static` 信息。
5. 由于 `Guard<'id>` 不能逃逸，本轮将单列“测试辅助函数重构”任务，避免在中途被测试脚手架阻塞。

## 基线调查结果（已在当前代码验证）

1. `ItemId` 等目前只是 `u32` newtype，没有 catalog 品牌信息。
   - 位置：`crates/end_model/src/catalog/model/types.rs`
2. `run_two_stage` 入口仍依赖 `validate_aic_item_ids` 做运行时边界检查。
   - 位置：`crates/end_opt/src/solver.rs:54-58`, `crates/end_opt/src/solver.rs:400-431`
3. `solver.rs` 的 indexing warning 仍然存在（已用 `cargo clippy -p end-opt --all-targets` 复现）。
   - 位置：`crates/end_opt/src/solver.rs:182`, `191`, `200`, `209`, `372`
4. `logistics.rs` 当前有一个调用处 `unsafe` 与两处“理论 unreachable 却转运行时错误”的分支。
   - 位置：`crates/end_opt/src/logistics.rs:205`, `299`, `309`
5. 从 IO 路径看，`aic.toml` 解析时已经通过 `catalog.item_id(key)` 做过 key->id 解析。
   - 位置：`crates/end_io/src/aic.rs:63-69`, `91-97`
   - 结论：CLI/Web 主路径已经“部分 parse”，但 `AicInputs` 作为公共模型类型仍可被外部代码手工构造并混入异 catalog id，因此 `run_two_stage` 仍需兜底检查。

## 可行性结论

可以使用 `arhan.md` 的思路（`generativity::{Guard, Id, make_guard}`）解决这 4 个问题，但需要权衡改造深度。

### 方案 A：全量 branded lifetime（完整语义，改动大）

核心思路：

1. 将核心 id 改为 `ItemId<'id>` 等 branded 类型。
2. `Catalog` / `CatalogBuilder` / `Stack` / `Recipe` / `PowerRecipe` / `AicInputs` 等关联类型全部带 `'id`。
3. 在边界创建 `Guard<'id>` 并传入构造流程（`load_catalog`, `load_aic`, tests）。
4. `run_two_stage` 只接受同 `'id` 的 `Catalog` + `AicInputs`，删除 `validate_aic_item_ids`。
5. `solver` 内建立 `ItemVec<'id, T>`（或同等 branded 容器）消除 indexing panic。
6. `logistics` 去掉 `unsafe`，并把 299/309 的“不可能分支”改为类型上不可达。

优点：

1. 问题 0/1/2/3 一次性从语义层修完。
2. `validate_aic_item_ids` 可删除，不再靠运行时兜底保证“同 catalog”。
3. 代码推理成本明显下降（“同来源”变为类型事实）。

代价：

1. 生命周期参数会穿透 `end_model/end_io/end_opt/end_report/end_web`，改动面大。
2. `Guard` 不能在定义作用域外逃逸，加载 API 需要改签名并在边界显式创建 guard。
3. 现有“foreign id 运行时报错”的测试要改为“类型层不可构造”或边界 parse 错误测试。

预计波及文件：至少 22 个（当前引用 `Catalog`/`AicInputs`/`ItemId` 的文件）。

### 方案 B：仅 end_opt 局部 branded（快速止血，语义不完整）

核心思路：

1. 保留 end_model 公共 id 不变。
2. 在 `run_two_stage` 内将输入 parse 成 `Bound*` 结构（局部 generativity/品牌）。
3. 用局部强类型容器消除 solver indexing warning；去掉 logistics unsafe 与多余分支。

优点：

1. 改动显著更小，风险低。
2. 能快速解决问题 1/2/3。

缺点：

1. 问题 0 在模型层仍存在，语义洞只是在优化器内部被局部封堵。

### 方案 C：不引入 generativity，仅做机械修复（最小改动）

核心思路：

1. 用 `get/get_mut + Result` 替换 indexing 与 unsafe。
2. 保留 `validate_aic_item_ids`。

优点：

1. 见效最快。

缺点：

1. 无法解决问题 0/2 的本质（仍是 validate，不是 parse + 类型保证）。

## 风险与缓解（新增）

1. 错误类型 lifetime 泄漏风险：
   - 风险：若 `Error` 直接携带 `ItemId<'id>` 等 branded id，错误链会失去 `'static`，影响 `anyhow::Context` 与跨 crate 传播。
   - 缓解：跨 crate `Error` 字段统一降级为 `u32/key/String`；branded id 仅在业务计算上下文内部存在，不进入公共错误边界。
2. `compile_fail` 证据未被 CI 覆盖风险：
   - 风险：仅靠 `nextest` 无法覆盖 rustdoc `compile_fail`，可能出现“本地通过、CI 漏检”。
   - 缓解：`cargo make done` 增加 doctest 任务；CI 继续只跑 `cargo make done` 即可覆盖。
3. `Guard<'id>` 导致测试脚手架阻塞风险：
   - 风险：大量测试 helper 目前返回 `Catalog`/`AicInputs`，迁移后需改为 guard 透传，否则会在中后期集中爆炸。
   - 缓解：P0 即开始改测试 helper 形态，单列“测试辅助函数重构”任务，与主类型迁移并行推进。
4. 迁移边界不清导致范围漂移风险：
   - 风险：`Catalog` 同源品牌化与 `AicInputs/OptimizationResult` 同源品牌化是两层问题，混在一起会放大工作量与不确定性。
   - 缓解：本轮范围固定在 catalog 同源；scenario/result 同源留待后续专题处理。
5. parse 路径不闭合风险：
   - 风险：若保留 `AicInputs::new` 公共构造，调用方可绕开 parse API，弱化“parse, don't validate”目标。
   - 缓解：删除 `AicInputs::new` 公共入口，统一改为 parse API（`load_aic`/`load_aic_from_str` 与测试辅助 parse）。

## Catalog + Builder 具体改法（签名草案）

### 1. 核心类型改造

现状（简化）：

```rust
pub struct ItemId(u32);
pub struct CatalogBuilder { ... item_index: HashMap<Key, ItemId> }
pub struct Catalog { ... item_index: HashMap<Key, ItemId> }
```

目标（简化）：

```rust
use generativity::Id;

pub struct ItemId<'id> {
    raw: u32,
    brand: Id<'id>,
}

pub struct CatalogBuilder<'id> {
    brand: Id<'id>,
    item_index: HashMap<Key, ItemId<'id>>,
    // facilities/recipes/power_recipes 同理带 'id
}

pub struct Catalog<'id> {
    brand: Id<'id>,
    item_index: HashMap<Key, ItemId<'id>>,
    // 其它字段同理带 'id
}
```

### 2. Builder API 改签名

现状（简化）：

```rust
impl Catalog {
    pub fn builder() -> CatalogBuilder
}
```

目标（简化）：

```rust
use generativity::Guard;

impl<'id> Catalog<'id> {
    pub fn builder(guard: Guard<'id>) -> CatalogBuilder<'id> {
        CatalogBuilder::new(guard)
    }
}

impl<'id> CatalogBuilder<'id> {
    pub fn new(guard: Guard<'id>) -> Self;
    pub fn add_item(&mut self, def: ItemDef) -> Result<ItemId<'id>, CatalogBuildError>;
    pub fn add_facility(&mut self, def: FacilityDef) -> Result<FacilityId<'id>, CatalogBuildError>;
    pub fn push_recipe(
        &mut self,
        facility: FacilityId<'id>,
        time_s: u32,
        ingredients: Vec<Stack<'id>>,
        products: Vec<Stack<'id>>,
    ) -> Result<RecipeId<'id>, CatalogBuildError>;
    pub fn build(self) -> Result<Catalog<'id>, CatalogBuildError>;
}
```

### 3. Catalog 查询 API 改签名

```rust
impl<'id> Catalog<'id> {
    pub fn item_id(&self, key: &str) -> Option<ItemId<'id>>;
    pub fn facility_id(&self, key: &str) -> Option<FacilityId<'id>>;
    pub fn item(&self, id: ItemId<'id>) -> Option<&ItemDef>;
    pub fn recipe(&self, id: RecipeId<'id>) -> Option<&Recipe<'id>>;
}
```

### 4. AIC 侧联动

```rust
pub struct AicInputs<'id> {
    supply_per_min: ItemNonZeroU32Map<'id>,
    outposts: Vec<OutpostInput<'id>>,
}
```

解析路径 `load_aic(..., catalog: &Catalog<'id>)` 直接产出 `AicInputs<'id>`，从源头保证和 catalog 同 brand。
同时移除 `AicInputs::new` 公共构造入口，统一收敛到 parse API。

### 5. 边界 API 形态（已定）

采用“直接传 guard”：

```rust
make_guard!(guard);
let catalog = load_catalog(data_dir, guard)?;
let aic = load_aic(path, &catalog)?;
```

说明：

1. 该方案允许对外暴露 `'id`（调用方签名透传）。
2. 本轮不引入 closure/HRTB 风格封装。

## 实验与现有用例覆盖核查

实验 crate：`experiments/catalog-generativity-lab`

### 已实证覆盖

1. 同 brand 绑定：
   - `ItemId<'id>` 只能和 `Catalog<'id>` / `AicInputs<'id>` 配对。
   - 证据：`compile_fail` 文档测试（跨 catalog 传 `ItemId` 编译失败）。
2. 边界 parse 替代 validate：
   - `AicInputs::parse(&Catalog<'id>, raw)` 在 key 解析阶段绑定 id。
   - `run_solver` 不再需要 `validate_aic_item_ids`。
3. `ItemVec` 用法：
   - 已实现 `Index/IndexMut<ItemId<'id>>`，业务代码可直接 `item_vec[item_id] += ...`。
   - `clippy::indexing_slicing` 不会对该自定义索引类型报错（实验已跑 clippy 验证）。
4. 去掉无效运行时守卫：
   - `debug_assert_eq!(item.brand, self.brand)` 已移除，不依赖 runtime 比较作为 soundness 前提。

### 与当前仓库用例的映射

1. `end_io::load_catalog` 路径：
   - 现状：通过 builder 构造 catalog。
   - 映射：改为 `CatalogBuilder<'id>::new(Guard<'id>)` 后等价可覆盖。
2. `end_io::load_aic` 路径：
   - 现状：用 `catalog.item_id(key)` 做 key->id，再构造 `AicInputs`。
   - 映射：改为输出 `AicInputs<'id>`，可直接覆盖现有流程。
3. `end_opt::run_two_stage` 路径：
   - 现状：入口运行 `validate_aic_item_ids` + `Vec` indexing。
   - 映射：签名变为同 `'id` 后可删 validate；`ItemVec<'id, T>` 覆盖 indexing 用例。
4. `end_report/end_web` 读取 id 路径：
   - 现状：仅读取/展示，不需要构造新 id。
   - 映射：只需跟随泛型 `'id` 透传签名即可，不改变业务语义。

### 尚未在实验里编码，但可同构迁移

1. `FacilityId/RecipeId/PowerRecipeId` 全套 branded。
2. `Stack<'id>`、`Recipe<'id>`、`PowerRecipe<'id>` 及 builder 的 push 路径。
3. `OutpostInput<'id>` / `ItemU32Map<'id>` / `ItemNonZeroU32Map<'id>` 全量泛型化。

这些项是工程化迁移工作量问题，不是模型可行性风险；类型关系与 `ItemId<'id>` 同构。

## TODO（按方案 A 拆分，供 #go 执行）

### P0：类型骨架与构造入口

- [x] 引入 `generativity` 依赖（workspace + 使用 crate）。
- [x] 将 `ItemId/FacilityId/RecipeId/PowerRecipeId` 改为 `<'id>` branded。
- [x] 将 `Catalog/CatalogBuilder/Stack/Recipe/PowerRecipe` 补齐 `'id` 约束。
- [x] 将 `AicInputs/ItemU32Map/ItemNonZeroU32Map` 补齐 `'id` 约束。
- [x] 删除 `AicInputs::new` 公共入口，新增/统一 `AicInputs::parse(&Catalog<'id>, ...)` 系列 parse API（含测试辅助 parse）。
- [x] 设计并落地边界 API：`load_catalog/load_aic` 需要 `Guard<'id>`，调用方透传 `'id`。
- [x] 重构测试 helper（尤其返回 `Catalog`/`AicInputs` 的函数）为 guard 透传形态，避免后续批次阻塞。

### P1：优化器与物流

- [x] 更新 `end_opt::types` 到 branded id。
- [x] 删除 `validate_aic_item_ids`；`run_two_stage` 只接受同 `'id` 输入。
- [x] 在 `solver` 引入 branded item 容器，清理所有 indexing warning。
- [x] 在 `logistics` 去掉 `unsafe`，并清理 `299/309` 的不必要错误处理。

### P2：调用方与测试

- [x] 更新 `end_io/end_report/end_cli/end_web` 的签名和调用链。
- [x] 调整 `end_io/end_opt/end_report/end_web/end_cli` 的错误类型与错误映射：保持 `Error` 链路 `'static` 可传播，公共错误不携带 branded id，统一转 `key/u32/String` 文本信息。
- [x] 将“跨 catalog 不可混用”的验证迁入 workspace 回归：首选 `trybuild/ui`，并补充 doctest 作为文档证据。
- [x] 将 doctest 纳入 CI 基线（`cargo make done`）并在 workflow 里保持单入口执行。
- [x] 重写 `foreign id` 相关测试与断言：从“运行时报错”切为“类型层不可混用 + 边界 parse 错误”；同步清理旧的运行时兜底路径测试。
- [x] 全量回归：`cargo make done`，并确认 clippy 不再报这些 warning。

## 执行顺序建议

1. 先做 P0，确保类型和构造路径稳定。
2. 再做 P1，集中消除 solver/logistics 风险点。
3. 最后做 P2，统一修调用方和测试。

该计划已按 2026-02-21 决策锁定为方案 A，可直接进入 `#go` 执行。
