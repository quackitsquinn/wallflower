use std::time::Duration;

use petal::{
    FrameInfo, Graphics,
    w::{SurfaceTarget, SurfaceTargetUnsafe},
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

    /// Creates a new Wallflower from the given unsafe surface target.
    ///
    /// # Safety
    ///
    /// `target` must be valid for the current platform.
    pub unsafe fn new_unsafe_surface(target: SurfaceTargetUnsafe) -> anyhow::Result<Self> {
        let gfx = unsafe { petal::init_graphics_raw(target, (1000, 1000)) }?;
        Ok(Self { gfx })
    }

    pub fn render(&self) -> Duration {
        let then = std::time::Instant::now();
        self.gfx.render_frame(&FrameInfo { frame: 0 }).unwrap();

        Duration::from_secs_f32(1.0 / 60.0).saturating_sub(then.elapsed())
    }
}
