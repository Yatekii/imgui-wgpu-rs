# dear imgui wgpu-rs renderer

Draw dear imgui UIs as a wgpu render pass. Based on imgui-gfx-renderer.

#### Initialize
```rust
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

let ui = imgui.frame(..);

let frame = swap_chain.get_next_texture();

let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(
  &wgpu::CommandEncoderDescriptor { todo: 0 });

renderer
  .render(ui, &mut device, &mut encoder, &frame)
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
* wgpu buffer uploading is still in development, so currently only one draw list is supported

Contributions welcome.
