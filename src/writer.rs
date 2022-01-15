use binrw::{prelude::*, NullString};
use image::GenericImageView;
use std::io::prelude::*;
use std::{
    convert::{Into, TryFrom, TryInto},
    error::Error,
};
use tegra_swizzle::{block_height_mip0, div_round_up, mip_block_height, swizzle_block_linear};

use crate::{NutexbFile, NutexbFooter, NutexbFormat};

// TODO: It should be possible to make a NutexbFile from anything that is ToNutexb.
// This avoids having to write the data somewhere.

/// A trait for creating a Nutexb from unswizzled image data.
/// Implement this trait for an image type to support writing a nutexb file with [write_nutexb].
pub trait ToNutexb {
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn depth(&self) -> u32;

    /// The raw image data for each mipmap layer before applying any swizzling.
    fn mipmaps(&self) -> Result<Vec<Vec<u8>>, Box<dyn Error>>;

    fn image_format(&self) -> Result<NutexbFormat, Box<dyn Error>>;
}

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
        // TODO: There are probably other DDS formats compatible with Nutexb.
        match value {
            ddsfile::DxgiFormat::R8G8B8A8_UNorm => Ok(NutexbFormat::R8G8B8A8Unorm),
            ddsfile::DxgiFormat::R8G8B8A8_UNorm_sRGB => Ok(NutexbFormat::R8G8B8A8Srgb),
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

/// Creates a [NutexbFile] with the nutexb string set to `name` and writes its data to `writer`.
/// The result of [ToNutexb::mipmaps] is swizzled according to the specified dimensions and format.
pub fn write_nutexb<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    let width = image.width();
    let height = image.height();
    let depth = image.depth();

    let image_format = image.image_format()?;
    let bytes_per_pixel = image_format.bytes_per_pixel();
    let block_width = image_format.block_width();
    let block_height = image_format.block_height();
    // TODO: Support 3D textures.
    let block_depth = image_format.block_depth();

    let mipmaps = image.mipmaps()?;

    // Mip sizes use the size before swizzling.
    let mip_sizes: Vec<u32> = mipmaps.iter().map(|m| m.len() as u32).collect();
    let mip_count = mipmaps.len() as u32;

    let data = swizzle_mipmaps_to_data(
        height as usize,
        block_width as usize,
        block_height as usize,
        mipmaps,
        width as usize,
        bytes_per_pixel as usize,
    );

    let size = data.len() as u32;
    NutexbFile {
        data,
        footer: NutexbFooter {
            mip_sizes,
            string: NullString::from_string(name.into()),
            width,
            height,
            depth,
            image_format,
            unk2: 4,
            mip_count,
            alignment: 0x1000,
            array_count: 1,
            data_size: size,
            version: (1, 2),
        },
    }
    .write_to(writer)?;
    Ok(())
}

fn swizzle_mipmaps_to_data(
    height: usize,
    block_width: usize,
    block_height: usize,
    mipmaps: Vec<Vec<u8>>,
    width: usize,
    bytes_per_pixel: usize,
) -> Vec<u8> {
    // Combine all the mipmaps into one contiguous region.
    let mut data = Vec::new();
    let block_height_mip0 = block_height_mip0(div_round_up(height, block_height));
    for (i, mip) in mipmaps.into_iter().enumerate() {
        let mip_width = div_round_up(width >> i, block_width);
        let mip_height = div_round_up(height >> i, block_height);

        // The block height will likely change for each mip level.
        let mip_block_height = mip_block_height(mip_height, block_height_mip0);

        let swizzled_mipmap = swizzle_block_linear(
            mip_width,
            mip_height,
            1,
            &mip,
            mip_block_height,
            bytes_per_pixel,
        )
        .unwrap();

        data.extend_from_slice(&swizzled_mipmap);
    }

    data
}

/// Writes a nutexb without any swizzling. Prefer [write_nutexb] for better memory access performance in most cases.
///
/// Textures created with [write_nutexb] use a memory layout optimized for the Tegra X1 with better access performance in the general case.
/// This function exists for the rare case where swizzling the image data is not desired for performance or compatibility reasons.
pub fn write_nutexb_unswizzled<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    let width = image.width();
    let height = image.height();
    let depth = image.depth();

    // TODO: Mipmaps
    let data = image.mipmaps()?[0].clone();

    let size = data.len() as u32;
    NutexbFile {
        data,
        footer: NutexbFooter {
            mip_sizes: vec![size as u32],
            string: NullString::from_string(name.into()),
            width,
            height,
            depth,
            image_format: image.image_format()?,
            unk2: 2,
            mip_count: 1,
            alignment: 0,
            array_count: 1,
            data_size: size,
            version: (2, 0),
        },
    }
    .write_to(writer)
    .map_err(Into::into)
}
