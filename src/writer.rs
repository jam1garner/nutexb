use std::convert::Into;
use std::io::{self, prelude::*};
use binwrite::BinWrite;

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
        Dxgi::BC7_UNorm_sRGB => 0xe5,
        _ => unreachable!()
    }
}

pub fn write_nutexb<W: Write, S: Into<String>>(name: S, dds: &ddsfile::Dds, writer: &mut W) -> io::Result<()> {
    let width = dds.get_width();
    let height = dds.get_height();
    let depth = dds.get_depth();
    let data = super::tegra_swizzle::swizzle(
        width, height, depth, /*blk_width and height*/ 4, 4, 0,
        false, 16, /*tile_mode*/ 0, 4, &dds.data
    );

    if dds.get_dxgi_format().unwrap() != ddsfile::DxgiFormat::BC7_UNorm_sRGB {
        return Err(io::Error::from(io::ErrorKind::InvalidInput))
    }

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
            image_format: dds.get_dxgi_format().unwrap(),
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
