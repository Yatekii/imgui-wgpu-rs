use winit::{
  WindowBuilder, Event, WindowEvent, EventsLoop,
  KeyboardInput, VirtualKeyCode, ElementState,
  dpi::LogicalSize,
};
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support;
use std::time::Instant;

fn main() {
  env_logger::init();
  
  //
  // Set up window and GPU
  //
  let instance = wgpu::Instance::new();

  let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
    power_preference: wgpu::PowerPreference::HighPerformance,
  });

  let mut device = adapter.create_device(&wgpu::DeviceDescriptor {
    extensions: wgpu::Extensions {
      anisotropic_filtering: false,
    },
  });
  
  let mut events_loop = EventsLoop::new();

  let version = env!("CARGO_PKG_VERSION");
  let window = WindowBuilder::new()
    .with_dimensions(LogicalSize { width: 1280.0, height: 720.0 })
    .with_title(format!("imgui-wgpu {}", version))
    .build(&events_loop).unwrap();

  let surface = instance.create_surface(&window);

  let mut dpi_factor = window.get_hidpi_factor();
  let mut size = window
    .get_inner_size()
    .unwrap()
    .to_physical(dpi_factor);


  //
  // Set up swap chain
  //
  let mut sc_desc = wgpu::SwapChainDescriptor {
    usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
    format: wgpu::TextureFormat::Bgra8Unorm,
    width: size.width as u32,
    height: size.height as u32,
  };

  let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);


  //
  // Set up dear imgui
  //
  let mut imgui = ImGui::init();
  imgui.set_ini_filename(None);

  let font_size = (13.0 * dpi_factor) as f32;
  imgui.set_font_global_scale((1.0 / dpi_factor) as f32);

  imgui.fonts().add_default_font_with_config(
      ImFontConfig::new()
          .oversample_h(1)
          .pixel_snap_h(true)
          .size_pixels(font_size),
  );

  imgui_winit_support::configure_keys(&mut imgui);


  //
  // Set up dear imgui wgpu renderer
  // 
  let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };
  let mut renderer = Renderer::new(&mut imgui, &mut device, sc_desc.format, Some(clear_color))
    .expect("Failed to initialize renderer");

  let mut last_frame = Instant::now();


  //
  // Event loop
  //
  let mut running = true;
  while running {
    events_loop.poll_events(|event| {

      match event {
        // On resize
        Event::WindowEvent {
          event: WindowEvent::Resized(_),
          ..
        } => {
          dpi_factor = window.get_hidpi_factor();
          size = window
            .get_inner_size()
            .unwrap()
            .to_physical(dpi_factor);

          sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width as u32,
            height: size.height as u32,
          };

          swap_chain = device.create_swap_chain(&surface, &sc_desc);
        }
        // On ESC / close
        Event::WindowEvent {
          event: WindowEvent::KeyboardInput {
            input: KeyboardInput {
              virtual_keycode: Some(VirtualKeyCode::Escape),
              state: ElementState::Pressed,
              ..
            },
            ..
          },
          ..
        } |
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          running = false;
        },
        _ => (),
      }

      imgui_winit_support::handle_event(
        &mut imgui,
        &event,
        dpi_factor,
        dpi_factor,
      );

    });

    let now = Instant::now();
    let delta = now - last_frame;
    let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
    last_frame = now;

    let frame = swap_chain.get_next_texture();
    let frame_size = imgui_winit_support::get_frame_size(&window, dpi_factor).unwrap();
    let ui = imgui.frame(frame_size, delta_s);

    {
      ui.window(im_str!("Hello world"))
        .size((300.0, 100.0), ImGuiCond::FirstUseEver)
        .build(|| {
            ui.text(im_str!("Hello world!"));
            ui.text(im_str!("This...is...imgui-rs on WGPU!"));
            ui.separator();
            let mouse_pos = ui.imgui().mouse_pos();
            ui.text(im_str!(
                "Mouse Position: ({:.1},{:.1})",
                mouse_pos.0,
                mouse_pos.1
            ));
        });

      ui.window(im_str!("Hello too"))
        .position((300.0, 300.0), ImGuiCond::FirstUseEver)
        .size((400.0, 200.0), ImGuiCond::FirstUseEver)
        .build(|| {
            ui.text(im_str!("Hello world!"));
        });
    }

    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

    renderer
      .render(ui, &mut device, &mut encoder, &frame.view)
      .expect("Rendering failed");

    device
      .get_queue()
      .submit(&[encoder.finish()]);
  }
}
