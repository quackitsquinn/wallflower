use wallflower::Wallflower;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // Power effeciency is important so we want to
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = Wallflower::default();
    event_loop.run_app(&mut app).unwrap();
}
