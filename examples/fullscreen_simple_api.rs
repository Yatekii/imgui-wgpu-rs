// you need to set --feature=simple_api_unstable to run this example
// cargo run --example fullscreen_simple_api --features=simple_api_unstable

use imgui::{im_str, Condition};
use imgui_wgpu::simple_api;

struct State {
    last_frame: std::time::Instant,
    height: f32,
    width: f32,
    high_dpi_factor: f64,
}

fn main() {
    let config = simple_api::Config {
        on_resize: &|input, state: &mut State, hdpi| {
            state.height = input.height as f32;
            state.width = input.width as f32;
            state.high_dpi_factor = hdpi;
        },
        ..Default::default()
    };

    let state = State {
        last_frame: std::time::Instant::now(),
        height: 100.0,
        width: 100.0,
        high_dpi_factor: 2.0,
    };

    imgui_wgpu::simple_api::run(config, state, |ui, state| {
        let now = std::time::Instant::now();

        imgui::Window::new(im_str!("full-window example"))
            .position([0.0, 0.0], Condition::Always)
            .collapsible(false)
            .resizable(false)
            .size(
                [
                    state.width / state.high_dpi_factor as f32,
                    state.height / state.high_dpi_factor as f32,
                ],
                Condition::Always,
            )
            .menu_bar(true)
            .build(&ui, || {
                ui.text(im_str!("Hello world!"));
            });

        state.last_frame = now;
    });
}
