// you need to set --feature=simple_api_unstable to run this example
// cargo run --example basic_simple_api --features=simple_api_unstable

fn main() {
    imgui_wgpu_simple::run(Default::default(), (), |ui, _| {
        imgui::Window::new("hello world").build(&ui, || {
            ui.text("Hello world!");
        });
    });
}
