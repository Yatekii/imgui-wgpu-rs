// you need to set --feature=simple_api_unstable to run this example
// cargo run --example basic_simple_api --features=simple_api_unstable

fn main() {
    imgui_wgpu::simple_api::run(Default::default(), (), |ui, _| {
        imgui::Window::new(imgui::im_str!("hwllo world")).build(&ui, || {
            ui.text(imgui::im_str!("Hello world!"));
        });
    });
}
