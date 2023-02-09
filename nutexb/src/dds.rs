use std::{error::Error, fmt::Display};

use ddsfile::{
    AlphaMode, Caps2, D3D10ResourceDimension, D3DFormat, Dds, DxgiFormat, FourCC, NewDxgiParams,
};

use crate::{NutexbFile, NutexbFormat, Surface};

/// Errors while creating a nutexb file from a DDS file.
#[derive(Debug)]
pub enum ReadDdsError {
    /// The DDS format is not a recognized or supported nutexb format.
    UnrecognizedFormat,
    /// The DDS data could not be swizzled.
    /// This usually means the DDS header does not accurately describe the image data.
    SwizzleError(tegra_swizzle::SwizzleError),
}

impl Display for ReadDdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadDdsError::UnrecognizedFormat => write!(f, "unrecognized DDS format"),
            ReadDdsError::SwizzleError(e) => write!(f, "failed to swizzle surface: {e}"),
        }
    }
}

impl From<tegra_swizzle::SwizzleError> for ReadDdsError {
    fn from(value: tegra_swizzle::SwizzleError) -> Self {
        Self::SwizzleError(value)
    }
}

impl Error for ReadDdsError {}

pub fn create_surface(dds: &Dds) -> Result<Surface<&[u8]>, ReadDdsError> {
    Ok(Surface {
        width: dds.get_width(),
        height: dds.get_height(),
        depth: dds.get_depth(),
        image_data: &dds.data,
        mipmap_count: dds.get_num_mipmap_levels(),
        layer_count: layer_count(dds),
        image_format: dds_image_format(dds).ok_or(ReadDdsError::UnrecognizedFormat)?,
    })
}

fn layer_count(dds: &Dds) -> u32 {
    // Array layers for DDS are calculated differently for cube maps.
    if matches!(&dds.header10, Some(header10) if header10.misc_flag == ddsfile::MiscFlag::TEXTURECUBE)
    {
        dds.get_num_array_layers() * 6
    } else {
        dds.get_num_array_layers()
    }
}

fn dds_image_format(dds: &Dds) -> Option<NutexbFormat> {
    // The format can be DXGI, D3D, or specified in the FOURCC.
    let dxgi = dds.get_dxgi_format();
    let d3d = dds.get_d3d_format();
    let fourcc = dds.header.spf.fourcc.as_ref();

    dxgi.and_then(image_format_from_dxgi)
        .or_else(|| d3d.and_then(image_format_from_d3d))
        .or_else(|| fourcc.and_then(image_format_from_fourcc))
}

fn image_format_from_dxgi(format: DxgiFormat) -> Option<NutexbFormat> {
    match format {
        DxgiFormat::R8_UNorm => Some(NutexbFormat::R8Unorm),
        DxgiFormat::R8G8B8A8_UNorm => Some(NutexbFormat::R8G8B8A8Unorm),
        DxgiFormat::R8G8B8A8_UNorm_sRGB => Some(NutexbFormat::R8G8B8A8Srgb),
        DxgiFormat::R32G32B32A32_Float => Some(NutexbFormat::R32G32B32A32Float),
        DxgiFormat::B8G8R8A8_UNorm => Some(NutexbFormat::B8G8R8A8Unorm),
        DxgiFormat::B8G8R8A8_UNorm_sRGB => Some(NutexbFormat::B8G8R8A8Srgb),
        DxgiFormat::BC1_UNorm => Some(NutexbFormat::BC1Unorm),
        DxgiFormat::BC1_UNorm_sRGB => Some(NutexbFormat::BC1Srgb),
        DxgiFormat::BC2_UNorm => Some(NutexbFormat::BC2Unorm),
        DxgiFormat::BC2_UNorm_sRGB => Some(NutexbFormat::BC2Srgb),
        DxgiFormat::BC3_UNorm => Some(NutexbFormat::BC3Unorm),
        DxgiFormat::BC3_UNorm_sRGB => Some(NutexbFormat::BC3Srgb),
        DxgiFormat::BC4_UNorm => Some(NutexbFormat::BC4Unorm),
        DxgiFormat::BC4_SNorm => Some(NutexbFormat::BC4Snorm),
        DxgiFormat::BC5_UNorm => Some(NutexbFormat::BC5Unorm),
        DxgiFormat::BC5_SNorm => Some(NutexbFormat::BC5Snorm),
        DxgiFormat::BC6H_SF16 => Some(NutexbFormat::BC6Sfloat),
        DxgiFormat::BC6H_UF16 => Some(NutexbFormat::BC6Ufloat),
        DxgiFormat::BC7_UNorm => Some(NutexbFormat::BC7Unorm),
        DxgiFormat::BC7_UNorm_sRGB => Some(NutexbFormat::BC7Srgb),
        _ => None,
    }
}

fn image_format_from_d3d(format: D3DFormat) -> Option<NutexbFormat> {
    match format {
        D3DFormat::DXT1 => Some(NutexbFormat::BC1Unorm),
        D3DFormat::DXT2 => Some(NutexbFormat::BC2Unorm),
        D3DFormat::DXT3 => Some(NutexbFormat::BC2Unorm),
        D3DFormat::DXT4 => Some(NutexbFormat::BC3Unorm),
        D3DFormat::DXT5 => Some(NutexbFormat::BC3Unorm),
        _ => None,
    }
}

const BC5U: u32 = u32::from_le_bytes(*b"BC5U");
const ATI2: u32 = u32::from_le_bytes(*b"ATI2");

fn image_format_from_fourcc(fourcc: &FourCC) -> Option<NutexbFormat> {
    match fourcc.0 {
        FourCC::DXT1 => Some(NutexbFormat::BC1Unorm),
        FourCC::DXT2 => Some(NutexbFormat::BC2Unorm),
        FourCC::DXT3 => Some(NutexbFormat::BC2Unorm),
        FourCC::DXT4 => Some(NutexbFormat::BC3Unorm),
        FourCC::DXT5 => Some(NutexbFormat::BC3Unorm),
        FourCC::BC4_UNORM => Some(NutexbFormat::BC4Unorm),
        FourCC::BC4_SNORM => Some(NutexbFormat::BC4Snorm),
        ATI2 | BC5U => Some(NutexbFormat::BC5Unorm),
        FourCC::BC5_SNORM => Some(NutexbFormat::BC5Snorm),
        _ => None,
    }
}

impl From<NutexbFormat> for DxgiFormat {
    fn from(value: NutexbFormat) -> Self {
        match value {
            NutexbFormat::BC1Unorm => Self::BC1_UNorm,
            NutexbFormat::BC1Srgb => Self::BC1_UNorm_sRGB,
            NutexbFormat::BC2Unorm => Self::BC2_UNorm,
            NutexbFormat::BC2Srgb => Self::BC2_UNorm_sRGB,
            NutexbFormat::BC3Unorm => Self::BC3_UNorm,
            NutexbFormat::BC3Srgb => Self::BC3_UNorm_sRGB,
            NutexbFormat::BC4Unorm => Self::BC4_UNorm,
            NutexbFormat::BC4Snorm => Self::BC4_SNorm,
            NutexbFormat::BC5Unorm => Self::BC5_UNorm,
            NutexbFormat::BC5Snorm => Self::BC5_SNorm,
            NutexbFormat::BC6Ufloat => Self::BC6H_UF16,
            NutexbFormat::BC6Sfloat => Self::BC6H_SF16,
            NutexbFormat::BC7Unorm => Self::BC7_UNorm,
            NutexbFormat::BC7Srgb => Self::BC7_UNorm_sRGB,
            NutexbFormat::R8Unorm => Self::R8_UNorm,
            NutexbFormat::R8G8B8A8Unorm => Self::R8G8B8A8_UNorm,
            NutexbFormat::R8G8B8A8Srgb => Self::R8G8B8A8_UNorm_sRGB,
            NutexbFormat::R32G32B32A32Float => Self::R32G32B32A32_Float,
            NutexbFormat::B8G8R8A8Unorm => Self::B8G8R8A8_UNorm,
            NutexbFormat::B8G8R8A8Srgb => Self::B8G8R8A8_UNorm_sRGB,
        }
    }
}

pub fn create_dds(nutexb: &NutexbFile) -> Result<Dds, tegra_swizzle::SwizzleError> {
    let some_if_above_one = |x| if x > 0 { Some(x) } else { None };

    // TODO: Avoid unwrap.
    let mut dds = Dds::new_dxgi(NewDxgiParams {
        height: nutexb.footer.height,
        width: nutexb.footer.width,
        depth: some_if_above_one(nutexb.footer.depth),
        format: nutexb.footer.image_format.into(),
        mipmap_levels: some_if_above_one(nutexb.footer.mipmap_count),
        array_layers: some_if_above_one(nutexb.footer.layer_count),
        caps2: if nutexb.footer.depth > 1 {
            Some(Caps2::VOLUME)
        } else {
            None
        },
        is_cubemap: nutexb.footer.layer_count == 6,
        resource_dimension: if nutexb.footer.depth > 1 {
            D3D10ResourceDimension::Texture3D
        } else {
            D3D10ResourceDimension::Texture2D
        },
        alpha_mode: AlphaMode::Unknown, // TODO: Alpha mode?
    })
    .unwrap();

    // DDS stores mipmaps in a contiguous region of memory.
    dds.data = nutexb.deswizzled_data()?;

    Ok(dds)
}
