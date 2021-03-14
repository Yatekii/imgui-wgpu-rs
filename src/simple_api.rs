/*!
A simple API to get an imgui context in only a few lines of code.


This API only provides stability on a best efforts basis because its meant for small/ temporary projects like if you need to quickly plot something
and just need a context do some imgui work.

It aims to make updating the wgpu imgui bindings easier to use as it abstracts all the setup. This comes with the drawback of yet another API.

It is basically a wrapper around the hello world example with a few customization options.

The API consists of a Config which you may not need to touch and just use the Default one.
Optionally, you can provide your own Struct to have a place to store mutable state in your small application.

```no_run
fn main() {
    imgui_wgpu::simple_api::run(Default::default(), (), |ui, _| {
        imgui::Window::new(imgui::im_str!("hello world")).build(&ui, || {
            ui.text(imgui::im_str!("Hello world!"));
        });
    });
}
```
*/

use crate::{Renderer, RendererConfig};
use imgui::*;
use pollster::block_on;

use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/// use `Default::default` if you don't need anything specific.
pub struct Config<State: 'static> {
    /// name of the window
    pub window_title: String,
    /// can be used to resize the window
    pub initial_window_width: f32,
    /// can be used to resize the window
    pub initial_window_height: f32,
    /// if you want to adjust your imgui window to match the size of the outer window
    /// this makes it possible to have a "fullscreen" imgui window spanning the whole current window.
    pub on_resize: &'static dyn Fn(&winit::dpi::PhysicalSize<u32>, &mut State, f64),
    /// called after the premade events have been handled which includes close request
    /// if you think you need to handle this, this api abstraction is probably to high level
    /// and you may want to copy the code from hello_world.rs and adapt directly
    pub on_event: &'static dyn Fn(&winit::event::WindowEvent<'_>, &mut State),
    /// font size
    pub font_size: Option<f32>,
    /// color that fills the window
    pub background_color: wgpu::Color,
}

impl<State> Default for Config<State> {
    fn default() -> Self {
        Self {
            window_title: "imgui".to_string(),
            initial_window_width: 1200.0,
            initial_window_height: 720.0,
            on_resize: &|_, _, _| {},
            on_event: &|_, _| {},
            font_size: None,
            background_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }
}

/// simple function to draw imgui
pub fn run<YourState: 'static, UiFunction: 'static + Fn(&imgui::Ui, &mut YourState)>(
    mut imgui: imgui::Context,
    config: Config<YourState>,
    mut state: YourState,
    render_ui: UiFunction,
) {
    // Set up window and GPU
    let event_loop = EventLoop::new();

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let (window, size, surface) = {
        let window = Window::new(&event_loop).unwrap();
        window.set_inner_size(LogicalSize {
            width: config.initial_window_width,
            height: config.initial_window_height,
        });
        window.set_title(&config.window_title);
        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(&window) };

        (window, size, surface)
    };

    let hidpi_factor = window.scale_factor();

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) =
        block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

    // Set up swap chain
    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width as u32,
        height: size.height as u32,
        // limits refresh rate to the monitor's refresh rate, not wasting power spinning very quickly
        present_mode: wgpu::PresentMode::Fifo,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    // Set up dear imgui
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = config.font_size.unwrap_or((13.0 * hidpi_factor) as f32);
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
    let renderer_config = RendererConfig {
        texture_format: sc_desc.format,
        ..Default::default()
    };

    let mut renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

    let mut last_frame = Instant::now();

    let mut last_cursor = None;

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let size = window.inner_size();

                let sc_desc = wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: size.width as u32,
                    height: size.height as u32,
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                swap_chain = device.create_swap_chain(&surface, &sc_desc);

                (config.on_resize)(&size, &mut state, hidpi_factor);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
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
                *control_flow = ControlFlow::Exit;
            }

            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawEventsCleared => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;

                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.frame();

                render_ui(&ui, &mut state);

                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if last_cursor != Some(ui.mouse_cursor()) {
                    last_cursor = Some(ui.mouse_cursor());
                    platform.prepare_render(&ui, &window);
                }

                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(config.background_color),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                renderer
                    .render(ui.render(), &queue, &device, &mut rpass)
                    .expect("Rendering failed");

                drop(rpass);

                queue.submit(Some(encoder.finish()));
            }
            Event::WindowEvent { ref event, .. } => {
                (config.on_event)(event, &mut state);
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
