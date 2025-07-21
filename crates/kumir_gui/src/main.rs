mod app;
mod app_state;
mod egui_tools;
mod executors;
mod gruvbox_egui;
mod gui;

use log::info;
#[cfg(unix)]
use tracy_client::Client;

use log::Level;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(Level::Trace);
    }
    info!("Starting App");
    #[cfg(target_arch = "wasm32")]
    if {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            navigator.gpu().is_object()
        } else {
            // No global window! This is probably running from a headless environment.
            // e.g. `wasm-pack test --node`
            false
        }
    } {
        log::error!("This platform doesn't support WebGPU!");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(run());
    }
}

async fn run() {
    #[cfg(unix)]
    Client::start();
    let event_loop = EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );

    event_loop.run_app(&mut app).expect("Failed to run app");
}
