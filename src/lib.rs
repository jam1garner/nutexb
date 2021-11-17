pub mod parser;
pub mod writer;
use binread::prelude::*;

pub use ddsfile;

// TODO: It's possible this is some sort of flags.
// num channels, format, type (srgb, unorm, etc)?
// TODO: Add these as methods?
#[derive(Debug, Clone, Copy, PartialEq, BinRead)]
#[br(repr(u32))]
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
            NutexbFormat::R8Unorm => 1
        }
    }
}
