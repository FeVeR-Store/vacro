# Vacro CLI

**Vacro 可视化终端工具**

[![crates.io](https://img.shields.io/crates/v/vacro-cli.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/vacro-cli)

## 简介

`vacro-cli` 是一个终端用户界面 (TUI) 工具，旨在可视化 Vacro 框架开发的过程宏的内部状态。

它消费由 `vacro-trace` 产生的数据，允许开发者：

* **检查日志**：查看宏展开过程中发出的结构化日志（`info!`、`warn!` 等）。
* **快照比对 (Diff)**：通过自动比较具有相同标签的快照，可视化 `TokenStream` 的演变过程。

## 安装

```bash
cargo install vacro-cli
```

## 使用方法

作为一个 Cargo 子命令，请在你的项目目录下运行：

```bash
cargo vacro
```
