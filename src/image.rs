use image::GenericImageView;
use std::error::Error;

use crate::{NutexbFormat, ToNutexb};

impl ToNutexb for image::DynamicImage {
    fn width(&self) -> u32 {
        self.dimensions().0
    }

    fn height(&self) -> u32 {
        self.dimensions().1
    }

    fn depth(&self) -> u32 {
        // No depth for a 2d image.
        1
    }

    fn mipmaps(&self) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        Ok(vec![self.to_rgba8().into_raw()])
    }

    fn image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
        Ok(NutexbFormat::R8G8B8A8Srgb)
    }
}
