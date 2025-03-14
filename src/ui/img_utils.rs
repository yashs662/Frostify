use crate::{asset, errors::AssetError};
use image::GenericImageView;

pub struct RgbaImg {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl RgbaImg {
    pub fn new(file_name: &str) -> Result<Self, AssetError> {
        // Try to get embedded asset first
        if let Some(bytes) = asset::get_asset(file_name) {
            if let Ok(img) = image::load_from_memory(bytes) {
                let rgba = img.to_rgba8();
                let dimensions = img.dimensions();

                Ok(Self {
                    bytes: rgba.into_raw(),
                    width: dimensions.0,
                    height: dimensions.1,
                })
            } else {
                Err(AssetError::ImageLoadError)
            }
        } else {
            Err(AssetError::NotFound)
        }
    }
}
