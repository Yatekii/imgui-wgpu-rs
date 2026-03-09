# imgui-wgpu

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Yatekii/imgui-wgpu-rs/build.yml?branch=master)
[![Documentation](https://docs.rs/imgui-wgpu/badge.svg)](https://docs.rs/imgui-wgpu)
[![Crates.io](https://img.shields.io/crates/v/imgui-wgpu)](https://crates.io/crates/imgui-wgpu)
![License](https://img.shields.io/crates/l/imgui-wgpu)

A [wgpu](https://crates.io/crates/wgpu) render backend for [dear imgui](https://crates.io/crates/imgui) (`imgui-rs`).

![screenshot](doc/img/screenshot.png)

## Getting Started

Add `imgui-wgpu` to your `Cargo.toml`:

```toml
[dependencies]
imgui-wgpu = "0.26"
imgui = "0.12"
wgpu = "27"
```

## Usage

Create a `Renderer` during setup, then call `render()` each frame inside a wgpu render pass:

```rust
// Setup
let mut imgui = imgui::Context::create();
let renderer_config = imgui_wgpu::RendererConfig {
    texture_format: surface_format, // must match your surface
    ..imgui_wgpu::RendererConfig::new()
};
let mut renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, renderer_config);

// Per frame
let draw_data = imgui_context.render();
let mut rpass = encoder.begin_render_pass(/* ... */);
renderer.render(draw_data, &queue, &device, &mut rpass).unwrap();
```

For more control over GPU buffer lifetime, use `prepare()` + `split_render()` instead of `render()`.

### Color Space

`RendererConfig::new()` (the default) outputs linear color for **sRGB framebuffers** (`Bgra8UnormSrgb`). Use `RendererConfig::new_srgb()` if rendering to a **linear framebuffer** (`Bgra8Unorm`).

### Custom Textures

```rust
let texture = imgui_wgpu::Texture::new(&device, &renderer, TextureConfig {
    size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
    ..Default::default()
});
texture.write(&queue, &pixel_data, 64, 64);
let tex_id = renderer.textures.insert(texture);
// Use tex_id with imgui::Image
```

## Examples

```sh
cargo run --release --example hello-world
cargo run --release --example custom-texture
cargo run --release --example cube
```

| Example | Description |
|---------|-------------|
| `hello-world` | Basic imgui window with winit integration |
| `custom-texture` | Loading and displaying a custom image |
| `cube` | 3D rendering with an imgui overlay |
| `empty` | Minimal test for empty draw lists |

## Version Compatibility

| imgui-wgpu | wgpu   | imgui |
|------------|--------|-------|
| 0.26.0     | 27     | 0.12  |
| 0.25.0     | 25     | 0.12  |
| 0.24.0     | 0.17   | 0.11  |
| 0.23.0     | 0.16   | 0.11  |
| 0.22.0     | 0.15   | 0.10  |
| 0.21.0     | 0.14   | 0.9   |
| 0.20.0     | 0.13   | 0.8   |
| 0.19.0     | 0.12   | 0.8   |
| 0.18.0     | 0.11   | 0.8   |
| 0.17.0     | 0.10   | 0.8   |
| 0.16.0     | 0.9    | 0.7   |
| 0.15.0     | 0.8    | 0.7   |
| 0.14.0     | 0.7    | 0.7   |
| 0.13.0     | 0.7    | 0.6   |
| 0.12.0     | 0.6    | 0.6   |
| 0.11.0     | 0.6    | 0.5   |
| 0.10.0     | 0.6    | 0.5   |
| 0.9.0      | 0.6    | 0.4   |
| 0.8.0      | 0.5    | 0.4   |
| 0.7.0      | 0.5    | 0.4   |
| 0.6.0      | 0.5    | 0.3   |
| 0.5.0      | 0.4    | 0.3   |
| 0.4.0      | 0.4    | 0.2   |
| 0.1.0      | 0.3    | 0.2   |

## Contributing

Contributions are welcome! Please open an issue or pull request on [GitHub](https://github.com/Yatekii/imgui-wgpu-rs).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
