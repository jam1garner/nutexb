use std::convert::Into;
use std::io::{self, prelude::*};
use binwrite::BinWrite;

pub trait ToNutexb {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_depth(&self) -> u32;

    fn get_block_width(&self) -> u32;
    fn get_block_height(&self) -> u32;
    fn get_block_depth(&self) -> u32;
    
    // TODO: Return a reference to avoid an extra copy?
    fn get_image_data(&self) -> Vec<u8>;

    fn get_bytes_per_pixel(&self) -> u32;

    // TODO: This should just use a Nutexb enum to avoid forcing dds as a dependency.
    // TODO: This function should probably return an error if the format can't be converted.
    // TODO: Have the format implement std::from for the Nutexb enum.
    fn get_image_format(&self) -> ddsfile::DxgiFormat;
}

impl ToNutexb for image::DynamicImage {
    fn get_width(&self) -> u32 {
        // TODO: Avoid copy?
        self.to_rgba().width()
    }

    fn get_height(&self) -> u32 {
        // TODO: Avoid copy?
        self.to_rgba().height()
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
        self.to_rgba().into_raw()
    }

    fn get_bytes_per_pixel(&self) -> u32 {
        4 // RGBA
    }

    // TODO: This should just use a Nutexb enum to avoid forcing dds as a dependency.
    // TODO: This function should probably return an error if the format can't be converted.
    // TODO: Have the format implement std::from for the Nutexb enum.
    fn get_image_format(&self) -> ddsfile::DxgiFormat {
        ddsfile::DxgiFormat::R8G8B8A8_UNorm_sRGB
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
        // TODO: Avoid the copy.
        self.data.clone()
    }

    fn get_bytes_per_pixel(&self) -> u32 {
        16 // RGBA
    }

    // TODO: This should just use a Nutexb enum to avoid forcing dds as a dependency.
    // TODO: This function should probably return an error if the format can't be converted.
    // TODO: Have the format implement std::from for the Nutexb enum.
    fn get_image_format(&self) -> ddsfile::DxgiFormat {
        let format = self.get_dxgi_format().unwrap();
        if format != ddsfile::DxgiFormat::BC7_UNorm_sRGB {
            // TODO: return a result instead.
            panic!("{:?}", format);
        } else {
            format
        }
    }
}

#[derive(BinWrite)]
struct NutexbFile {
    data: Vec<u8>,
    footer: NutexbFooter
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
    image_format: ddsfile::DxgiFormat,

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

fn format_to_byte(format: ddsfile::DxgiFormat) -> u8 {
    use ddsfile::DxgiFormat as Dxgi;
    match format {
        Dxgi::R8G8B8A8_UNorm_sRGB => 0x5,
        Dxgi::BC7_UNorm_sRGB => 0xe5,
        _ => unreachable!()
    }
}

pub fn write_nutexb<W: Write, S: Into<String>, N: ToNutexb>(name: S, image: &N, writer: &mut W) -> io::Result<()> {
    let width = image.get_width();
    let height = image.get_height();
    let depth = image.get_depth();

    let block_width = image.get_block_width();
    let block_height = image.get_block_height();
    let block_depth = image.get_block_depth();

    let bpp = image.get_bytes_per_pixel();

    let data = super::tegra_swizzle::swizzle(
        width, height, depth, /*blk_width and height*/ block_width, block_height, block_depth,
        false, bpp, /*tile_mode*/ 0, 4, &image.get_image_data()
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
            image_format: image.get_image_format(),
            unk: 4,
            unk2: 4,
            mip_count: 1,
            alignment: 0x1000,
            array_count: 1,
            size,
            tex_magic: *b" XET",
            version_stuff: (1, 2)
        }
    }.write(writer)
}
