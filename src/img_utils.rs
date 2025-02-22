use image::GenericImageView;
use log::{error, info};
use std::fs::File;
use std::io;
use std::io::Read;

pub struct RgbaImg {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl RgbaImg {
    pub fn new(file_path: &str) -> Option<Self> {
        if let Ok(file_bytes) = read_file_to_memory(file_path) {
            info!("Loading image from: {}", file_path);
            let dynamic_img = image::load_from_memory(&file_bytes[..]).unwrap();
            let rgba_img = dynamic_img.to_rgba8();
            let (width, height) = dynamic_img.dimensions();
            info!("Image loaded: {}x{}", width, height);
            Some(Self {
                width,
                height,
                bytes: rgba_img.into_raw(),
            })
        } else {
            error!("Failed to load image: {}", file_path);
            None
        }
    }
}

fn read_file_to_memory(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
