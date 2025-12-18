## vacro-doc-i18n

> Multi-language Rustdoc (i18n) support via a lightweight attribute macro.

`vacro-doc-i18n` 是一个 **过程宏 crate**，用于在 **Rust 文档注释（`///` / `#[doc]`）中支持多语言内容**，并在编译期根据 **Cargo feature** 自动裁剪文档语言，从而同时兼顾：

- **docs.rs**：一次构建，加载全量多语言文档（前端 JS 切换）
- **IDE（rust-analyzer）**：只显示当前启用语言的干净文档（无 HTML 标签）

---

## 核心特性

- 基于 **属性宏 `#[doc_i18n]`**
- 文档语言选择 **完全由 Cargo feature 决定**
- 支持两种文档写法：
  - **多行 Block 模式**
  - **单行 Inline 模式**

- 编译期处理，输出仍是标准 `#[doc = "..."]`
- 对 rustdoc / rust-analyzer / docs.rs 透明

---

## 安装

```toml
[dependencies]
vacro-doc-i18n = "0.1"
```

选择文档语言（示例）：

```toml
vacro-doc-i18n = { version = "0.1", features = ["doc-cn"] }
```

---

## 可用 Features

| Feature   | 说明                             |
| --------- | -------------------------------- |
| `doc-en`  | 英文文档（默认）                 |
| `doc-cn`  | 中文文档                         |
| `doc-all` | 保留所有语言文档（用于 docs.rs） |

> Feature 是 **构建级别选择**，不是宏参数。

---

## 使用方式

在需要处理多语言文档的 item 上添加 `#[doc_i18n]`：

```rust
use vacro_doc_i18n::doc_i18n;

#[doc_i18n]
/// ...
pub struct Capture;
```

---

## 文档写法

### 多行 Block 模式（推荐）

```rust
#[doc_i18n]
/// <div class="doc-cn">
/// 即时解析宏：在现有解析逻辑中快速消费 `TokenStream`
/// </div>
/// <div class="doc-en">
/// On-the-fly parsing macro: quickly consume a `TokenStream`
/// </div>
pub struct Capture;
```

规则：

- `<div class="doc-xx">` **必须独占一行**
- `</div>` **必须独占一行**
- 中间内容可跨多行
- 只识别 `class` 中包含 `doc-<lang>` 的 `div`

---

### 单行 Inline 模式

```rust
#[doc_i18n]
/// <div class="doc-cn"> 即时解析宏：快速消费 TokenStream </div>
/// <div class="doc-en"> On-the-fly parsing macro </div>
pub struct Capture;
```

规则：

- **开标签与闭标签必须在同一行**
- 不允许跨行
- 可与普通文本混合在同一行中

---

## 行为说明

### IDE / 本地构建

- 仅保留 **当前 feature 对应语言** 的文档
- 自动 **移除 `<div>` 标签**
- IDE hover 中只显示纯文本

### docs.rs

- 推荐启用 `doc-all`
- 文档中保留所有语言内容
- 可通过前端 JS 实现语言切换

---

## 配置示例

在 **生成文档的 crate**（如 `vacro`）中：

```toml
[features]
doc-en = ["vacro-doc-i18n/doc-en"]
doc-cn = ["vacro-doc-i18n/doc-cn"]
doc-all = ["vacro-doc-i18n/doc-all"]

[dependencies]
vacro-doc-i18n = { version = "0.1" }

[package.metadata.docs.rs]
features = ["doc-all"]
```

---

## 限制

- 仅识别 `<div class="doc-xx"> ... </div>` 结构
- 不支持嵌套的 i18n block
- 未启用严格校验（未闭合标签默认宽松处理）
- 语言码目前内置支持：
  - `doc-en`
  - `doc-cn`

---

## 设计原则

- 不引入新的 DSL，直接复用 HTML 作为“文档标记层”
- 保持最小侵入性，输出仍是标准 Rustdoc

---


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
