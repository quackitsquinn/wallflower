use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use image::{ColorType, DynamicImage, GenericImage, GenericImageView};

// These are all `Arc`s because we probably won't ever be writing to a LoadedImage. Down the line there *might*
// be a feature or two that would require writing to an image, but that should probably be handled elsewhere.
pub type LumaImage = image::ImageBuffer<image::Luma<u16>, Arc<[u16]>>;
pub type LumaAlphaImage = image::ImageBuffer<image::LumaA<u16>, Arc<[u16]>>;
pub type RgbImage = image::ImageBuffer<image::Rgb<f32>, Arc<[f32]>>;
pub type RgbaImage = image::ImageBuffer<image::Rgba<f32>, Arc<[f32]>>;

/// An enum that represents an image that has been loaded into memory. The image is stored as an `Arc` for fast cloning
#[derive(Debug, Clone)]
pub enum LoadedImage {
    /// A grayscale 16-bit image. 
    Luma(LumaImage),
    /// A grayscale 16-bit image with an alpha channel.
    LumaA(LumaAlphaImage),
    /// A 32-bit floating point RGB image.
    Rgb(RgbImage),
    /// A 32-bit floating point RGBA image.
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
