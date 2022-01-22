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
use binrw::{binrw, prelude::*, NullString, ReadOptions, VecArgs};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    num::NonZeroUsize,
};
use surface::deswizzle_data;
use tegra_swizzle::surface::BlockDim;

#[cfg(feature = "ddsfile")]
pub use ddsfile;

#[cfg(feature = "ddsfile")]
mod dds;

#[cfg(feature = "ddsfile")]
pub use dds::create_dds;

mod convert;
pub use convert::*;

#[cfg(feature = "image")]
pub use image;

#[cfg(feature = "image")]
mod rgbaimage;

mod surface;

const FOOTER_SIZE: usize = 112;
const LAYER_MIPMAPS_SIZE: usize = 64;

/// The data stored in a nutexb file like `"def_001_col.nutexb"`.
#[derive(Debug, Clone, BinWrite)]
pub struct NutexbFile {
    /// Combined image data for all array and mipmap levels.
    // Use a custom parser since we don't know the length yet.
    pub data: Vec<u8>,

    /// The size of the mipmaps for each array layer.
    ///
    /// Most nutexb files use swizzled image data,
    /// so these sizes won't add up to the length of [data](struct.NutexbFile.html#structfield.data).
    pub layer_mipmaps: Vec<LayerMipmaps>,

    /// Information about the image stored in [data](#structfield.data).
    pub footer: NutexbFooter,
}

impl BinRead for NutexbFile {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        // We need the footer to know the size of the layer mipmaps.
        reader.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;
        let footer: NutexbFooter = reader.read_le()?;

        // We need the layer mipmaps to know the size of the data section.
        reader.seek(SeekFrom::Current(
            -(FOOTER_SIZE as i64 + LAYER_MIPMAPS_SIZE as i64 * footer.layer_count as i64),
        ))?;

        // The image data takes up the remaining space.
        let data_size = reader.stream_position()?;

        let layer_mipmaps: Vec<LayerMipmaps> = reader.read_le_args(VecArgs {
            count: footer.layer_count as usize,
            inner: (footer.mipmap_count,),
        })?;

        reader.seek(SeekFrom::Start(0))?;

        let mut data = vec![0u8; data_size as usize];
        reader.read_exact(&mut data)?;

        Ok(Self {
            data,
            layer_mipmaps,
            footer,
        })
    }
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

    pub fn deswizzled_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        deswizzle_data(
            self.footer.width as usize,
            self.footer.height as usize,
            self.footer.depth as usize,
            self.footer.image_format.block_dim(),
            self.footer.image_format.bytes_per_pixel() as usize,
            &self.data,
            self.footer.mipmap_count as usize,
            self.footer.layer_count as usize,
        )
        .map_err(Into::into)
    }
}

/// Information about the image data.
#[binrw]
#[derive(Debug, Clone, PartialEq)]
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
    pub unk2: u32, // TODO: Some kind of flags?
    /// The number of mipmaps in [data](struct.NutexbFile.html#structfield.data) or 1 for no mipmapping.
    pub mipmap_count: u32,
    pub alignment: u32, // TODO: Fix this field name
    /// The number of texture layers in [data](struct.NutexbFile.html#structfield.data). This is 6 for cubemaps and 1 otherwise.
    pub layer_count: u32,
    /// The size in bytes of [data](struct.NutexbFile.html#structfield.data).
    pub data_size: u32,
    #[brw(magic = b" XET")]
    pub version: (u16, u16),
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(mipmap_count: u32))]
pub struct LayerMipmaps {
    /// The size in bytes of the deswizzled data for each mipmap.
    #[brw(pad_size_to = 0x40)]
    #[br(count = mipmap_count)]
    pub mipmap_sizes: Vec<u32>,
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
// ex: num channels, format, type (srgb, unorm, etc)?
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

    pub(crate) fn block_dim(&self) -> BlockDim {
        BlockDim {
            width: NonZeroUsize::new(self.block_width() as usize).unwrap(),
            height: NonZeroUsize::new(self.block_height() as usize).unwrap(),
            depth: NonZeroUsize::new(self.block_depth() as usize).unwrap(),
        }
    }
}
