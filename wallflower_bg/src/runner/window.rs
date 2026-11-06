use std::{rc::Rc, sync::Arc};

use petal::{FrameInfo, Graphics};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

use crate::{Wallflower, plat};

pub struct WindowRunner {
    cfg: WindowRunConfig,
    window: Option<Arc<Window>>,
    wf: Option<Wallflower>,
}

impl WindowRunner {
    fn new(cfg: WindowRunConfig) -> Self {
        Self {
            cfg,
            window: None,
            wf: None,
        }
    }

    fn redraw(&mut self) {
        if let Some(wf) = self.wf.as_ref() {
            let requested_duration = wf.render();
            let window = self.window.as_ref().unwrap();
            window.request_redraw();
        }
    }

    fn window_attributes(&self, el: &ActiveEventLoop) -> WindowAttributes {
        let mut wa = if self.cfg.floating.is_none() {
            WindowAttributes::default()
                .with_position(PhysicalPosition::new(0, 0))
                .with_window_level(WindowLevel::AlwaysOnBottom)
                .with_resizable(false)
        } else {
            WindowAttributes::default()
        };

        let monitor_size: Size = match el.primary_monitor() {
            Some(m) => m.size().into(),
            None => Size::new(PhysicalSize::new(1000, 1000)),
        };

        let size = self
            .cfg
            .floating
            .as_ref()
            .and_then(|f| f.requested_resolution)
            .unwrap_or(monitor_size);

        wa = wa.with_inner_size(size);
        wa = plat::cfg_window_attrib_platform(wa);

        wa
    }
}

impl ApplicationHandler for WindowRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(self.window_attributes(event_loop))
            .unwrap();
        plat::cfg_window_platform(&window);
        window.set_outer_position(PhysicalPosition::new(0, 0));
        let window = Arc::new(window);
        self.window = Some(window.clone());
        self.wf = Some(Wallflower::new(window.clone()).unwrap());
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
                self.redraw();
            }
            _ => (),
        }
    }
}

pub struct WindowRunConfig {
    /// Should the background be rendered as a normal window? Useful for testing.
    ///
    /// `!floating` is not supported on Wayland, and will be ignored on that platform.
    pub floating: Option<FloatingWindow>,
}

pub struct FloatingWindow {
    /// The resolution to use for the window. If `None`, the window will be the resolution of the primary monitor.
    pub requested_resolution: Option<Size>,
}

pub fn run_windowed(wrc: WindowRunConfig) -> anyhow::Result<()> {
    let mut runner = WindowRunner::new(wrc);
    let event_loop = EventLoop::new().unwrap();

    // Power efficiency is important so we want to wait for events instead of polling.
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(&mut runner)?;

    Ok(())
}
