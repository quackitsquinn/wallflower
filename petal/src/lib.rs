use std::{
    borrow::Cow, cell::RefCell, mem::MaybeUninit, num::NonZeroU32, ops::Deref, rc::Rc, sync::Arc,
    time::Duration,
};

use anyhow::Context;

use w::rwh;
use winit::{event_loop::ActiveEventLoop, window::Window};

pub mod display_surface;
pub use display_surface::DisplaySurface;

pub mod composite;
pub mod img;
pub use crate::img::LoadedImage;

#[derive(Clone)]
pub struct Graphics {
    pub instance: w::Instance,
    pub adapter: w::Adapter,
    pub device: w::Device,
    pub queue: w::Queue,
    display_surface: RefCell<DisplaySurface>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameInfo {
    pub frame: u64,
}

impl Graphics {
    pub fn render_frame(&self, info: &FrameInfo) -> anyhow::Result<()> {
        let mut enc = self.create_encoder("main");
        let surf = self.display_surface.borrow();
        let (surf_txt, view) = match surf
            .current_texture_view()
            .with_context(|| "failed to get view")?
        {
            Some(v) => v,
            None => {
                surf.reconfigure();
                return Ok(());
            }
        };

        let shade = ((info.frame as f64 * 0.1).sin() + glm::pi::<f64>()) / glm::two_pi::<f64>();
        let color_attachment = w::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: w::Operations {
                load: w::LoadOp::Clear(w::Color {
                    r: shade,
                    g: shade,
                    b: shade,
                    a: 1.0,
                }),
                store: w::StoreOp::Store,
            },
            depth_slice: None,
        };

        let pass = enc.begin_pass(
            "main",
            &RenderPassConfig {
                color_attachments: vec![Some(color_attachment)],
                ..Default::default()
            },
        );

        drop(pass);

        self.queue.submit(Some(enc.finish()));
        self.queue.present(surf_txt);
        Ok(())
    }

    pub fn create_encoder(&self, label: &str) -> Encoder {
        Encoder::new(&self.device, label)
    }
}

pub struct Encoder {
    encoder: w::CommandEncoder,
}

impl Encoder {
    fn new(device: &w::Device, label: &str) -> Self {
        let encoder =
            device.create_command_encoder(&w::CommandEncoderDescriptor { label: Some(label) });
        Self { encoder }
    }

    pub fn begin_pass<'a>(&'a mut self, label: &str, cfg: &RenderPassConfig<'a>) -> RenderPass<'a> {
        let render_pass = self.encoder.begin_render_pass(&cfg.build_descriptor(label));
        RenderPass { render_pass }
    }

    pub fn finish(self) -> w::CommandBuffer {
        self.encoder.finish()
    }
}

pub struct RenderPass<'a> {
    render_pass: w::RenderPass<'a>,
}

#[derive(Debug, Clone, Default)]
pub struct RenderPassConfig<'a> {
    pub color_attachments: Vec<Option<w::RenderPassColorAttachment<'a>>>,
    pub depth_stencil_attachment: Option<w::RenderPassDepthStencilAttachment<'a>>,
    pub occlusion_query_set: Option<&'a w::QuerySet>,
    pub timestamp_writes: Option<w::RenderPassTimestampWrites<'a>>,
    pub multiview_mask: Option<NonZeroU32>,
}

impl<'a> RenderPassConfig<'a> {
    pub fn build_descriptor(&'a self, label: &'a str) -> w::RenderPassDescriptor<'a> {
        w::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &self.color_attachments,
            depth_stencil_attachment: self.depth_stencil_attachment.clone(),
            occlusion_query_set: self.occlusion_query_set,
            timestamp_writes: self.timestamp_writes.clone(),
            multiview_mask: self.multiview_mask,
        }
    }
}

pub fn init_graphics(
    target: impl Into<w::SurfaceTarget<'static>>,
    window_size: (u32, u32),
) -> anyhow::Result<Graphics> {
    // TODO: display handle will be important for multiple displays, but for now we can just use the default display.
    let instance = w::Instance::new(w::InstanceDescriptor::new_without_display_handle());
    let surface = instance
        .create_surface(target)
        .with_context(|| "failed to create surface")?;

    let adapter = instance.request_adapter(&w::RequestAdapterOptions {
        power_preference: w::PowerPreference::LowPower,
        compatible_surface: Some(&surface),
        ..Default::default()
    });
    let adapter = smol::block_on(adapter).with_context(|| "failed to request adapter")?;

    let device = adapter.request_device(&w::DeviceDescriptor::default());
    let (device, queue) = smol::block_on(device).with_context(|| "failed to request device")?;

    let display_surface = DisplaySurface::init(&adapter, device.clone(), surface, window_size)
        .with_context(|| "failed to initialize display surface")?;

    let gfx = Graphics {
        instance,
        adapter,
        device,
        queue,
        display_surface: RefCell::new(display_surface),
    };

    Ok(gfx)
}

/// Initializes graphics using wgpu's unsafe surface creation. Used for Wayland layer support.
///
/// # Safety
/// - See respective [`w::SurfaceTargetUnsafe`] variants for safety requirements.
pub unsafe fn init_graphics_raw(
    target: w::SurfaceTargetUnsafe,
    surface_size: (u32, u32),
) -> anyhow::Result<Graphics> {
    // TODO: display handle will be important for multiple displays, but for now we can just use the default display.
    let instance = w::Instance::new(w::InstanceDescriptor::new_without_display_handle());
    let surface = unsafe { instance.create_surface_unsafe(target) }
        .with_context(|| "failed to create surface")?;

    let adapter = instance.request_adapter(&w::RequestAdapterOptions {
        power_preference: w::PowerPreference::LowPower,
        compatible_surface: Some(&surface),
        ..Default::default()
    });
    let adapter = smol::block_on(adapter).with_context(|| "failed to request adapter")?;

    let device = adapter.request_device(&w::DeviceDescriptor::default());
    let (device, queue) = smol::block_on(device).with_context(|| "failed to request device")?;

    let display_surface = DisplaySurface::init(&adapter, device.clone(), surface, surface_size)
        .with_context(|| "failed to initialize display surface")?;

    let gfx = Graphics {
        instance,
        adapter,
        device,
        queue,
        display_surface: RefCell::new(display_surface),
    };

    Ok(gfx)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// The base position of the viewport in our coordinate system (0, 0 is the top left corner of the window).
    pub base: glm::Vec2,
    /// The size of the viewport in our coordinate system.
    pub size: glm::Vec2,
}

pub enum LayerContents {
    SolidColor(w::Color),
    Image(LoadedImage),
}

pub struct Layer {
    vp: Viewport,
    contents: LayerContents,
    blend: w::BlendState,
}
