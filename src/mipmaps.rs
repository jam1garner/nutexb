use tegra_swizzle::{block_height_mip0, div_round_up, mip_block_height, swizzle_block_linear};

pub fn swizzle_mipmaps_to_data(
    height: usize,
    block_width: usize,
    block_height: usize,
    mipmaps: Vec<Vec<u8>>,
    width: usize,
    bytes_per_pixel: usize,
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
