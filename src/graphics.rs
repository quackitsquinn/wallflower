use std::{rc::Rc, sync::Arc};

use anyhow::Context;
use wgpu::{self as w, Surface};
use winit::{event_loop::ActiveEventLoop, window::Window};

pub struct Graphics {
    pub window: Arc<Window>,
    pub instance: w::Instance,
    pub adapter: w::Adapter,
    pub surface: Surface<'static>,
    pub device: w::Device,
    pub queue: w::Queue,
}

impl Graphics {
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

pub fn init_graphics(_: &ActiveEventLoop, wind: Window) -> anyhow::Result<Graphics> {
    let instance = w::Instance::new(w::InstanceDescriptor::new_without_display_handle());
    let window = Arc::new(wind);
    let surface = instance
        .create_surface(window.clone())
        .with_context(|| "failed to create window surface")?;

    let adapter = instance.request_adapter(&w::RequestAdapterOptions {
        power_preference: w::PowerPreference::LowPower,
        compatible_surface: Some(&surface),
        ..Default::default()
    });
    let adapter = smol::block_on(adapter).with_context(|| "failed to request adapter")?;

    let device = adapter.request_device(&w::DeviceDescriptor::default());
    let (device, queue) = smol::block_on(device).with_context(|| "failed to request device")?;

    Ok(Graphics {
        window,
        instance,
        adapter,
        surface,
        device,
        queue,
    })
}
