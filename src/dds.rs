use std::{
    convert::{TryFrom, TryInto},
    error::Error,
};

use ddsfile::{AlphaMode, Caps2, D3D10ResourceDimension, Dds, DxgiFormat, MiscFlag, NewDxgiParams};

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
        // TODO: Try dxgi format, then try d3d, then error?
        let format = self.get_dxgi_format().unwrap().try_into()?;
        Ok(format)
    }
}

impl TryFrom<DxgiFormat> for NutexbFormat {
    type Error = String;

    fn try_from(value: DxgiFormat) -> Result<Self, Self::Error> {
        // DDS supports all the known nutexb formats.
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
                "{:?} is not a supported Nutexb image format.",
                value
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

// TODO: Support D3D format types as well?

pub fn create_dds(nutexb: &NutexbFile) -> Dds {
    // TODO: 3D Support.
    // We don't actually need to set the array count here.
    // Cube maps are set using the appropriate flag.
    // Setting both the flag and arrays would create an array of 6 cube maps.
    // TODO: ddsfile has no way of reading this flag?
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
    dds.data = nutexb.deswizzled_data();

    dds
}
