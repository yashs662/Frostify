use std::{collections::HashMap, sync::OnceLock};

static ASSET_STORE: OnceLock<HashMap<&'static str, &'static [u8]>> = OnceLock::new();

// macro to insert assets into the asset store
macro_rules! insert_assets {
    ($($file_name:literal),*) => {
        {
            let mut assets = HashMap::new();
            $(
                assets.insert(
                    $file_name,
                    include_bytes!(concat!("../../assets/", $file_name)) as &[u8],
                );
            )*
            ASSET_STORE
                .set(assets)
                .expect("Asset store already initialized");
        }
    };
}

pub fn initialize_assets() {
    insert_assets!(
        "close.png",
        "minimize.png",
        "maximize.png",
        "test.png",
        "album_art.png",
        "frostify_logo.png",
        "shuffle.png",
        "repeat.png",
        "skip-back.png",
        "play.png",
        "pause.png",
        "skip-forward.png",
        "volume.png"
    );
}

pub fn get_asset(file_name: &str) -> Option<&'static [u8]> {
    ASSET_STORE
        .get()
        .expect("Asset store not initialized")
        .get(file_name)
        .copied()
}
