use crate::{surface::swizzle_data, LayerMipmaps, NutexbFile, NutexbFooter, NutexbFormat};
use binrw::NullString;
use std::{
    cmp::max,
    error::Error,
    io::{Seek, Write},
};
use tegra_swizzle::{div_round_up, surface::BlockDim};

// TODO: It should be possible to make a NutexbFile from anything that is ToNutexb.
// This avoids having to write the data somewhere.

/// A trait for creating a Nutexb from unswizzled image data.
/// Implement this trait for an image type to support writing a nutexb file with [write_nutexb].
pub trait ToNutexb {
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn depth(&self) -> u32;

    /// The raw image data for each layer and mipmap before applying any swizzling.
    fn image_data(&self) -> Result<Vec<u8>, Box<dyn Error>>;

    // TODO: Add an option to generate mipmaps?
    fn mipmap_count(&self) -> u32;

    fn layer_count(&self) -> u32;

    fn image_format(&self) -> Result<NutexbFormat, Box<dyn Error>>;
}

// TODO: Do we need these write functions?
/// Creates a [NutexbFile] with the nutexb string set to `name` and writes its data to `writer`.
pub fn write_nutexb<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    create_nutexb(image, name)?.write(writer)
}

/// Creates a [NutexbFile] from `image` with the nutexb string set to `name`.
/// The result of [ToNutexb::mipmaps] is swizzled according to the specified dimensions and format.
pub fn create_nutexb<N: ToNutexb, S: Into<String>>(
    image: &N,
    name: S,
) -> Result<NutexbFile, Box<dyn Error>> {
    let width = image.width();
    let height = image.height();
    let depth = image.depth();

    let image_format = image.image_format()?;
    let bytes_per_pixel = image_format.bytes_per_pixel();
    let block_dim = image_format.block_dim();

    let image_data = image.image_data()?;

    let mip_count = image.mipmap_count();

    let layer_count = image.layer_count();

    let layer_mipmaps = calculate_layer_mip_sizes(
        width as usize,
        height as usize,
        depth as usize,
        block_dim,
        bytes_per_pixel as usize,
        mip_count as usize,
        layer_count as usize,
    );

    let data = swizzle_data(
        width as usize,
        height as usize,
        depth as usize,
        block_dim,
        bytes_per_pixel as usize,
        &image_data,
        mip_count as usize,
        layer_count as usize,
    )?;

    let size = data.len() as u32;

    let unk2 = unk2(depth, layer_count);

    Ok(NutexbFile {
        data,
        layer_mipmaps,
        footer: NutexbFooter {
            string: NullString::from_string(name.into()),
            width,
            height,
            depth,
            image_format,
            unk2,
            mipmap_count: mip_count,
            alignment: 0x1000,
            layer_count,
            data_size: size,
            version: (1, 2),
        },
    })
}

fn unk2(depth: u32, layer_count: u32) -> u32 {
    // TODO: What does this value do?
    if depth > 1 {
        8
    } else if layer_count > 1 {
        9
    } else {
        4
    }
}

// TODO: Move into tegra_swizzle?
fn calculate_layer_mip_sizes(
    width: usize,
    height: usize,
    depth: usize,
    block_dim: BlockDim,
    bytes_per_pixel: usize,
    mip_count: usize,
    layer_count: usize,
) -> Vec<LayerMipmaps> {
    // Mipmaps are repeated for each layer.
    let layer = LayerMipmaps {
        mipmap_sizes: (0..mip_count)
            .into_iter()
            .map(|mip| {
                // Halve dimensions for each mip level after the base level.
                // The minimum mipmap size depends on the format.
                let mip_width = max(div_round_up(width >> mip, block_dim.width.get()), 1);
                let mip_height = max(div_round_up(height >> mip, block_dim.height.get()), 1);
                let mip_depth = max(div_round_up(depth >> mip, block_dim.depth.get()), 1);

                let mip_size = mip_width * mip_height * mip_depth * bytes_per_pixel;
                max(mip_size, bytes_per_pixel) as u32
            })
            .collect(),
    };
    vec![layer; layer_count]
}

/// Creates a [NutexbFile] from `image` with the nutexb string set to `name` without any swizzling.
/// This assumes no layers or mipmaps for `image`.
/// Prefer [create_nutexb] for better memory access performance in most cases.
///
/// Textures created with [create_nutexb] use a memory layout optimized for the Tegra X1 with better access performance in the general case.
/// This function exists for the rare case where swizzling the image data is not desired for performance or compatibility reasons.
pub fn create_nutexb_unswizzled<N: ToNutexb, S: Into<String>>(
    image: &N,
    name: S,
) -> Result<NutexbFile, Box<dyn Error>> {
    let width = image.width();
    let height = image.height();
    let depth = image.depth();

    // TODO: Mipmaps
    let data = image.image_data()?;

    let image_format = image.image_format()?;
    let bytes_per_pixel = image_format.bytes_per_pixel();
    let block_dim = image_format.block_dim();

    let layer_mipmaps = calculate_layer_mip_sizes(
        width as usize,
        height as usize,
        depth as usize,
        block_dim,
        bytes_per_pixel as usize,
        1,
        1,
    );

    let size = data.len() as u32;
    Ok(NutexbFile {
        data,
        layer_mipmaps,
        footer: NutexbFooter {
            string: NullString::from_string(name.into()),
            width,
            height,
            depth,
            image_format,
            unk2: 2,
            mipmap_count: 1,
            alignment: 0,
            layer_count: 1,
            data_size: size,
            version: (2, 0),
        },
    })
}

/// Writes a nutexb without any swizzling.
pub fn write_nutexb_unswizzled<W: Write + Seek, S: Into<String>, N: ToNutexb>(
    name: S,
    image: &N,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    create_nutexb_unswizzled(image, name)?.write(writer)
}
