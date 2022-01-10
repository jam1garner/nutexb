use binrw::{prelude::*, NullString};
use image::GenericImageView;
use std::io::prelude::*;
use std::{
    convert::{Into, TryFrom, TryInto},
    error::Error,
};
use tegra_swizzle::{div_round_up, block_height_mip0, swizzle_block_linear};

use crate::{NutexbFormat, NutexbFile, NutexbFooter};

pub trait ToNutexb {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_depth(&self) -> u32;

    fn get_block_width(&self) -> u32;
    fn get_block_height(&self) -> u32;
    fn get_block_depth(&self) -> u32;

    fn get_image_data(&self) -> Vec<u8>;

    fn get_bytes_per_pixel(&self) -> u32;

    fn try_get_image_format(&self) -> Result<NutexbFormat, Box<dyn Error>>;
}

impl ToNutexb for image::DynamicImage {
    fn get_width(&self) -> u32 {
        self.dimensions().0
    }

    fn get_height(&self) -> u32 {
        self.dimensions().1
    }

    fn get_depth(&self) -> u32 {
        // No depth for a 2d image.
        1
    }

    // Uncompressed formats don't use block compression.
    fn get_block_width(&self) -> u32 {
        1
    }

    fn get_block_height(&self) -> u32 {
        1
    }

    fn get_block_depth(&self) -> u32 {
        1
    }

    fn get_image_data(&self) -> Vec<u8> {
        self.to_rgba8().into_raw()
    }

    fn get_bytes_per_pixel(&self) -> u32 {
        4 // RGBA
    }

    fn try_get_image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
        Ok(NutexbFormat::R8G8B8A8Srgb)
    }
}

impl ToNutexb for ddsfile::Dds {
    fn get_width(&self) -> u32 {
        self.get_width()
    }

    fn get_height(&self) -> u32 {
        self.get_height()
    }

    fn get_depth(&self) -> u32 {
        // No depth for a 2d image.
        1
    }

    // TODO: Support other formats
    fn get_block_width(&self) -> u32 {
        4
    }

    fn get_block_height(&self) -> u32 {
        4
    }

    fn get_block_depth(&self) -> u32 {
        0
    }

    fn get_image_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn get_bytes_per_pixel(&self) -> u32 {
        // TODO: Support other formats.
        16
    }

    fn try_get_image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
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

pub fn write_nutexb<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    let width = image.get_width();
    let height = image.get_height();
    let depth = image.get_depth();

    let block_width = image.get_block_width();
    let block_height = image.get_block_height();
    let block_depth = image.get_block_depth();

    let bytes_per_pixel = image.get_bytes_per_pixel();

    let block_height_mip0 = block_height_mip0(height as usize);
    let data = swizzle_block_linear(
        div_round_up(width as usize, block_width as usize),
        div_round_up(height as usize, block_height as usize),
        div_round_up(depth as usize, block_depth as usize),
        &image.get_image_data(),
        block_height_mip0,
        bytes_per_pixel as usize,
    ).unwrap();

    let size = data.len() as u32;
    NutexbFile {
        data,
        footer: NutexbFooter {
            mip_sizes: vec![size as u32],
            string: NullString::from_string(name.into()),
            width,
            height,
            depth,
            image_format: image.try_get_image_format()?,
            unk2: 4,
            mip_count: 1,
            alignment: 0x1000,
            array_count: 1,
            size,
            version: (1, 2),
        },
    }
    .write_to(writer)?;
    Ok(())
}

pub fn write_nutexb_unswizzled<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    let width = image.get_width();
    let height = image.get_height();
    let depth = image.get_depth();

    let data = image.get_image_data();

    let size = data.len() as u32;
    NutexbFile {
        data,
        footer: NutexbFooter {
            mip_sizes: vec![size as u32],
            string: NullString::from_string(name.into()),
            width,
            height,
            depth,
            image_format: image.try_get_image_format()?,
            unk2: 2,
            mip_count: 1,
            alignment: 0,
            array_count: 1,
            size,
            version: (2, 0),
        },
    }
    .write_to(writer)
    .map_err(Into::into)
}
