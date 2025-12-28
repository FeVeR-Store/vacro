# Vacro CLI

**The Visualization Tool for Vacro**

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

Since this is a Cargo subcommand, run it inside your project directory:

```bash
cargo vacro
```

This will compile your project and open the TUI to display the collected trace data.

## Key Features

* **Snapshot Diffing**: Select a tag (e.g., "rewrite") and see a side-by-side diff of the TokenStream before and after transformation.
* **Log Filtering**: Filter logs by level or module to focus on specific parts of your macro.
