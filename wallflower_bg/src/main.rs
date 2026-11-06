use wallflower_bg::{
    Wallflower,
    runner::{self, FloatingWindow, WindowRunConfig},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    let wrc = WindowRunConfig {
        floating: Some(FloatingWindow {
            requested_resolution: None,
        }),
    };

    runner::run_windowed(wrc).unwrap();
}
