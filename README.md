# dear imgui wgpu-rs renderer

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Yatekii/imgui-wgpu-rs/Build)
[![Crates.io](https://img.shields.io/crates/v/imgui-wgpu)](https://crates.io/crates/imgui-wgpu)
[![Documentation](https://docs.rs/imgui-wgpu/badge.svg)](https://docs.rs/imgui-wgpu)
![License](https://img.shields.io/crates/l/imgui-wgpu)

Draw dear imgui UIs as a wgpu render pass. Based on [imgui-gfx-renderer](https://github.com/Gekkio/imgui-rs/tree/master/imgui-gfx-renderer) from [imgui-rs](https://github.com/Gekkio/imgui-rs).

![screenshot](doc/img/screenshot.png)

# Usage

For usage, please have a look at the [example](examples/hello-world.rs).

# Example

Run the example with
```
cargo run --release --example hello_world
```

# Status

Supports `wgpu` `0.12` and imgui `0.8`. `winit-0.26` is used with the examples.

Contributions are very welcome.

# Troubleshooting

## Cargo resolver

Starting with [`wgpu` 0.10](https://github.com/gfx-rs/wgpu/blob/06316c1bac8b78ac04d762cfb1a886bd1d453b30/CHANGELOG.md#v010-2021-08-18), the [resolver version](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions) needs to be set in your `Cargo.toml` to avoid build errors:

```toml
resolver = "2"
```
