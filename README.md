# end (v2 workspace)

## Quick Start

```bash
cargo run -q
```

默认行为：
- 运行 `end-cli solve`
- 读取当前目录 `aic.toml`；若不存在则使用默认配置并给出 warning

## Commands

初始化 `aic.toml`：

```bash
cargo run -q -- init
```

指定语言与数据目录求解：

```bash
cargo run -q -- solve --lang en --data-dir ./data
```

将 v1 Lua 输入转换为 v2 TOML：

```bash
cargo run -q -p end-lua2toml -- convert -i data/input -o data
```

## Workspace Crates

- `crates/end_model`: 纯数据模型与公共类型
- `crates/end_io`: TOML 加载与引用/Schema 校验
- `crates/end_opt`: 两阶段 MILP 求解
- `crates/end_report`: 报告渲染与 i18n 显示
- `crates/end_lua_convert`: v1 Lua -> v2 TOML 转换器库
- `crates/end_cli`: v2 主程序
- `crates/end_lua2toml`: 转换器 CLI
