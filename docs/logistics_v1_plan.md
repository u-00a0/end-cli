# 产线规划功能方案（Logistics v1）

> 目的：把现有两阶段 MILP 求产线规模的结果，进一步生成一个整体物流图。  
> 方法：对每个物品单独构建运输/分配子问题（供给点、需求点按机器粒度），在满足所有需求的前提下使用 Best-Fit 启发式尽量减少连接数量，再把所有物品的结果合并成一张图。

## 0. 范围与非目标

### 0.1 目标
- 输入：`Catalog` + `AicInputs` + 阶段 2 求解结果（含流量信息）。
- 输出：
  - 每个物品的一组物流连接（供给点 -> 需求点，带 `flow_per_min`）。
  - 合并后的整体物流图（按物品分层）。

### 0.2 不做
- 不优化布局面积，只优化连接数量。
- 不做时序、缓冲、库存动态，只做稳态流量（units/min）。
- 不引入第二种物流求解策略，先只实现 Best-Fit（启发式分配）。

## 1. 类型先行（先把接口写清楚）

### 1.1 需要从 `end_opt` 阶段 2 补充的解数据

当前 `StageSolution` 里有 `value_per_min`，但物流需要 `qty_per_min`。  
因此先补齐“可重建物流”的最小闭包数据：

```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PosF64(f64);

impl PosF64 {
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct OutpostSaleQty {
    pub outpost_index: OutpostId,
    pub item: ItemId,
    pub qty_per_min: PosF64,
    pub price: u32,       // 冗余保留，便于和收入对账
}

#[derive(Debug, Clone)]
pub struct RecipeRun {
    pub recipe_index: RecipeId,
    pub machines: u32,            // y_r
    pub executions_per_min: f64,  // x_r
}

#[derive(Debug, Clone)]
pub struct ThermalBankRun {
    pub power_recipe_index: PowerRecipeId,
    pub ingredient: ItemId,
    pub banks: u32, // z_b
    pub duration_s: u32,
}
```

### 1.2 物流求解输入/输出类型草案

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SupplyNodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DemandNodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MachineOrdinal(u32); // 从 1 开始

#[derive(Debug, Clone)]
pub enum SupplySite {
    ExternalSupply { item: ItemId },
    RecipeOutput {
        recipe_index: RecipeId,
        machine: MachineOrdinal,
        item: ItemId,
    },
}

#[derive(Debug, Clone)]
pub enum DemandSite {
    RecipeInput {
        recipe_index: RecipeId,
        machine: MachineOrdinal,
        item: ItemId,
    },
    OutpostSale { outpost_index: OutpostId, item: ItemId },
    ThermalBankFuel {
        power_recipe_index: PowerRecipeId,
        bank: MachineOrdinal,
        item: ItemId,
    },
}

#[derive(Debug, Clone)]
pub struct SupplyNode {
    pub id: SupplyNodeId,
    pub site: SupplySite,
    pub capacity_per_min: PosF64,
}

#[derive(Debug, Clone)]
pub struct DemandNode {
    pub id: DemandNodeId,
    pub site: DemandSite,
    pub demand_per_min: PosF64,
}

#[derive(Debug, Clone)]
pub struct ItemSubproblem {
    pub item: ItemId,
    pub supplies: Vec<SupplyNode>,
    pub demands: Vec<DemandNode>,
}

#[derive(Debug, Clone)]
pub struct ItemFlowEdge {
    pub item: ItemId,
    pub from: SupplyNodeId,
    pub to: DemandNodeId,
    pub flow_per_min: PosF64,
}

#[derive(Debug, Clone)]
pub struct ItemFlowPlan {
    pub item: ItemId,
    pub edges: Vec<ItemFlowEdge>,
}

#[derive(Debug, Clone)]
pub struct LogisticsPlan {
    pub per_item: Vec<ItemFlowPlan>,
}
```

### 1.3 类型不变量（实现时要固化）

- 所有“严格大于 0”的流量字段使用 `PosF64` 表达，禁止裸 `f64`。
- `MachineOrdinal` 在同一 recipe/power-recipe 内唯一。
- `ItemSubproblem` 中的 `SupplyNodeId` / `DemandNodeId` 必须各自连续、无重复。
- `SupplyNodeId` / `DemandNodeId` 的分配顺序必须稳定（禁止依赖 `HashMap` 迭代顺序）。
- 每个 `ItemFlowPlan` 只允许出现该 `item` 的边，不跨物品。

## 2. 问题定义与 Best-Fit 算法（类似 `docs/model_v1.md` 的写法）

> 总体思路：按物品分解。对每个物品 $i$ 单独运行一个确定性的 Best-Fit 启发式分配器。

### 2.1 从阶段 2 解展开机器粒度节点

记配方 $r$ 的总执行速率为 $x_r$（次/min），机器数为 $y_r$，单机上限为 $u_r = 60 / time_r$。  
将其拆到每台机器 $m = 1..y_r$：

$$
\rho_{r,m} = \min\left(u_r,\ \max\left(x_r - (m-1)u_r,\ 0\right)\right)
$$

则有：
- $0 \le \rho_{r,m} \le u_r$
- $\sum_{m=1}^{y_r} \rho_{r,m} = x_r$

记配方净变化为 $a_{r,i}$（和 `model_v1` 一致，产出为正，消耗为负）：

- 若 $a_{r,i} > 0$，机器 $(r,m)$ 对物品 $i$ 的供给量为 $a_{r,i} \cdot \rho_{r,m}$。
- 若 $a_{r,i} < 0$，机器 $(r,m)$ 对物品 $i$ 的需求量为 $(-a_{r,i}) \cdot \rho_{r,m}$。

热容池燃料需求：对 power recipe $p$（燃料物品 $ing(p)$，时长 $D_p$，台数 $z_p$）：

$$
\delta_p = \frac{60}{D_p}
$$

每台 bank 产生一个需求点，需求量 $\delta_p$。

### 2.2 集合（Sets）

对固定物品 $i$：

- 供给点集合：$s \in S_i$
  - 外部供给点（可选）；
  - 所有对 $i$ 有正净产出的机器实例点。
- 需求点集合：$d \in D_i$
  - 所有消耗 $i$ 的机器实例点；
  - 售卖 $i$ 的 outpost 点；
  - 消耗 $i$ 的热容池实例点。

### 2.3 参数（Parameters）

- $U_{i,s} > 0$：供给点 $s$ 的最大可供给流量（units/min）。
- $V_{i,d} > 0$：需求点 $d$ 的必须满足流量（units/min）。
- $\varepsilon > 0$：浮点容差（推荐 `1e-9`）。

可行性前提（来自阶段 2 物料守恒）：

$$
\sum_{s \in S_i} U_{i,s} \ge \sum_{d \in D_i} V_{i,d}
$$

### 2.4 Best-Fit 分配规则

定义剩余量：

- $R_{i,s}$：供给点 $s$ 的剩余可供给量，初始为 $U_{i,s}$。
- $N_{i,d}$：需求点 $d$ 的剩余需求量，初始为 $V_{i,d}$。

需求处理顺序（保证稳定）：

1. 按 $V_{i,d}$ 降序（先处理大需求）。
2. 同流量下按 `DemandNodeId` 升序。

对每个需求点 $d$，循环直到 $N_{i,d} \le \varepsilon$：

1. 构造可一口气满足当前需求的候选集合  
   $C_d = \{ s \in S_i \mid R_{i,s} \ge N_{i,d} \}$。
2. 若 $C_d$ 非空，选  
   $s^\* = \arg\min_{s \in C_d}(R_{i,s}, \text{SupplyNodeId}(s))$，  
   即“能装下且最紧”的供给点（Best-Fit），分配 $q = N_{i,d}$。
3. 若 $C_d$ 为空且存在 $R_{i,s} > \varepsilon$，选剩余量最大的供给点  
   $s^\* = \arg\max_{s: R_{i,s} > \varepsilon}(R_{i,s}, -\text{SupplyNodeId}(s))$，  
   分配 $q = \min(R_{i,s^\*}, N_{i,d})$。
4. 若 $C_d$ 为空且不存在 $R_{i,s} > \varepsilon$，返回不可行错误。
5. 记一条流量边（若同一边已存在则累加）：
   $f_{i,s^\*,d} \mathrel{+}= q$。
6. 更新剩余量：
   $R_{i,s^\*} \leftarrow R_{i,s^\*} - q$，
   $N_{i,d} \leftarrow N_{i,d} - q$。

### 2.5 结果性质（Properties）

- 可行性前提成立时，算法应满足每个需求点：
$$
\sum_{s \in S_i} f_{i,s,d} = V_{i,d} \quad \forall d \in D_i
$$
- 任一供给点不超上限：
$$
\sum_{d \in D_i} f_{i,s,d} \le U_{i,s} \quad \forall s \in S_i
$$
- 由于使用启发式，连接数量不是全局最优保证，只保证确定性与可复现。

### 2.6 复杂度与终止（Complexity）

- 朴素实现每次扫描供给点，单物品时间复杂度约为
  $O(|D_i| \cdot |S_i| + K_i \cdot |S_i|)$，其中 $K_i$ 是拆分产生的额外分配轮次。
- 算法单调减少总剩余需求 $\sum_d N_{i,d}$，若可行性前提成立则必然终止。
- 若运行中出现 $N_{i,d} > \varepsilon$ 且所有 $R_{i,s} \le \varepsilon$，应返回不可行错误。

## 3. 求解与产物组装

1. 读取阶段 2 解，先构建 `RecipeRun` / `OutpostSaleQty` / `ThermalBankRun`。
2. 把每个 recipe 按 $\rho_{r,m}$ 展开到机器粒度，生成各物品的供给/需求点。
3. 对每个物品构建 `ItemSubproblem` 并运行 Best-Fit 分配，得到 `ItemFlowPlan`。
4. 过滤 `flow_per_min <= ε` 的数值噪声边。
5. 合并所有 `ItemFlowPlan` 形成 `LogisticsPlan`。

## 4. 实施 TODO（v1）

- [ ] 在 `end_opt` 阶段 2 结果中补齐 `OutpostSaleQty`（不能只保留 value）。
- [ ] 新增物流模块类型：`ItemSubproblem` / `ItemFlowPlan` / `LogisticsPlan`。
- [ ] 实现“配方执行速率 -> 机器实例速率”展开器（$x_r, y_r \rightarrow \rho_{r,m}$）。
- [ ] 实现单物品 Best-Fit 分配器（目标：在满足需求下尽量减少连接数量）。
- [ ] 增加回归测试：
  - [ ] 机器粒度拆分后总流量守恒；
  - [ ] 每个需求点完全满足；
  - [ ] 同一输入结果稳定（边集合与连接数量一致）。
