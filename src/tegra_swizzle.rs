pub fn deswizzle(
    width: u32,
    height: u32,
    _depth: u32,
    blk_width: u32,
    blk_height: u32,
    _blk_depth: u32,
    round_pitch: bool,
    bpp: u32,
    tile_mode: u32,
    size_range: i32,
    data: &[u8],
) -> Vec<u8> {
    _swizzle(
        width,
        height,
        _depth,
        blk_width,
        blk_height,
        _blk_depth,
        round_pitch,
        bpp,
        tile_mode,
        size_range,
        data,
        false,
    )
}

pub fn swizzle(
    width: u32,
    height: u32,
    _depth: u32,
    blk_width: u32,
    blk_height: u32,
    _blk_depth: u32,
    round_pitch: bool,
    bpp: u32,
    tile_mode: u32,
    size_range: i32,
    data: &[u8],
) -> Vec<u8> {
    _swizzle(
        width,
        height,
        _depth,
        blk_width,
        blk_height,
        _blk_depth,
        round_pitch,
        bpp,
        tile_mode,
        size_range,
        data,
        true,
    )
}

// Ported from https://github.com/KillzXGaming/Switch-Toolbox/blob/f7d674fe1896decf5234329c01ca2c868e88d96f/Switch_Toolbox_Library/Texture%20Decoding/Switch/TegraX1Swizzle.cs
fn _swizzle(
    width: u32,
    height: u32,
    _depth: u32,
    blk_width: u32,
    blk_height: u32,
    _blk_depth: u32,
    round_pitch: bool,
    bpp: u32,
    tile_mode: u32,
    block_height_log_2: i32,
    data: &[u8],
    to_swizzle: bool,
) -> Vec<u8> {
    let block_height = 1 << block_height_log_2;

    //Console.WriteLine($"Swizzle {width} {height} {blk_width} {blk_height} {round_pitch} {bpp} {tile_mode} {block_height_log_2} {data.Length} {to_swizzle}");

    let width = div_round_up(width, blk_width);
    let height = div_round_up(height, blk_height);

    let pitch;
    let surf_size;
    if tile_mode == 1 {
        if round_pitch {
            pitch = round_up(width * bpp, 32);
        } else {
            pitch = width * bpp;
        }

        surf_size = pitch * height;
    } else {
        pitch = round_up(width * bpp, 64);
        surf_size = pitch * round_up(height, block_height * 8);
    }

    let mut result = vec![0u8; surf_size as usize];

    for y in 0..height {
        for x in 0..width {
            let pos = if tile_mode == 1 {
                y * pitch + x * bpp
            } else {
                get_addr_block_linear(x, y, width, bpp, 0, block_height)
            } as usize;

            let pos_ = ((y * width + x) * bpp) as usize;
            let bpp = bpp as usize;

            if pos + bpp <= surf_size as usize {
                if to_swizzle {
                    (&mut result[pos..pos + bpp]).copy_from_slice(&data[pos_..pos_ + bpp]);
                } else {
                    (&mut result[pos_..pos_ + bpp]).copy_from_slice(&data[pos..pos + bpp]);
                }
            }
        }
    }

    result
}

fn div_round_up(n: u32, d: u32) -> u32 {
    (n + d - 1) / d
}

fn round_up(x: u32, y: u32) -> u32 {
    ((x - 1) | (y - 1)) + 1
}

fn get_addr_block_linear(
    x: u32,
    y: u32,
    width: u32,
    bytes_per_pixel: u32,
    base_address: u32,
    block_height: u32,
) -> u32 {
    /*
    From Tega X1 TRM
                     */
    let image_width_in_gobs = div_round_up(width * bytes_per_pixel, 64);

    let gob_address = base_address
        + (y / (8 * block_height)) * 512 * block_height * image_width_in_gobs
        + (x * bytes_per_pixel / 64) * 512 * block_height
        + (y % (8 * block_height) / 8) * 512;

    let x = x * bytes_per_pixel;

    gob_address
        + ((x % 64) / 32) * 256
        + ((y % 8) / 2) * 64
        + ((x % 32) / 16) * 32
        + (y % 2) * 16
        + (x % 16)
}
