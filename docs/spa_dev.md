# SPA 开发说明

本文档说明 `end_web` + `web/` 的本地开发流程。

## 1. 先决条件

1. Rust toolchain（含 `wasm32-unknown-emscripten` target）。
2. Emscripten (`emcc`)。
3. Node.js + npm。

如果 `emcc` 不在 PATH，请先加载 emsdk 环境：

```bash
source /home/xks/apps/emsdk/emsdk_env.sh
```

## 2. 构建 wasm 产物

在仓库根目录执行：

```bash
bash scripts/build_web_wasm.sh
```

该脚本会：

1. 编译 `crates/end_web` 的 `end-web` binary（target: `wasm32-unknown-emscripten`）。
2. 复制产物到 `web/public/wasm/`：
   1. `end_web.js`
   2. `end_web.wasm`

> 注意：前端运行依赖这两个文件；改 Rust 后需要重跑脚本。

## 3. 启动 SPA

```bash
cd web
npm install
npm run dev
```

默认地址：`http://localhost:5173`。

## 4. 前端构建与类型检查

```bash
cd web
npm run check
npm run build
```

## 5. 常见问题

### 5.1 `createEndWebModule not found`

说明 `web/public/wasm/end_web.js` 不存在或不是最新版本。

解决：回到仓库根目录重新执行 `bash scripts/build_web_wasm.sh`。

### 5.2 wasm 求解报错

前端会直接展示 Rust 返回的错误信息（TOML 解析、数据校验、求解异常）。
建议先导出当前 GUI 为 `aic.toml`，再用 CLI 复现对比：

```bash
end-cli solve --aic aic.toml
```

### 5.3 物流图节点过多

可在页面右侧 `Logistics Graph` 选择单个 item 过滤查看。
