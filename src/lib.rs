//! # nutexb
//! Nutexb is an image texture format used in Super Smash Bros Ultimate and some other games.
//! The extension ".nutexb" may stand for "Namco Universal Texture Binary".
//!
//! Image data is stored in a contiguous region of memory with metadata stored in the [NutexbFooter].
//! The supported image formats in [NutexbFormat] use standard compressed and uncompressed formats used for DDS files.
//! The arrays and mipmaps for the image data are stored in a memory layout optimized for the Tegra X1 in a process known as swizzling.
//! This library provides tools for reading and writing nutexb files as well as working with the swizzled image data.
//!
//! ## Reading
//!
//! ## Writing
//! The easiest way to create a [NutexbFile] is by implementing the [ToNutexb] trait and calling [create_nutexb].
//! This trait is already implemented for [ddsfile::Dds] and [image::DynamicImage].
/*!
```rust no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use nutexb::create_nutexb;

let image = image::open("col_001.png")?;
let nutexb = create_nutexb(&image, "col_001")?;

let mut writer = std::io::Cursor::new(Vec::new());
nutexb.write(&mut writer)?;
# Ok(()) }
```
 */
use binrw::{prelude::*, NullString, ReadOptions};
use mipmaps::{deswizzle_data_to_mipmaps, swizzle_mipmaps_to_data, deswizzle_data};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

// TODO: Make dds support optional.
pub use ddsfile;
mod dds;

pub use dds::create_dds;

// TODO: make image support optional.
pub use image;
mod rgbaimage;

mod mipmaps;

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

impl NutexbFile {
    /// Reads the [NutexbFile] from the specified `reader`.
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le::<NutexbFile>()
    }

    /// Reads the [NutexbFile] from the specified `path`.
    /// The entire file is buffered to improve performance.
    pub fn read_from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<NutexbFile, Box<dyn std::error::Error>> {
        let mut file = Cursor::new(std::fs::read(path)?);
        let nutexb = file.read_le::<NutexbFile>()?;
        Ok(nutexb)
    }

    /// Writes the [NutexbFile] to the specified `writer`.
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        self.write_to(writer).map_err(Into::into)
    }

    pub fn deswizzled_mipmaps(&self) -> Vec<Vec<u8>> {
        deswizzle_data_to_mipmaps(
            self.footer.width as usize,
            self.footer.height as usize,
            self.footer.image_format.block_width() as usize,
            self.footer.image_format.block_height() as usize,
            self.footer.image_format.bytes_per_pixel() as usize,
            self.footer.mip_count as usize,
            &self.data,
        )
    }

    pub fn deswizzled_data(&self) -> Vec<u8> {
        deswizzle_data(
            self.footer.width as usize,
            self.footer.height as usize,
            self.footer.image_format.block_width() as usize,
            self.footer.image_format.block_height() as usize,
            self.footer.image_format.bytes_per_pixel() as usize,
            self.footer.mip_count as usize,
            &self.data,
        )
    }
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
    # use nutexb::NutexbFormat;
    assert_eq!(1, NutexbFormat::R8Unorm.bytes_per_pixel());
    assert_eq!(4, NutexbFormat::R8G8B8A8Unorm.bytes_per_pixel());
    assert_eq!(8, NutexbFormat::BC1Unorm.bytes_per_pixel());
    assert_eq!(16, NutexbFormat::BC7Srgb.bytes_per_pixel());
    assert_eq!(16, NutexbFormat::R32G32B32A32Float.bytes_per_pixel());
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

    /// The width in pixels for a compressed block or `1` for uncompressed formats.
    ///
    /// # Examples
    /**
    ```rust
    # use nutexb::NutexbFormat;
    assert_eq!(1, NutexbFormat::R8Unorm.block_width());
    assert_eq!(1, NutexbFormat::R8G8B8A8Unorm.block_width());
    assert_eq!(4, NutexbFormat::BC1Unorm.block_width());
    assert_eq!(4, NutexbFormat::BC7Srgb.block_width());
    ```
    */
    pub fn block_width(&self) -> u32 {
        match &self {
            NutexbFormat::R8Unorm
            | NutexbFormat::R8G8B8A8Unorm
            | NutexbFormat::R8G8B8A8Srgb
            | NutexbFormat::R32G32B32A32Float
            | NutexbFormat::B8G8R8A8Unorm
            | NutexbFormat::B8G8R8A8Srgb => 1,
            _ => 4,
        }
    }

    /// The height in pixels for a compressed block or `1` for uncompressed formats.
    ///
    /// # Examples
    /**
    ```rust
    # use nutexb::NutexbFormat;
    assert_eq!(1, NutexbFormat::R8Unorm.block_height());
    assert_eq!(1, NutexbFormat::R8G8B8A8Unorm.block_height());
    assert_eq!(4, NutexbFormat::BC1Unorm.block_height());
    assert_eq!(4, NutexbFormat::BC7Srgb.block_height());
    ```
    */
    pub fn block_height(&self) -> u32 {
        // All known nutexb formats use square blocks.
        self.block_width()
    }

    /// The depth in pixels for a compressed block or `1` for uncompressed formats.
    ///
    /// # Examples
    /**
    ```rust
    # use nutexb::NutexbFormat;
    assert_eq!(1, NutexbFormat::R8Unorm.block_depth());
    assert_eq!(1, NutexbFormat::R8G8B8A8Unorm.block_depth());
    assert_eq!(1, NutexbFormat::BC1Unorm.block_depth());
    assert_eq!(1, NutexbFormat::BC7Srgb.block_depth());
    ```
    */
    pub fn block_depth(&self) -> u32 {
        // All known nutexb formats use 2D blocks.
        1
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

// TODO: Do we need these write functions?
/// Creates a [NutexbFile] with the nutexb string set to `name` and writes its data to `writer`.
pub fn write_nutexb<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    create_nutexb(image, name)?.write(writer)
}

/// Creates a [NutexbFile] from `image` with the nutexb string set to `name`.
/// The result of [ToNutexb::mipmaps] is swizzled according to the specified dimensions and format.
pub fn create_nutexb<N: ToNutexb, S: Into<String>>(
    image: &N,
    name: S,
) -> Result<NutexbFile, Box<dyn Error>> {
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
        width as usize,
        height as usize,
        block_width as usize,
        block_height as usize,
        bytes_per_pixel as usize,
        mipmaps,
    );

    let size = data.len() as u32;

    Ok(NutexbFile {
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
    })
}

/// Creates a [NutexbFile] from `image` with the nutexb string set to `name` without any swizzling.
/// Prefer [create_nutexb] for better memory access performance in most cases.
///
/// Textures created with [create_nutexb] use a memory layout optimized for the Tegra X1 with better access performance in the general case.
/// This function exists for the rare case where swizzling the image data is not desired for performance or compatibility reasons.
pub fn create_nutexb_unswizzled<N: ToNutexb, S: Into<String>>(
    image: &N,
    name: S,
) -> Result<NutexbFile, Box<dyn Error>> {
    let width = image.width();
    let height = image.height();
    let depth = image.depth();

    // TODO: Mipmaps
    let data = image.mipmaps()?[0].clone();

    let size = data.len() as u32;
    Ok(NutexbFile {
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
    })
}

/// Writes a nutexb without any swizzling.
pub fn write_nutexb_unswizzled<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    create_nutexb_unswizzled(image, name)?.write(writer)
}
