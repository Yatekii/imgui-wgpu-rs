# dear imgui wgpu-rs renderer

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Yatekii/imgui-wgpu-rs/Build)
[![Crates.io](https://img.shields.io/crates/v/imgui-wgpu)](https://crates.io/crates/imgui-wgpu)
[![Documentation](https://docs.rs/imgui-wgpu/badge.svg)](https://docs.rs/imgui-wgpu)
![License](https://img.shields.io/crates/l/imgui-wgpu)

Draw dear imgui UIs as a wgpu render pass. Based on [imgui-gfx-renderer](https://github.com/Gekkio/imgui-rs/tree/master/imgui-gfx-renderer) from [imgui-rs](https://github.com/Gekkio/imgui-rs).

![screenshot](doc/img/screenshot.png)

# Usage

For usage, please have a look at the [example](examples/hello_world.rs).

# Example

Run the example with
```
cargo run --release --example hello_world
```

# Status

Basic features are useable. Uses `wgpu-0.9` and `imgui-0.7` upstream. `winit-0.24` is used with the examples.

Contributions are very welcome.
