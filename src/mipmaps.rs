use tegra_swizzle::{
    block_height_mip0, deswizzle_block_linear, deswizzled_surface_size, div_round_up,
    mip_block_height, swizzle_block_linear, swizzled_surface_size,
};

pub fn swizzle_mipmaps_to_data(
    width: usize,
    height: usize,
    block_width: usize,
    block_height: usize,
    bytes_per_pixel: usize,
    mipmaps: Vec<Vec<u8>>,
) -> Vec<u8> {
    // Combine all the mipmaps into one contiguous region.
    let mut data = Vec::new();
    let block_height_mip0 = block_height_mip0(div_round_up(height, block_height));

    for (i, mip) in mipmaps.into_iter().enumerate() {
        let mip_width = div_round_up(width >> i, block_width);
        let mip_height = div_round_up(height >> i, block_height);

        // The block height will likely change for each mip level.
        let mip_block_height = mip_block_height(mip_height, block_height_mip0);

        let swizzled_mipmap = swizzle_block_linear(
            mip_width,
            mip_height,
            1,
            &mip,
            mip_block_height,
            bytes_per_pixel,
        )
        .unwrap();

        data.extend_from_slice(&swizzled_mipmap);
    }

    data
}

pub fn deswizzle_data_to_mipmaps(
    width: usize,
    height: usize,
    block_width: usize,
    block_height: usize,
    bytes_per_pixel: usize,
    mipmap_count: usize,
    data: &[u8],
) -> Vec<Vec<u8>> {
    // TODO: 3D support.
    let mut mipmaps = Vec::new();

    let block_height_mip0 = block_height_mip0(div_round_up(height, block_height));

    let mut offset = 0;
    for mip in 0..mipmap_count {
        let mip_width = div_round_up(width >> mip, block_width);
        let mip_height = div_round_up(height >> mip, block_height);

        // The block height will likely change for each mip level.
        let mip_block_height = mip_block_height(mip_height, block_height_mip0);

        let deswizzled_mipmap = deswizzle_block_linear(
            mip_width,
            mip_height,
            1,
            &data[offset..],
            mip_block_height,
            bytes_per_pixel,
        )
        .unwrap();

        mipmaps.push(deswizzled_mipmap);

        offset += swizzled_surface_size(mip_width, mip_height, 1, mip_block_height, 16);
    }

    mipmaps
}
