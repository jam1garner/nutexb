use std::{
    convert::{TryFrom, TryInto},
    error::Error,
};

use crate::{NutexbFormat, ToNutexb};

impl ToNutexb for ddsfile::Dds {
    fn width(&self) -> u32 {
        self.get_width()
    }

    fn height(&self) -> u32 {
        self.get_height()
    }

    fn depth(&self) -> u32 {
        // No depth for a 2d image.
        1
    }

    fn mipmaps(&self) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        // TODO: How to test this?
        let mut mipmaps = Vec::new();

        // DDS doesn't encode the mip offsets or sizes directly.
        // Assume no padding between mipmaps to allow for calculating offsets.
        let mut mip_offset = 0;
        let base_size = self.get_main_texture_size().unwrap();

        for mip in 0..self.get_num_mipmap_levels() {
            // Halve width and height for each mip level after the base level.
            // The minimum mipmap size depends on the format.
            let mip_size =
                std::cmp::max(base_size >> (2 * mip), self.get_min_mipmap_size_in_bytes()) as usize;
            mipmaps.push(self.data[mip_offset..mip_offset + mip_size].to_vec());

            mip_offset += mip_size;
        }

        // TODO: Error if mip offset does not equal data size at this point?

        Ok(mipmaps)
    }

    fn image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
        let format = self.get_dxgi_format().unwrap().try_into()?;
        Ok(format)
    }
}

impl TryFrom<ddsfile::DxgiFormat> for NutexbFormat {
    type Error = String;

    fn try_from(value: ddsfile::DxgiFormat) -> Result<Self, Self::Error> {
        // DDS supports all the known nutexb formats.
        match value {
            ddsfile::DxgiFormat::R8_UNorm => Ok(NutexbFormat::R8Unorm),
            ddsfile::DxgiFormat::R8G8B8A8_UNorm => Ok(NutexbFormat::R8G8B8A8Unorm),
            ddsfile::DxgiFormat::R8G8B8A8_UNorm_sRGB => Ok(NutexbFormat::R8G8B8A8Srgb),
            ddsfile::DxgiFormat::B8G8R8A8_UNorm => Ok(NutexbFormat::B8G8R8A8Unorm),
            ddsfile::DxgiFormat::B8G8R8A8_UNorm_sRGB => Ok(NutexbFormat::B8G8R8A8Srgb),
            ddsfile::DxgiFormat::BC1_UNorm => Ok(NutexbFormat::BC1Unorm),
            ddsfile::DxgiFormat::BC1_UNorm_sRGB => Ok(NutexbFormat::BC1Srgb),
            ddsfile::DxgiFormat::BC2_UNorm => Ok(NutexbFormat::BC2Unorm),
            ddsfile::DxgiFormat::BC2_UNorm_sRGB => Ok(NutexbFormat::BC2Srgb),
            ddsfile::DxgiFormat::BC3_UNorm => Ok(NutexbFormat::BC3Unorm),
            ddsfile::DxgiFormat::BC3_UNorm_sRGB => Ok(NutexbFormat::BC3Srgb),
            ddsfile::DxgiFormat::BC4_UNorm => Ok(NutexbFormat::BC4Unorm),
            ddsfile::DxgiFormat::BC4_SNorm => Ok(NutexbFormat::BC4Snorm),
            ddsfile::DxgiFormat::BC5_UNorm => Ok(NutexbFormat::BC5Unorm),
            ddsfile::DxgiFormat::BC5_SNorm => Ok(NutexbFormat::BC5Snorm),
            ddsfile::DxgiFormat::BC7_UNorm => Ok(NutexbFormat::BC7Unorm),
            ddsfile::DxgiFormat::BC7_UNorm_sRGB => Ok(NutexbFormat::BC7Srgb),
            _ => Err(format!(
                "{:?} is not a supported Nutexb image format.",
                value
            )),
        }
    }
}
