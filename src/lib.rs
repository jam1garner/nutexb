pub mod parser;
pub mod tegra_swizzle;
pub mod writer;
use binread::prelude::*;

pub use ddsfile;

#[derive(Debug, Clone, Copy, PartialEq, BinRead)]
#[br(repr(u8))]
pub enum NutexbFormat {
    R8G8B8A8Unorm = 0,
    R8G8B8A8Srgb = 5,
    R32G32B32A32Float = 52,
    B8G8R8A8Unorm = 80,
    B8G8R8A8Srgb = 85,
    BC1Unorm = 128,
    BC1Srgb = 133,
    BC2Unorm = 144,
    BC2Srgb = 149,
    BC3Unorm = 160,
    BC3Srgb = 165,
    BC4Unorm = 176,
    BC4Snorm = 181,
    BC5Unorm = 192,
    BC5Snorm = 197,
    BC6Ufloat = 215,
    BC7Unorm = 224,
    BC7Srgb = 229,
}

impl NutexbFormat {
    /// The number of bytes to store a single pixel.
    /// For block compressed formats like [NutexbFormat::BC7Srgb], this is the size in bytes of a single block.
    /**
    ```rust
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
            NutexbFormat::BC5Unorm => 16,
            NutexbFormat::BC5Snorm => 16,
            NutexbFormat::BC6Ufloat => 16,
            NutexbFormat::BC7Unorm => 16,
            NutexbFormat::BC7Srgb => 16,
        }
    }
}
