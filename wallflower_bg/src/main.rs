use wallflower_bg::{
    Wallflower,
    runner::{self, FloatingWindow, WindowRunConfig},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    runner::run_wayland().unwrap()
}
