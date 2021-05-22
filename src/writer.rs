use binwrite::BinWrite;
use image::GenericImageView;
use std::io::prelude::*;
use std::{
    convert::{Into, TryFrom, TryInto},
    error::Error,
};

use crate::NutexbFormat;

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

    // TODO: This should just use a Nutexb enum to avoid forcing dds as a dependency.
    // TODO: This function should probably return an error if the format can't be converted.
    // TODO: Have the format implement std::from for the Nutexb enum.
    fn try_get_image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
        let format = self.get_dxgi_format().unwrap().try_into()?;
        Ok(format)
    }
}

#[derive(BinWrite)]
struct NutexbFile {
    data: Vec<u8>,
    footer: NutexbFooter,
}

#[derive(BinWrite)]
struct NutexbFooter {
    #[binwrite(align_after(0x40))]
    mip_sizes: Vec<u32>,

    string_magic: [u8; 4],

    #[binwrite(align_after(0x40))]
    string: String,

    #[binwrite(pad(4))]
    width: u32,
    height: u32,
    depth: u32,

    #[binwrite(preprocessor(format_to_byte))]
    image_format: NutexbFormat,

    #[binwrite(pad_after(0x2))]
    unk: u8, // 4?

    unk2: u32,
    mip_count: u32,
    alignment: u32,
    array_count: u32,
    size: u32,

    tex_magic: [u8; 4],
    version_stuff: (u16, u16),
}

fn format_to_byte(format: NutexbFormat) -> u8 {
    format as u8
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

pub fn write_nutexb<W: Write, S: Into<String>, N: ToNutexb>(
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

    let bpp = image.get_bytes_per_pixel();

    let data = super::tegra_swizzle::swizzle(
        width,
        height,
        depth,
        /*blk_width and height*/ block_width,
        block_height,
        block_depth,
        false,
        bpp,
        /*tile_mode*/ 0,
        if width <= 64 && height <= 64 { 3 } else { 4 },
        &image.get_image_data(),
    );

    let size = data.len() as u32;
    NutexbFile {
        data,
        footer: NutexbFooter {
            mip_sizes: vec![size as u32],
            string_magic: *b" XNT",
            string: name.into(),
            width,
            height,
            depth,
            image_format: image.try_get_image_format()?,
            unk: 4,
            unk2: 4,
            mip_count: 1,
            alignment: 0x1000,
            array_count: 1,
            size,
            tex_magic: *b" XET",
            version_stuff: (1, 2),
        },
    }
    .write(writer)?;
    Ok(())
}
