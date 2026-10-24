use std::sync::Arc;

use winit::window::Window;

#[derive(Clone)]
pub struct DisplaySurface {
    device: w::Device,
    data: Arc<(w::Surface<'static>, w::SurfaceConfiguration)>,
}

pub struct DisplaySurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub format: w::TextureFormat,
}

impl DisplaySurface {
    pub fn init(
        adapter: &w::Adapter,
        device: w::Device,
        surface: w::Surface<'static>,
        size: (u32, u32),
    ) -> anyhow::Result<Self> {
        let surface_caps = surface.get_capabilities(adapter);
        let surface_format = surface_caps.formats[0];

        let config = w::SurfaceConfiguration {
            usage: w::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: w::SurfaceColorSpace::Auto,
        };

        let display_surface = DisplaySurface {
            device,
            data: Arc::new((surface, config)),
        };
        display_surface.reconfigure();
        Ok(display_surface)
    }

    pub fn reconfigure(&self) {
        let (surface, config) = &*self.data;
        surface.configure(&self.device, config);
    }

    pub fn current_texture(&self) -> anyhow::Result<Option<w::SurfaceTexture>> {
        match self.data.0.get_current_texture() {
            w::CurrentSurfaceTexture::Success(txt) => Ok(Some(txt)),
            w::CurrentSurfaceTexture::Timeout
            | w::CurrentSurfaceTexture::Occluded
            | w::CurrentSurfaceTexture::Validation => Ok(None),
            w::CurrentSurfaceTexture::Suboptimal(txt) => {
                self.reconfigure();
                Ok(Some(txt))
            }
            w::CurrentSurfaceTexture::Outdated | w::CurrentSurfaceTexture::Lost => {
                self.reconfigure();
                Ok(None)
            }
        }
    }

    /// Returns a texture view of the current surface texture, if available.
    pub fn current_texture_view(
        &self,
    ) -> anyhow::Result<Option<(w::SurfaceTexture, w::TextureView)>> {
        let txt = self.current_texture()?;
        let txt = match txt {
            Some(txt) => txt,
            None => return Ok(None),
        };
        let view = txt
            .texture
            .create_view(&w::TextureViewDescriptor::default());
        Ok(Some((txt, view)))
    }
}
