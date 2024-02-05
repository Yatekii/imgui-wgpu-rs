use image::ImageFormat;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use pollster::block_on;
use std::time::Instant;
use wgpu::Extent3d;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, WindowEvent, KeyEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

fn main() {
    env_logger::init();

    // Set up window and GPU
    let event_loop = EventLoop::new().expect("Error creating event loop");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let (window, size) = {
        let version = env!("CARGO_PKG_VERSION");

        let window = Window::new(&event_loop).unwrap();
        let _ = window.request_inner_size(LogicalSize::new(1280.0, 720.0));
        window.set_title(&format!("imgui-wgpu {version}"));
        let size = window.inner_size();

        (window, size)
    };

    let surface = instance.create_surface(&window).unwrap();

    let hidpi_factor = window.scale_factor();

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    // Set up swap chain
    let surface_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        desired_maximum_frame_latency: 0,
    };

    surface.configure(&device, &surface_desc);

    // Set up dear imgui
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(imgui::FontConfig {
            oversample_h: 1,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
        }),
    }]);

    //
    // Set up dear imgui wgpu renderer
    //
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
    let renderer_config = RendererConfig {
        texture_format: surface_desc.format,
        ..Default::default()
    };

    let mut renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

    let mut last_frame = Instant::now();

    // Set up Lenna texture
    let lenna_bytes = include_bytes!("../resources/checker.png");
    let image =
        image::load_from_memory_with_format(lenna_bytes, ImageFormat::Png).expect("invalid image");
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();
    let raw_data = image.into_raw();

    let texture_config = TextureConfig {
        size: Extent3d {
            width,
            height,
            ..Default::default()
        },
        label: Some("lenna texture"),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        ..Default::default()
    };

    let texture = Texture::new(&device, &renderer, texture_config);

    texture.write(&queue, &raw_data, width, height);
    let lenna_texture_id = renderer.textures.insert(texture);

    let mut last_cursor = None;

    event_loop.set_control_flow(ControlFlow::Poll);

    // Event loop
    let _ = event_loop.run(|event, elwt| {
        if cfg!(feature = "metal-auto-capture") {
            elwt.exit();
        };
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let surface_desc = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
                    desired_maximum_frame_latency: 0,
                };

                surface.configure(&device, &surface_desc);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {e:?}");
                        return;
                    }
                };
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.frame();

                {
                    let size = [width as f32, height as f32];
                    let window = ui.window("Hello world");
                    window
                        .size([400.0, 600.0], Condition::FirstUseEver)
                        .build(|| {
                            ui.text("Hello textures!");
                            ui.text("Say hello to checker.png");
                            Image::new(lenna_texture_id, size).build(ui);
                        });
                }

                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if last_cursor != Some(ui.mouse_cursor()) {
                    last_cursor = Some(ui.mouse_cursor());
                    platform.prepare_render(ui, &window);
                }

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                renderer
                    .render(imgui.render(), &queue, &device, &mut rpass)
                    .expect("Rendering failed");

                drop(rpass);

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
