use std::{
    convert::{TryFrom, TryInto},
    error::Error,
};

use ddsfile::{Dds, DxgiFormat};
use tegra_swizzle::div_round_up;

use crate::{mipmaps::deswizzle_data_to_mipmaps, NutexbFile, NutexbFormat, ToNutexb};

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
        let width = self.header.width as usize;
        let height = self.header.height as usize;

        let image_format = self.image_format()?;
        let bytes_per_pixel = image_format.bytes_per_pixel() as usize;
        let block_width = image_format.block_width() as usize;
        let block_height = image_format.block_height() as usize;
        // TODO: Support 3D textures.
        let block_depth = image_format.block_depth() as usize;

        for mip in 0..self.get_num_mipmap_levels() {
            // Halve width and height for each mip level after the base level.
            // The minimum mipmap size depends on the format.
            let mip_width = div_round_up(width >> mip, block_width);
            let mip_height = div_round_up(height >> mip, block_height);

            let mip_size = mip_width * mip_height * bytes_per_pixel;
            let mip_size = std::cmp::max(mip_size, self.get_min_mipmap_size_in_bytes() as usize);

            mipmaps.push(self.data[mip_offset..mip_offset + mip_size].to_vec());

            mip_offset += mip_size;
        }

        // TODO: Error if mip offset does not equal data size at this point?
        dbg!(self.data.len(), mip_offset);
        Ok(mipmaps)
    }

    fn image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
        // TODO: Try dxgi format, then try d3d, then error?
        let format = self.get_dxgi_format().unwrap().try_into()?;
        Ok(format)
    }
}

impl TryFrom<DxgiFormat> for NutexbFormat {
    type Error = String;

    fn try_from(value: DxgiFormat) -> Result<Self, Self::Error> {
        // DDS supports all the known nutexb formats.
        match value {
            DxgiFormat::R8_UNorm => Ok(NutexbFormat::R8Unorm),
            DxgiFormat::R8G8B8A8_UNorm => Ok(NutexbFormat::R8G8B8A8Unorm),
            DxgiFormat::R8G8B8A8_UNorm_sRGB => Ok(NutexbFormat::R8G8B8A8Srgb),
            DxgiFormat::B8G8R8A8_UNorm => Ok(NutexbFormat::B8G8R8A8Unorm),
            DxgiFormat::B8G8R8A8_UNorm_sRGB => Ok(NutexbFormat::B8G8R8A8Srgb),
            DxgiFormat::BC1_UNorm => Ok(NutexbFormat::BC1Unorm),
            DxgiFormat::BC1_UNorm_sRGB => Ok(NutexbFormat::BC1Srgb),
            DxgiFormat::BC2_UNorm => Ok(NutexbFormat::BC2Unorm),
            DxgiFormat::BC2_UNorm_sRGB => Ok(NutexbFormat::BC2Srgb),
            DxgiFormat::BC3_UNorm => Ok(NutexbFormat::BC3Unorm),
            DxgiFormat::BC3_UNorm_sRGB => Ok(NutexbFormat::BC3Srgb),
            DxgiFormat::BC4_UNorm => Ok(NutexbFormat::BC4Unorm),
            DxgiFormat::BC4_SNorm => Ok(NutexbFormat::BC4Snorm),
            DxgiFormat::BC5_UNorm => Ok(NutexbFormat::BC5Unorm),
            DxgiFormat::BC5_SNorm => Ok(NutexbFormat::BC5Snorm),
            DxgiFormat::BC6H_UF16 => Ok(NutexbFormat::BC6Ufloat),
            DxgiFormat::BC6H_SF16 => Ok(NutexbFormat::BC6Sfloat),
            DxgiFormat::BC7_UNorm => Ok(NutexbFormat::BC7Unorm),
            DxgiFormat::BC7_UNorm_sRGB => Ok(NutexbFormat::BC7Srgb),
            _ => Err(format!(
                "{:?} is not a supported Nutexb image format.",
                value
            )),
        }
    }
}

impl From<NutexbFormat> for DxgiFormat {
    fn from(format: NutexbFormat) -> Self {
        match format {
            NutexbFormat::R8Unorm => DxgiFormat::R8_UNorm,
            NutexbFormat::R8G8B8A8Unorm => DxgiFormat::R8G8B8A8_UNorm,
            NutexbFormat::R8G8B8A8Srgb => DxgiFormat::R8G8B8A8_UNorm_sRGB,
            NutexbFormat::R32G32B32A32Float => DxgiFormat::R32G32B32A32_Float,
            NutexbFormat::B8G8R8A8Unorm => DxgiFormat::B8G8R8A8_UNorm,
            NutexbFormat::B8G8R8A8Srgb => DxgiFormat::B8G8R8A8_UNorm_sRGB,
            NutexbFormat::BC1Unorm => DxgiFormat::BC1_UNorm,
            NutexbFormat::BC1Srgb => DxgiFormat::BC1_UNorm_sRGB,
            NutexbFormat::BC2Unorm => DxgiFormat::BC2_UNorm,
            NutexbFormat::BC2Srgb => DxgiFormat::BC2_UNorm_sRGB,
            NutexbFormat::BC3Unorm => DxgiFormat::BC3_UNorm,
            NutexbFormat::BC3Srgb => DxgiFormat::BC3_UNorm_sRGB,
            NutexbFormat::BC4Unorm => DxgiFormat::BC4_UNorm,
            NutexbFormat::BC4Snorm => DxgiFormat::BC4_SNorm,
            NutexbFormat::BC5Unorm => DxgiFormat::BC5_UNorm,
            NutexbFormat::BC5Snorm => DxgiFormat::BC5_SNorm,
            NutexbFormat::BC6Ufloat => DxgiFormat::BC6H_UF16,
            NutexbFormat::BC6Sfloat => DxgiFormat::BC6H_SF16,
            NutexbFormat::BC7Unorm => DxgiFormat::BC7_UNorm,
            NutexbFormat::BC7Srgb => DxgiFormat::BC7_UNorm_sRGB,
        }
    }
}

// TODO: Support D3D format types as well?

pub fn create_dds(nutexb: &NutexbFile) -> Dds {
    // TODO: 3D Support.
    let mut dds = Dds::new_dxgi(
        nutexb.footer.height,
        nutexb.footer.width,
        None,
        nutexb.footer.image_format.into(),
        Some(nutexb.footer.mip_count),
        None,
        None,
        false,
        ddsfile::D3D10ResourceDimension::Texture2D,
        ddsfile::AlphaMode::Unknown, // TODO: Alpha mode?
    )
    .unwrap();

    dbg!(nutexb.footer.mip_count as usize);
    // DDS stores mipmaps in a contiguous region of memory.
    let combined_data = deswizzle_data_to_mipmaps(
        nutexb.footer.width as usize,
        nutexb.footer.height as usize,
        nutexb.footer.image_format.block_width() as usize,
        nutexb.footer.image_format.block_height() as usize,
        nutexb.footer.image_format.bytes_per_pixel() as usize,
        nutexb.footer.mip_count as usize,
        &nutexb.data,
    )
    .into_iter()
    .flatten()
    .collect();

    dds.data = combined_data;
    dbg!(dds.get_num_mipmap_levels());

    dds
}
