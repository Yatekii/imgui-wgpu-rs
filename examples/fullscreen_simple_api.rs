use imgui::{im_str, Condition};
use imgui_wgpu::simple_api;

struct State {
    last_frame: std::time::Instant,
    height: f32,
    width: f32,
    highdpi_factor: f64,
    plotcontext: implot::Context,
}

fn main() {
    let config = simple_api::Config {
        on_resize: &|input, state: &mut State, hdpi| {
            state.height = input.height as f32;
            state.width = input.width as f32;
            state.highdpi_factor = hdpi;
        },
        ..Default::default()
    };

    let plotcontext = implot::Context::create();

    let state = State {
        last_frame: std::time::Instant::now(),
        height: 100.0,
        width: 100.0,
        highdpi_factor: 2.0,
        plotcontext,
    };

    imgui_wgpu::simple_api::run(config, state, |ui, state| {
        let now = std::time::Instant::now();

        imgui::Window::new(im_str!("full-window example"))
            .position([0.0, 0.0], Condition::Always)
            .collapsible(false)
            .resizable(false)
            .size(
                [
                    state.width / state.highdpi_factor as f32,
                    state.height / state.highdpi_factor as f32,
                ],
                Condition::Always,
            )
            .menu_bar(true)
            .build(&ui, || {
                ui.text(im_str!("Hello world!"));

                // begin implot
                let plot_ui = &state.plotcontext.get_plot_ui();

                let content_width = ui.window_content_region_width();
                implot::Plot::new("Simple line plot")
                    // The size call could also be omitted, though the defaults don't consider window
                    // width, which is why we're not doing so here.
                    .size(content_width, 300.0)
                    .build(plot_ui, || {
                        // If this is called outside a plot build callback, the program will panic.
                        let x_positions = vec![0.1, 0.9];
                        let y_positions = vec![0.1, 0.9];
                        implot::PlotLine::new("legend label").plot(&x_positions, &y_positions);
                    });

                // end implot
            });

        state.last_frame = now;
    });
}
