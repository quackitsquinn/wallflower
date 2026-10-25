use wallflower_bg::Wallflower;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    // Power efficiency is important so we want to wait for events instead of polling.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = Wallflower::default();
    event_loop.run_app(&mut app).unwrap();
}
