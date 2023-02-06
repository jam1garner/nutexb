use crate::{LayerMipmaps, NutexbFile, NutexbFooter, NutexbFormat};
use binrw::NullString;
use std::cmp::max;
use tegra_swizzle::{div_round_up, surface::BlockDim};

/// A surface describing a contiguous chunk of image data for the array layers and mipmaps used to create a [NutexbFile].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Surface<T> {
    /// The width of the image in pixels.
    pub width: u32,

    /// The height of the image in pixels.
    pub height: u32,

    /// The depth of the image in pixels. This should be `1` for 2D textures.
    pub depth: u32,

    /// The raw image data for each layer and mipmap before applying any swizzling.
    /// Data should be arranged in row-major order with no padding between arrays and mipmaps.
    /// See [tegra_swizzle::surface] for details.
    ///
    /// Functions accept owned data like `Vec<u8>` or borrowed data like `&[u8]`.
    pub image_data: T,

    // TODO: Add an option to generate mipmaps?
    /// The number of mipmaps or `1` to indicate no mipmaps.
    pub mipmap_count: u32,

    /// The number of array layers or `1` to indicate no layers.
    /// This should be `6` for cube maps.
    pub layer_count: u32,

    /// The format of the data stored in [image_data](#structfield.image_data).
    pub image_format: NutexbFormat,
}

pub fn create_nutexb<T: AsRef<[u8]>, S: Into<String>>(
    image: Surface<T>,
    name: S,
) -> Result<NutexbFile, tegra_swizzle::SwizzleError> {
    let width = image.width;
    let height = image.height;
    let depth = image.depth;

    let image_format = image.image_format;
    let bytes_per_pixel = image_format.bytes_per_pixel();
    let block_dim = image_format.block_dim();

    let mip_count = image.mipmap_count;

    let layer_count = image.layer_count;

    let layer_mipmaps = calculate_layer_mip_sizes(
        width as usize,
        height as usize,
        depth as usize,
        block_dim,
        bytes_per_pixel as usize,
        mip_count as usize,
        layer_count as usize,
    );

    let data = tegra_swizzle::surface::swizzle_surface(
        width as usize,
        height as usize,
        depth as usize,
        image.image_data.as_ref(),
        block_dim,
        None,
        bytes_per_pixel as usize,
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
            unk3: 0x1000,
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

                (mip_width * mip_height * mip_depth * bytes_per_pixel) as u32
            })
            .collect(),
    };
    vec![layer; layer_count]
}

pub fn create_nutexb_unswizzled<T: AsRef<[u8]>, S: Into<String>>(
    surface: &Surface<T>,
    name: S,
) -> NutexbFile {
    let width = surface.width;
    let height = surface.height;
    let depth = surface.depth;

    // TODO: Mipmaps and array layers?
    let data = surface.image_data.as_ref().to_vec();

    let image_format = surface.image_format;
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
    NutexbFile {
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
            unk3: 0, // TODO: toggles swizzling?
            layer_count: 1,
            data_size: size,
            version: (2, 0),
        },
    }
}
