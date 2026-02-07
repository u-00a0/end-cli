## 1. 集合（Sets）

- 物品：$i \in I$，$|I|=m$
- 配方：$r \in R$
- outpost：$o \in O$，$|O|=k$
- 机器类型：$t \in T$
- 电池类型（可被热容池消耗以发电）：$b \in B \subseteq I$

## 2. 参数（Parameters）

### 2.1 物料与配方
- $s_i \ge 0$：每分钟外部供给
- $a_{r,i}$：配方净变化
  - $a_{r,i} > 0$ 产出，$a_{r,i} < 0$ 消耗

### 2.2 贸易
- $p_{o,i} \ge 0$：outpost $o$ 对物品 $i$ 的收购单价
- $C_o \ge 0$：outpost $o$ **每小时**最大交易额

### 2.3 机器产能与耗电
- 机器类型 $t$ 的机器可以生产配方集合 $R_t$，每台机器只能专门生产一个配方。
- $R_t \subseteq R$：类型 $t$ 机器可生产的配方集合
- $u_{t,r} \ge 0$：若一台类型 $t$ 机器专门跑配方 $r$，其每分钟最大吞吐
  - 约定：若 $r \notin R_t$，则 $u_{t,r}=0$
- $w_t \ge 0$：类型 $t$ 机器的功率消耗（瓦）
- $P^\text{core} = 200$: 核心免费送 200 瓦电力
- $P^\text{ext} \ge 0$: 生产机器外的其他电力消耗

### 2.4 热容池（Thermal Bank）
- 热容池自身不耗电，只发电；电池消耗计入物料守恒。
- 在该模型中热容池必须持续喂入同一类型电池以稳定输出功率。事实上，游戏电网存在容量，允许启停热容池。
- 对每个 $b \in B$：
  - $P_b \ge 0$：每台热容池持续喂入电池 $b$ 时输出功率（瓦）
  - $D_b > 0$：电池 $b$ 在热容池中持续发电秒数

## 3. 决策变量（Decision Variables）

### 3.1 流量变量（连续）
- $x_r \ge 0$：每分钟执行配方 $r$ 次数
- $q_{o,i} \ge 0$：每分钟卖给 outpost $o$ 的物品 $i$ 数量

### 3.2 离散产线设计变量（整数）
- $M_t \in \mathbb{Z}_{\ge 0}$：类型 $t$ 机器总台数
- $Y_{t,r} \in \mathbb{Z}_{\ge 0}\quad \forall t\in T,\ \forall r\in R_t$：类型 $t$ 中专门分配给配方 $r$ 的机器台数
- $Z_b \in \mathbb{Z}_{\ge 0}$：**持续**喂入电池 $b$ 的热容池台数

## 4. 目标（Two-stage Objective）

### Stage 1：最大化每分钟收入
$$
R^* = \max \ \text{Rev} \;=\; \max \ \sum_{o \in O}\sum_{i \in I} p_{o,i}\, q_{o,i}
$$

### Stage 2：在 Stage 1 最优解集合中，最小化机器数量
收入约束：
$$
\sum_{o,i} p_{o,i} q_{o,i} \;\ge\; R^*
$$

最小化机器数量，消除闲置耗电机器：
$$
\min \ \sum_{t \in T} M_t \;+\; \sum_{b \in B} Z_b
$$

## 5. 约束（Constraints）

### 5.1 物品守恒（每分钟）

- 对所有 $i \in I \setminus B$：
$$
s_i \;+\; \sum_{r \in R} a_{r,i}\, x_r \;-\; \sum_{o \in O} q_{o,i} \;\ge\; 0
$$

- 对所有 $b \in B$：
$$
s_b \;+\; \sum_{r \in R} a_{r,b}\, x_r \;-\; \sum_{o \in O} q_{o,b} \;-\; \frac{60}{D_b}\, Z_b \;\ge\; 0
$$

### 5.2 outpost 每小时交易额上限
对所有 $o \in O$：
$$
\sum_{i \in I} p_{o,i}\, q_{o,i} \;\le\; \frac{C_o}{60}
$$

### 5.3 一台机器只能做一个配方（不允许闲置）
对所有 $t \in T$：
$$
\sum_{r \in R_t} Y_{t,r} \;=\; M_t
$$

### 5.4 配方执行速率受分配机器吞吐限制
对所有 $r \in R$：
$$
x_r \;\le\; \sum_{t \in T:\ r \in R_t} u_{t,r}\, Y_{t,r}
$$

### 5.5 电力平衡

总耗电功率：
$$
P^{use} \;=\; \sum_{t \in T} w_t\, M_t \;+\; P^\text{ext}
$$

每台持续喂入电池 $b$ 的热容池稳定输出 $P_b$，总发电功率：
$$
P^{gen} \;=\; \sum_{b \in B} P_b\, Z_b \;+\; P^\text{core}
$$

电力不短缺：
$$
P^{gen} \;\ge\; P^{use}
$$
