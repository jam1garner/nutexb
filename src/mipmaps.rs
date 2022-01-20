use std::num::NonZeroUsize;

use tegra_swizzle::surface::BlockDim;

pub fn swizzle_data(
    width: usize,
    height: usize,
    depth: usize,
    block_width: usize,
    block_height: usize,
    block_depth: usize,
    bytes_per_pixel: usize,
    data: &[u8],
    mipmap_count: usize,
    array_count: usize,
) -> Vec<u8> {
    // Combine all the mipmaps and arrays into one contiguous region.
    // TODO: 3D support.
    // TODO: Error handling?
    tegra_swizzle::surface::swizzle_surface(
        width,
        height,
        depth,
        data,
        BlockDim {
            width: NonZeroUsize::new(block_width).unwrap(),
            height: NonZeroUsize::new(block_height).unwrap(),
            depth: NonZeroUsize::new(block_depth).unwrap(),
        },
        None,
        bytes_per_pixel,
        mipmap_count,
        array_count,
    )
    .unwrap()
}

// TODO: Avoid duplicated code with the version for separate mipmaps.
pub fn deswizzle_data(
    width: usize,
    height: usize,
    depth: usize,
    block_width: usize,
    block_height: usize,
    block_depth: usize,
    bytes_per_pixel: usize,
    data: &[u8],
    mipmap_count: usize,
    array_count: usize,
) -> Vec<u8> {
    // TODO: 3D support.
    // TODO: Error handling?
    tegra_swizzle::surface::deswizzle_surface(
        width,
        height,
        depth,
        data,
        BlockDim {
            width: NonZeroUsize::new(block_width).unwrap(),
            height: NonZeroUsize::new(block_height).unwrap(),
            depth: NonZeroUsize::new(block_depth).unwrap(),
        },
        None,
        bytes_per_pixel,
        mipmap_count,
        array_count,
    )
    .unwrap()
}
