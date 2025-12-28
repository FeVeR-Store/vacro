# Vacro CLI

**The Visualization Tool for Vacro**

[![crates.io](https://img.shields.io/crates/v/vacro-cli.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/vacro-cli)

## Introduction

`vacro-cli` is a Terminal User Interface (TUI) tool designed to visualize the internal state of Procedural Macros developed with the Vacro framework.

It consumes data produced by `vacro-trace`, allowing developers to:

* **Inspect Logs**: View structured logs (`info!`, `warn!`, etc.) emitted during macro expansion.
* **Diff Snapshots**: Visualize how `TokenStream`s evolve by automatically comparing snapshots with the same tag.

## Installation

```bash
cargo install vacro-cli
```

## Usage

Run it as a cargo subcommand in your project root:

```bash
cargo vacro
```
