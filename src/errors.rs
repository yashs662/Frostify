// create a error type for asset loading errors
#[derive(Debug)]
pub enum AssetError {
    NotFound,
    ImageLoadError,
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::NotFound => write!(f, "Asset not found"),
            AssetError::ImageLoadError => write!(f, "Failed to load image"),
        }
    }
}
