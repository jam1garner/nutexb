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