# end-cli

`end-cli` 是一个针对《明日方舟：终末地》中的自动化生产问题的求解工具，支持多语言配置和数据输入。它使用两阶段 MILP 求解方法，能够处理复杂的生产链优化问题。

## 安装

### 方式一：下载 GitHub Releases 二进制文件

在 [GitHub Releases](https://github.com/sssxks/end-cli/releases) 页面下载对应平台的程序，解压后将 `end-cli` 可执行文件放到系统 PATH 中即可。

### 方式二：cargo install

```bash
cargo install --git https://github.com/sssxks/end-cli --bin end-cli
```

## 快速开始

```bash
end-cli init
```

默认会在当前目录生成 `aic.toml`。修改上述文件填入数值后，运行：

```bash
end-cli solve
```

## 常用命令速查

```bash
# 覆盖已有 aic.toml
end-cli init --force

# 指定 aic 文件
end-cli solve --aic ./configs/aic.toml
# 指定输出语言，可选语言：`zh`、`en`
end-cli solve --lang en 

# 查看帮助
end-cli --help
end-cli solve --help
end-lua2toml --help
```
