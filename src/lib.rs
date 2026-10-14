use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

mod plat;

pub struct Wallflower {
    window: Option<Window>,
}

impl Default for Wallflower {
    fn default() -> Self {
        Self { window: None }
    }
}

fn background_window_attribs(el: &ActiveEventLoop) -> WindowAttributes {
    let mut wa = WindowAttributes::default()
        .with_position(PhysicalPosition::new(0, 0))
        .with_window_level(WindowLevel::AlwaysOnBottom)
        .with_resizable(false);

    let monitor = el.primary_monitor().unwrap();
    let size = monitor.size();
    wa = wa.with_inner_size(size);

    wa = plat::cfg_window_attrib_platform(wa);

    wa
}

impl ApplicationHandler for Wallflower {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(background_window_attribs(event_loop))
            .unwrap();
        plat::cfg_window_platform(&window);
        window.set_outer_position(PhysicalPosition::new(0, 0));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Exiting!");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
