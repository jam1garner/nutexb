use tegra_swizzle::{surface::BlockDim, SwizzleError};

pub fn swizzle_data(
    width: usize,
    height: usize,
    depth: usize,
    block_dim: BlockDim,
    bytes_per_pixel: usize,
    data: &[u8],
    mipmap_count: usize,
    array_count: usize,
) -> Result<Vec<u8>, SwizzleError> {
    // Combine all the mipmaps and arrays into one contiguous region.
    tegra_swizzle::surface::swizzle_surface(
        width,
        height,
        depth,
        data,
        block_dim,
        None,
        bytes_per_pixel,
        mipmap_count,
        array_count,
    )
}

pub fn deswizzle_data(
    width: usize,
    height: usize,
    depth: usize,
    block_dim: BlockDim,
    bytes_per_pixel: usize,
    data: &[u8],
    mipmap_count: usize,
    array_count: usize,
) -> Result<Vec<u8>, SwizzleError> {
    tegra_swizzle::surface::deswizzle_surface(
        width,
        height,
        depth,
        data,
        block_dim,
        None,
        bytes_per_pixel,
        mipmap_count,
        array_count,
    )
}
