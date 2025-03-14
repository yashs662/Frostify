use std::{collections::HashMap, sync::OnceLock};

static ASSET_STORE: OnceLock<HashMap<&'static str, &'static [u8]>> = OnceLock::new();

pub fn initialize_assets() {
    let mut assets = HashMap::new();
    assets.insert(
        "close.png",
        include_bytes!("../../assets/close.png") as &[u8],
    );
    assets.insert(
        "minimize.png",
        include_bytes!("../../assets/minimize.png") as &[u8],
    );
    assets.insert(
        "maximize.png",
        include_bytes!("../../assets/maximize.png") as &[u8],
    );
    assets.insert("test.png", include_bytes!("../../assets/test.png") as &[u8]);
    assets.insert(
        "album_art.png",
        include_bytes!("../../assets/album_art.png") as &[u8],
    );

    ASSET_STORE
        .set(assets)
        .expect("Asset store already initialized");
}

pub fn get_asset(file_name: &str) -> Option<&'static [u8]> {
    ASSET_STORE
        .get()
        .expect("Asset store not initialized")
        .get(file_name)
        .copied()
}
