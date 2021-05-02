use binread::io::SeekFrom;
use binread::prelude::*;
use binread::NullString;

use binread::BinResult;

#[derive(BinRead, Debug, Clone)]
pub struct NutexbFile {
    #[br(seek_before = SeekFrom::End(-8))]
    pub footer: NutexbFooter,

    #[br(seek_before = SeekFrom::Start(0), count = footer.size)]
    pub data: Vec<u8>,
}

#[derive(BinRead, Debug, Clone)]
#[br(magic = b" XET")]
pub struct NutexbFooter {
    pub version: (u16, u16),

    #[br(seek_before = SeekFrom::End(-0x2C))]
    pub width: u32,
    pub height: u32,
    pub depth: u32,

    #[br(map = u8::into)]
    pub image_format: NutexbFormat,

    #[br(pad_after = 0x2)]
    pub unk: u8, // 4?

    pub unk2: u32,
    pub mip_count: u32,
    pub alignment: u32,
    pub array_count: u32,
    pub size: u32,

    #[br(seek_before = SeekFrom::End(-0xB0), count = mip_count)]
    pub mip_sizes: Vec<u32>,

    #[br(seek_before = SeekFrom::End(-0x70), magic = b" XNT", map = NullString::into_string)]
    pub string: String,
}

pub fn read_nutexb(path: &std::path::Path) -> Result<NutexbFile, Box<dyn std::error::Error>> {
    let mut file = std::io::Cursor::new(std::fs::read(path)?);
    let nutexb = file.read_le::<NutexbFile>()?;
    Ok(nutexb)
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum NutexbFormat {
    BC1_UNorm_sRGB,
    Unknown(u8),
}

impl From<u8> for NutexbFormat {
    fn from(num: u8) -> Self {
        match num {
            0x85 => NutexbFormat::BC1_UNorm_sRGB,
            _ => NutexbFormat::Unknown(num),
        }
    }
}

fn to_nutexb_format(num: u8) -> ddsfile::DxgiFormat {
    match num {
        0x85 => ddsfile::DxgiFormat::BC1_UNorm_sRGB,
        _ => ddsfile::DxgiFormat::Unknown,
    }
}

use std::io::{Read, Seek};

impl NutexbFile {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le::<NutexbFile>()
    }
}

use image::RgbaImage;

//    fn get_width(&self) -> u32 {
//        self.dimensions().0
//
//    fn get_height(&self) -> u32 {
//        self.dimensions().1
//
//    fn get_depth(&self) -> u32 {
//        // No depth for a 2d image.
//        1
//
//    // Uncompressed formats don't use block compression.
//    fn get_block_width(&self) -> u32 {
//        1
//
//    fn get_block_height(&self) -> u32 {
//        1
//
//    fn get_block_depth(&self) -> u32 {
//        1
//
//    fn get_image_data(&self) -> Vec<u8> {
//        self.to_rgba().into_raw()
//
//    fn get_bytes_per_pixel(&self) -> u32 {
//        4 // RGBA

impl NutexbFile {
    pub fn as_rgba8(&self) -> RgbaImage {
        let width = self.footer.width;
        let height = self.footer.height;
        //let depth = self.footer.depth;
        //let blk_width = 4;
        //let blk_height = 4;
        //let blk_depth = 1;
        //let round_pitch = false;
        //let bpp = 4;
        //let tile_mode = 0;

        //let buf = crate::tegra_swizzle::deswizzle(
        //    width,
        //    height,
        //    depth,
        //    blk_width,
        //    blk_height,
        //    blk_depth,
        //    round_pitch,
        //    bpp,
        //    tile_mode,
        //    8,
        //    &self.data
        //);

        let buf = &self.data;

        let buf = match self.footer.image_format {
            NutexbFormat::BC1_UNorm_sRGB => crate::bc1::decode_image_cmpr(
                &buf, width as _, height as _
            ),
            _ => todo!()
        };

        //let depth = self.footer.depth;
        //let blk_width = 1;
        //let blk_height = 1;
        //let blk_depth = 1;
        //let round_pitch = true;
        //let bpp = 4;
        //let tile_mode = 0;

        //buf.extend_from_slice(&vec![0; 300000]);

        //dbg!(buf.len());

        RgbaImage::from_raw(width, height, buf).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use binread::BinReaderExt;

    #[test]
    fn test_parse() {
        let mut file = std::fs::File::open("/home/jam/dev/arc-browser/metal_szerosuit_001_emi.nutexb").unwrap();

        let x: NutexbFile = file.read_le().unwrap();

        dbg!(&x.footer);

        x.as_rgba8().save("out.png").unwrap();
    }
}
