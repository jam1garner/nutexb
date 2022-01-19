pub fn swizzle_data(
    width: usize,
    height: usize,
    block_width: usize,
    block_height: usize,
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
        1,
        &data,
        block_width,
        block_height,
        1,
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
    block_width: usize,
    block_height: usize,
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
        1,
        data,
        block_width,
        block_height,
        1,
        None,
        bytes_per_pixel,
        mipmap_count,
        array_count,
    )
    .unwrap()
}
