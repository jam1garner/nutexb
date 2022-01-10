pub mod parser;
pub mod writer;
use binrw::{prelude::*, NullString, ReadOptions};
use std::io::{Read, Seek, SeekFrom};

// TODO: Make dds optional.
pub use ddsfile;

/// The data stored in a nutexb file like `"def_001_col.nutexb"`.
// TODO: Alignment requirements for the data or file length?
#[derive(BinRead, BinWrite, Debug, Clone)]
pub struct NutexbFile {
    /// Combined image data for all array and mipmap levels.
    // Use a custom parser since we don't know the length yet.
    #[br(parse_with = until_footer)]
    pub data: Vec<u8>,

    /// Information about the image stored in [data](#structfield.data).
    // Add padding on write to fill in mip sizes later.
    // TODO: Does nutexb support more than 16 mips (0x40 bytes)?
    #[br(seek_before = SeekFrom::End(-112))]
    #[bw(pad_before = 0x40)]
    pub footer: NutexbFooter,
}

/// Information about the image data.
#[derive(BinRead, BinWrite, Debug, Clone)]
#[brw(magic = b" XNT")]
pub struct NutexbFooter {
    // TODO: Make this field "name: String"
    // TODO: Names can be at most 63 characters + 1 null byte?
    /// The name of the texture, which usually matches the file name without its extension like `"def_001_col"`.
    #[brw(pad_size_to = 0x40)]
    pub string: NullString,
    /// The width of the texture in pixels.
    pub width: u32,
    /// The height of the texture in pixels.
    pub height: u32,
    /// The depth of the texture in pixels or 1 for 2D textures.
    pub depth: u32,
    /// The format of [data](struct.NutexbFile.html#structfield.data).
    pub image_format: NutexbFormat,
    pub unk2: u32,
    /// The number of mipmaps in [data](struct.NutexbFile.html#structfield.data) or 1 for no mipmapping.
    pub mip_count: u32,
    pub alignment: u32, // TODO: Fix this field name
    /// The number of texture arrays in [data](struct.NutexbFile.html#structfield.data). This is 6 for cubemaps and 1 otherwise.
    pub array_count: u32,
    /// The size in bytes of [data](struct.NutexbFile.html#structfield.data).
    pub data_size: u32,
    #[brw(magic = b" XET")]
    pub version: (u16, u16),

    /// The size in bytes of the deswizzled data for each mipmap.
    /// 
    /// Most nutexb files use swizzled image data, 
    /// so these sizes won't add up to the length of [data](struct.NutexbFile.html#structfield.data).
    #[brw(seek_before = SeekFrom::End(-176))]
    #[br(count = mip_count)]
    pub mip_sizes: Vec<u32>,
}

/// Supported image data formats.
/// 
/// These formats have a corresponding format in modern versions of graphics APIs like OpenGL, Vulkan, etc.
/// Most of the compressed formats are supported by [Dds](ddsfile::Dds).
/// 
/// In some contexts, "Unorm" is called "linear" or expanded to "unsigned normalized".
/// "U" and "S" prefixes refer to "unsigned" and "signed" data, respectively.
/// 
/// Variants with "Srgb" store identical data as "Unorm" variants but signal to the graphics API to 
/// convert from sRGB to linear gamma when accessing texture data. 
// TODO: It's possible this is some sort of flags.
// num channels, format, type (srgb, unorm, etc)?
// TODO: Add these as methods?
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
#[brw(repr(u32))]
pub enum NutexbFormat {
    R8Unorm = 0x0100,
    R8G8B8A8Unorm = 0x0400,
    R8G8B8A8Srgb = 0x0405,
    R32G32B32A32Float = 0x0434,
    B8G8R8A8Unorm = 0x0450,
    B8G8R8A8Srgb = 0x0455,
    BC1Unorm = 0x0480,
    BC1Srgb = 0x0485,
    BC2Unorm = 0x0490,
    BC2Srgb = 0x0495,
    BC3Unorm = 0x04a0,
    BC3Srgb = 0x04a5,
    BC4Unorm = 0x0180,
    BC4Snorm = 0x0185,
    BC5Unorm = 0x0280,
    BC5Snorm = 0x0285,
    BC6Ufloat = 0x04d7,
    BC6Sfloat = 0x04d8,
    BC7Unorm = 0x04e0,
    BC7Srgb = 0x04e5,
}

impl NutexbFormat {
    /// The number of bytes to store a single pixel.
    /// For block compressed formats like [NutexbFormat::BC7Srgb], this is the size in bytes of a single block.
    /**
    ```rust
    assert_eq!(1, NutexbFormat::R8Unorm.size_in_bytes());
    assert_eq!(4, NutexbFormat::R8G8B8A8Unorm.size_in_bytes());
    assert_eq!(8, NutexbFormat::BC1Unorm.size_in_bytes());
    assert_eq!(16, NutexbFormat::BC7Unorm.size_in_bytes());
    assert_eq!(16, NutexbFormat::R32G32B32A32Float.size_in_bytes());
    ```
    */
    pub fn bytes_per_pixel(&self) -> u32 {
        match &self {
            NutexbFormat::R8G8B8A8Unorm
            | NutexbFormat::R8G8B8A8Srgb
            | NutexbFormat::B8G8R8A8Unorm
            | NutexbFormat::B8G8R8A8Srgb => 4,
            NutexbFormat::R32G32B32A32Float => 16,
            NutexbFormat::BC1Unorm | NutexbFormat::BC1Srgb => 8,
            NutexbFormat::BC2Unorm | NutexbFormat::BC2Srgb => 16,
            NutexbFormat::BC3Unorm | NutexbFormat::BC3Srgb => 16,
            NutexbFormat::BC4Unorm | NutexbFormat::BC4Snorm => 8,
            NutexbFormat::BC5Unorm | NutexbFormat::BC5Snorm => 16,
            NutexbFormat::BC6Ufloat | NutexbFormat::BC6Sfloat => 16,
            NutexbFormat::BC7Unorm | NutexbFormat::BC7Srgb => 16,
            NutexbFormat::R8Unorm => 1,
        }
    }
}

fn until_footer<R: Read + Seek>(reader: &mut R, _: &ReadOptions, _: ()) -> BinResult<Vec<u8>> {
    // Assume the footer has a fixed size.
    // Smash Ultimate doesn't require the footer to correctly report the image size.
    let footer_start = reader.seek(SeekFrom::End(-176))?;
    reader.seek(SeekFrom::Start(0))?;

    let mut data = vec![0u8; footer_start as usize];
    reader.read_exact(&mut data)?;
    Ok(data)
}
