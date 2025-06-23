mod app;
mod egui_tools;
mod executors;
mod gruvbox_egui;
mod gui;

#[cfg(unix)]
use tracy_client::Client;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    env_logger::init();
    #[cfg(unix)]
    Client::start();
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
