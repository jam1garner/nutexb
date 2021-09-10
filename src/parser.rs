use crate::NutexbFormat;
use binread::helpers::read_bytes;
use binread::io::SeekFrom;
use binread::prelude::*;
use binread::BinResult;
use binread::NullString;
use std::io::{Read, Seek};

#[derive(BinRead, Debug, Clone)]
pub struct NutexbFile {
    #[br(seek_before = SeekFrom::End(-112))]
    pub footer: NutexbFooter,

    // Specify the parse function to avoid reading bytes individually.
    #[br(seek_before = SeekFrom::Start(0), parse_with = read_bytes, count = footer.size)]
    pub data: Vec<u8>,
}

#[derive(BinRead, Debug, Clone)]
#[br(magic = b" XNT")]
pub struct NutexbFooter {
    #[br(map = NullString::into_string)]
    pub string: String,

    #[br(seek_before = SeekFrom::End(-44))]
    pub width: u32,
    pub height: u32,
    pub depth: u32,

    pub image_format: NutexbFormat,

    #[br(pad_after = 0x2)]
    pub unk: u8, // 4?

    pub unk2: u32,
    pub mip_count: u32,
    pub alignment: u32,
    pub array_count: u32,
    pub size: u32,

    #[br(magic = b" XET")]
    pub version: (u16, u16),

    #[br(seek_before = SeekFrom::End(-176), count = mip_count)]
    pub mip_sizes: Vec<u32>,
}

/// Reads the nutexb from the specified `path`. The entire file is buffered to improve performance.
pub fn read_nutexb<P: AsRef<std::path::Path>>(path: P) -> Result<NutexbFile, Box<dyn std::error::Error>> {
    let mut file = std::io::Cursor::new(std::fs::read(path)?);
    let nutexb = file.read_le::<NutexbFile>()?;
    Ok(nutexb)
}

impl NutexbFile {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le::<NutexbFile>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use binread::BinReaderExt;

    #[test]
    fn test_parse() {
        let mut file = std::fs::File::open("/home/jam/Downloads/alp_ike_002_col.nutexb").unwrap();

        let x: NutexbFile = file.read_le().unwrap();

        dbg!(x.footer);
    }
}
