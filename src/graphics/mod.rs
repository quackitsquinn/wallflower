use std::{
    borrow::Cow, cell::RefCell, mem::MaybeUninit, num::NonZeroU32, ops::Deref, rc::Rc, sync::Arc,
    time::Duration,
};

use anyhow::Context;
use image::{ColorType, DynamicImage, GenericImage, GenericImageView};
use winit::{event_loop::ActiveEventLoop, window::Window};

pub mod display_surface;
pub use display_surface::DisplaySurface;

#[derive(Clone)]
pub struct Graphics {
    pub window: Arc<Window>,
    pub instance: w::Instance,
    pub adapter: w::Adapter,
    pub device: w::Device,
    pub queue: w::Queue,
    display_surface: RefCell<DisplaySurface>,
}

impl Graphics {
    pub fn request_redraw(&self) -> anyhow::Result<()> {
        self.window.request_redraw();

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

pub fn init_graphics(_: &ActiveEventLoop, wind: Window) -> anyhow::Result<Graphics> {
    let instance = w::Instance::new(w::InstanceDescriptor::new_without_display_handle());
    let window = Arc::new(wind);

    let surface = instance
        .create_surface(window.clone())
        .with_context(|| "failed to create surface")?;

    let adapter = instance.request_adapter(&w::RequestAdapterOptions {
        power_preference: w::PowerPreference::LowPower,
        compatible_surface: Some(&surface),
        ..Default::default()
    });
    let adapter = smol::block_on(adapter).with_context(|| "failed to request adapter")?;

    let device = adapter.request_device(&w::DeviceDescriptor::default());
    let (device, queue) = smol::block_on(device).with_context(|| "failed to request device")?;

    let display_surface = DisplaySurface::init(&adapter, device.clone(), surface, window.clone())
        .with_context(|| "failed to initialize display surface")?;

    let gfx = Graphics {
        window,
        instance,
        adapter,
        device,
        queue,
        display_surface: RefCell::new(display_surface),
    };

    Ok(gfx)
}

/// A clock that can be used to trigger updates at a certain frequency.
///
/// The accuracy of the clock is not guaranteed and is best effort. It should, on average, trigger at the requested frequency,
/// but may be delayed or early due to system load or other factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    /// A clock that triggers with the global update clock. The speed of this clock is unspecified, so using delta time is recommended.
    ///
    /// This clock is probably the most energy efficient, as the global update clock will intentionally be throttled to save energy.
    Global,
    /// A clock that triggers at a certain frequency, starting at the next moment in time that is a multiple of the frequency.
    Every {
        /// The frequency at which the clock should trigger.
        frequency: Duration,
        /// Whether the clock should trigger immediately, or wait until the next multiple of the frequency.
        wait: bool,
    },
    /// A clock that will trigger at the exact moment in time that is requested with `.0`. After that, the clock will be switched to `Clock::Stopped`.
    Snoozed(Duration),
    /// A clock that will not trigger at all.
    Stopped,
}

pub struct RenderScheduler {
    tracking_clock: Clock,
    specified_clock: Clock,
}

impl RenderScheduler {
    pub fn new(specified_clock: Clock) -> Self {
        Self {
            tracking_clock: specified_clock.clone(),
            specified_clock,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// The base position of the viewport in our coordinate system (0, 0 is the top left corner of the window).
    pub base: glm::Vec2,
    /// The size of the viewport in our coordinate system.
    pub size: glm::Vec2,
}

pub struct Layer {
    vp: Viewport,
    img: Box<dyn ImageSource>,
    blend: w::BlendState,
}

pub trait ImageSource {
    fn next_image(&mut self) -> Option<w::Texture>;
    fn current_image(&self) -> Option<&w::Texture>;
}

// These are all `Arc`s because we probably won't ever be writing to a LoadedImage. Down the line there *might*
// be a feature or two that would require writing to an image, but that should probably be handled elsewhere.
pub type LumaImage = image::ImageBuffer<image::Luma<u16>, Arc<[u16]>>;
pub type LumaAlphaImage = image::ImageBuffer<image::LumaA<u16>, Arc<[u16]>>;
pub type RgbImage = image::ImageBuffer<image::Rgb<f32>, Arc<[f32]>>;
pub type RgbaImage = image::ImageBuffer<image::Rgba<f32>, Arc<[f32]>>;

pub enum LoadedImage {
    Luma(LumaImage),
    LumaA(LumaAlphaImage),
    Rgb(RgbImage),
    RgbA(RgbaImage),
}

pub fn load_image_from_bytes(bytes: &[u8]) -> anyhow::Result<LoadedImage> {
    let img = image::load_from_memory(bytes).with_context(|| "failed to load image from memory")?;
    arc_image_from_dyn_image(img).with_context(|| "failed to convert image to Arc")
}

fn arc_image_from_dyn_image(img: DynamicImage) -> anyhow::Result<LoadedImage> {
    fn conv_to_arc<P: image::Pixel, C: Deref<Target = [P::Subpixel]>>(
        img: image::ImageBuffer<P, C>,
    ) -> anyhow::Result<image::ImageBuffer<P, Arc<[P::Subpixel]>>> {
        let (w, h) = img.dimensions();
        let new_container = Arc::from(&*img.into_raw());
        image::ImageBuffer::from_raw(w, h, new_container)
            .with_context(|| "failed to create image buffer from raw data")
    }

    match img.color() {
        ColorType::L8 | ColorType::L16 => {
            let buf = conv_to_arc(img.into_luma16())?;
            Ok(LoadedImage::Luma(buf))
        }
        ColorType::La8 | ColorType::La16 => {
            let buf = conv_to_arc(img.into_luma_alpha16())?;
            Ok(LoadedImage::LumaA(buf))
        }
        ColorType::Rgb8 | ColorType::Rgb16 | ColorType::Rgb32F => {
            let buf = conv_to_arc(img.into_rgb32f())?;
            Ok(LoadedImage::Rgb(buf))
        }
        ColorType::Rgba8 | ColorType::Rgba16 | ColorType::Rgba32F => {
            let buf = conv_to_arc(img.into_rgba32f())?;
            Ok(LoadedImage::RgbA(buf))
        }
        _ => anyhow::bail!("unsupported image color type: {:?}", img.color()),
    }
}
