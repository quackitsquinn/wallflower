use std::{rc::Rc, sync::Arc, time::Duration};

use petal::{FrameInfo, Graphics, w::SurfaceTarget};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

mod plat;
pub mod runner;

pub struct Wallflower {
    gfx: Graphics,
}

impl Wallflower {
    pub fn new<S>(target: S) -> anyhow::Result<Self>
    where
        S: Into<SurfaceTarget<'static>>,
    {
        let gfx = petal::init_graphics(target, (1000, 1000))?;
        Ok(Self { gfx })
    }

    pub fn render(&self) -> Duration {
        let then = std::time::Instant::now();
        self.gfx.render_frame(&FrameInfo { frame: 0 }).unwrap();

        Duration::from_secs_f32(1.0 / 60.0).saturating_sub(then.elapsed())
    }
}
