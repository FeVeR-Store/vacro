# Vacro CLI

**Vacro 可视化终端工具**

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

这将编译你的项目并打开 TUI 界面，展示收集到的追踪数据。

## 核心特性

* **快照 Diff**：选择一个标签（例如 "rewrite"），并在分栏视图中查看 TokenStream 转换前后的差异。
* **日志过滤**：按级别或模块过滤日志，专注于宏的特定部分。
