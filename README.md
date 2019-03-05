# dear imgui wgpu-rs renderer

Draw dear imgui UIs as a wgpu render pass. Based on imgui-gfx-renderer from [imgui-rs](https://github.com/Gekkio/imgui-rs).

#### Initialize
```rust
use imgui_wgpu::Renderer;

let device: wgpu::Device = ..;
let imgui: imgui::ImGui = ..;
let format = wgpu::TextureFormat::Bgra8Unorm;
let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };

let mut renderer = Renderer::new(&mut imgui, &mut device, format, Some(clear_color))
  .expect("Failed to initialize renderer");
```

#### Render
```rust
let device: wgpu::Device = ..;
let imgui: imgui::ImGui = ..;
let swap_chain: wgpu::SwapChain = ..;
let mut encoder: wgpu::CommandEncoder = ..;

let ui = imgui.frame(..);

let frame = swap_chain.get_next_texture();

renderer
  .render(ui, &mut device, &mut encoder, &frame.view)
  .expect("Rendering failed");
```

# Example

For usage:
```
cargo run --example hello_world --features [vulkan|metal|dx12]
```

# Status

Work in progress.

Known issues:
* clip rects are not implemented
* wgpu buffer uploading is still in development, so currently there is a hardcoded limit on # of vertices/indices to draw

Contributions welcome.
