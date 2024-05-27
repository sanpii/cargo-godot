# cargo-godot

[![Crates.io](https://img.shields.io/crates/v/cargo-godot)](https://crates.io/crates/cargo-godot)
[![Github actions](https://github.com/sanpii/cargo-godot/workflows/.github/workflows/ci.yml/badge.svg)](https://github.com/sanpii/cargo-godot/actions?query=workflow%3A.github%2Fworkflows%2Fci.yml)
[![pipeline status](https://gitlab.com/sanpi/cargo-godot/badges/main/pipeline.svg)](https://gitlab.com/sanpi/cargo-godot/-/commits/main)

Cargo helper to improve godot rust developement experience.

## Install

```
cargo install cargo-godot
```

## Use

In your rust project, you should add metadata in the Cargo.toml to specify the
godot project path:

```toml
[package.metadata.godot]
project = "../godot"
```

Then, you can directly run/export/debug your project directly via cargo:

```
cargo godot run
```

`cargo-godot` generates the `.gdextension` file at the top of rust project. You
can create a link in your godot project:

```
cd ../godot
ln -s ../rust/project.gdextension
```
