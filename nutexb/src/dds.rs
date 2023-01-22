use std::{
    convert::{TryFrom, TryInto},
    error::Error,
};

use ddsfile::{
    AlphaMode, Caps2, D3D10ResourceDimension, D3DFormat, Dds, DxgiFormat, FourCC, NewDxgiParams,
};

use crate::{NutexbFile, NutexbFormat, ToNutexb};

impl ToNutexb for ddsfile::Dds {
    fn width(&self) -> u32 {
        self.get_width()
    }

    fn height(&self) -> u32 {
        self.get_height()
    }

    fn depth(&self) -> u32 {
        self.get_depth()
    }

    fn image_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.data.clone())
    }

    fn mipmap_count(&self) -> u32 {
        self.get_num_mipmap_levels()
    }

    fn layer_count(&self) -> u32 {
        // Array layers for DDS are calculated differently for cube maps.
        if matches!(&self.header10, Some(header10) if header10.misc_flag == ddsfile::MiscFlag::TEXTURECUBE)
        {
            self.get_num_array_layers() * 6
        } else {
            self.get_num_array_layers()
        }
    }

    fn image_format(&self) -> Result<NutexbFormat, Box<dyn Error>> {
        // The format can be DXGI, D3D, or specified in the FOURCC.
        // Checking FOURCC is necessary for compatibility with some applications.
        self.get_dxgi_format()
            .ok_or_else(|| "Missing DXGI format.".to_owned())
            .and_then(|dxgi| dxgi.try_into())
            .or_else(|_| {
                self.get_d3d_format()
                    .ok_or_else(|| "Missing D3D format.".to_owned())
                    .and_then(|d3d| d3d.try_into())
            })
            .or_else(|_| {
                self.header
                    .spf
                    .fourcc
                    .as_ref()
                    .ok_or_else(|| "Missing FOURCC.".to_owned())
                    .and_then(|fourcc| fourcc.clone().try_into())
            })
            .map_err(Into::into)
    }
}

impl TryFrom<DxgiFormat> for NutexbFormat {
    type Error = String;

    fn try_from(value: DxgiFormat) -> Result<Self, Self::Error> {
        // DXGI DDS supports all the known nutexb formats.
        match value {
            DxgiFormat::R8_UNorm => Ok(NutexbFormat::R8Unorm),
            DxgiFormat::R8G8B8A8_UNorm => Ok(NutexbFormat::R8G8B8A8Unorm),
            DxgiFormat::R8G8B8A8_UNorm_sRGB => Ok(NutexbFormat::R8G8B8A8Srgb),
            DxgiFormat::R32G32B32A32_Float => Ok(NutexbFormat::R32G32B32A32Float),
            DxgiFormat::B8G8R8A8_UNorm => Ok(NutexbFormat::B8G8R8A8Unorm),
            DxgiFormat::B8G8R8A8_UNorm_sRGB => Ok(NutexbFormat::B8G8R8A8Srgb),
            DxgiFormat::BC1_UNorm => Ok(NutexbFormat::BC1Unorm),
            DxgiFormat::BC1_UNorm_sRGB => Ok(NutexbFormat::BC1Srgb),
            DxgiFormat::BC2_UNorm => Ok(NutexbFormat::BC2Unorm),
            DxgiFormat::BC2_UNorm_sRGB => Ok(NutexbFormat::BC2Srgb),
            DxgiFormat::BC3_UNorm => Ok(NutexbFormat::BC3Unorm),
            DxgiFormat::BC3_UNorm_sRGB => Ok(NutexbFormat::BC3Srgb),
            DxgiFormat::BC4_UNorm => Ok(NutexbFormat::BC4Unorm),
            DxgiFormat::BC4_SNorm => Ok(NutexbFormat::BC4Snorm),
            DxgiFormat::BC5_UNorm => Ok(NutexbFormat::BC5Unorm),
            DxgiFormat::BC5_SNorm => Ok(NutexbFormat::BC5Snorm),
            DxgiFormat::BC6H_UF16 => Ok(NutexbFormat::BC6Ufloat),
            DxgiFormat::BC6H_SF16 => Ok(NutexbFormat::BC6Sfloat),
            DxgiFormat::BC7_UNorm => Ok(NutexbFormat::BC7Unorm),
            DxgiFormat::BC7_UNorm_sRGB => Ok(NutexbFormat::BC7Srgb),
            _ => Err(format!(
                "DDS DXGI format {:?} does not have a corresponding Nutexb format.",
                value
            )),
        }
    }
}

impl TryFrom<D3DFormat> for NutexbFormat {
    type Error = String;

    fn try_from(value: D3DFormat) -> Result<Self, Self::Error> {
        match value {
            D3DFormat::DXT1 => Ok(Self::BC1Unorm),
            D3DFormat::DXT2 => Ok(Self::BC2Unorm),
            D3DFormat::DXT3 => Ok(Self::BC2Unorm),
            D3DFormat::DXT4 => Ok(Self::BC3Unorm),
            D3DFormat::DXT5 => Ok(Self::BC3Unorm),
            _ => Err(format!(
                "DDS D3D format {:?} does not have a corresponding Nutexb format.",
                value
            )),
        }
    }
}

const BC5U: u32 = u32::from_le_bytes(*b"BC5U");
const ATI2: u32 = u32::from_le_bytes(*b"ATI2");

impl TryFrom<FourCC> for NutexbFormat {
    type Error = String;

    fn try_from(fourcc: FourCC) -> Result<Self, Self::Error> {
        match fourcc.0 {
            FourCC::DXT1 => Ok(Self::BC1Unorm),
            FourCC::DXT2 => Ok(Self::BC2Unorm),
            FourCC::DXT3 => Ok(Self::BC2Unorm),
            FourCC::DXT4 => Ok(Self::BC3Unorm),
            FourCC::DXT5 => Ok(Self::BC3Unorm),
            FourCC::BC4_UNORM => Ok(Self::BC4Unorm),
            FourCC::BC4_SNORM => Ok(Self::BC4Snorm),
            ATI2 | BC5U => Ok(Self::BC5Unorm),
            FourCC::BC5_SNORM => Ok(Self::BC5Snorm),
            _ => Err(format!(
                "DDS FOURCC {:x?} does not have a corresponding Nutexb format.",
                fourcc.0
            )),
        }
    }
}

impl From<NutexbFormat> for DxgiFormat {
    fn from(format: NutexbFormat) -> Self {
        match format {
            NutexbFormat::R8Unorm => DxgiFormat::R8_UNorm,
            NutexbFormat::R8G8B8A8Unorm => DxgiFormat::R8G8B8A8_UNorm,
            NutexbFormat::R8G8B8A8Srgb => DxgiFormat::R8G8B8A8_UNorm_sRGB,
            NutexbFormat::R32G32B32A32Float => DxgiFormat::R32G32B32A32_Float,
            NutexbFormat::B8G8R8A8Unorm => DxgiFormat::B8G8R8A8_UNorm,
            NutexbFormat::B8G8R8A8Srgb => DxgiFormat::B8G8R8A8_UNorm_sRGB,
            NutexbFormat::BC1Unorm => DxgiFormat::BC1_UNorm,
            NutexbFormat::BC1Srgb => DxgiFormat::BC1_UNorm_sRGB,
            NutexbFormat::BC2Unorm => DxgiFormat::BC2_UNorm,
            NutexbFormat::BC2Srgb => DxgiFormat::BC2_UNorm_sRGB,
            NutexbFormat::BC3Unorm => DxgiFormat::BC3_UNorm,
            NutexbFormat::BC3Srgb => DxgiFormat::BC3_UNorm_sRGB,
            NutexbFormat::BC4Unorm => DxgiFormat::BC4_UNorm,
            NutexbFormat::BC4Snorm => DxgiFormat::BC4_SNorm,
            NutexbFormat::BC5Unorm => DxgiFormat::BC5_UNorm,
            NutexbFormat::BC5Snorm => DxgiFormat::BC5_SNorm,
            NutexbFormat::BC6Ufloat => DxgiFormat::BC6H_UF16,
            NutexbFormat::BC6Sfloat => DxgiFormat::BC6H_SF16,
            NutexbFormat::BC7Unorm => DxgiFormat::BC7_UNorm,
            NutexbFormat::BC7Srgb => DxgiFormat::BC7_UNorm_sRGB,
        }
    }
}

pub fn create_dds(nutexb: &NutexbFile) -> Result<Dds, Box<dyn Error>> {
    let some_if_above_one = |x| if x > 0 { Some(x) } else { None };

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
