use std::{collections::HashMap, sync::OnceLock};

static ASSET_STORE: OnceLock<HashMap<&'static str, &'static [u8]>> = OnceLock::new();

// macro to insert assets into the asset store
macro_rules! insert_assets {
    ($($item:tt)*) => {
        {
            let mut assets = HashMap::new();
            insert_assets_helper!(assets; $($item)*);
            ASSET_STORE
                .set(assets)
                .expect("Asset store already initialized");
        }
    };
}

macro_rules! insert_assets_helper {
    ($assets:ident; $folder:literal / $file_name:literal as $access_name:literal, $($rest:tt)*) => {
        $assets.insert(
            $access_name,
            include_bytes!(concat!("../../assets/", $folder, "/", $file_name)) as &[u8],
        );
        insert_assets_helper!($assets; $($rest)*);
    };
    ($assets:ident; $folder:literal / $file_name:literal, $($rest:tt)*) => {
        $assets.insert(
            $file_name,
            include_bytes!(concat!("../../assets/", $folder, "/", $file_name)) as &[u8],
        );
        insert_assets_helper!($assets; $($rest)*);
    };
    ($assets:ident; $file_name:literal as $access_name:literal, $($rest:tt)*) => {
        $assets.insert(
            $access_name,
            include_bytes!(concat!("../../assets/", $file_name)) as &[u8],
        );
        insert_assets_helper!($assets; $($rest)*);
    };
    ($assets:ident; $file_name:literal, $($rest:tt)*) => {
        $assets.insert(
            $file_name,
            include_bytes!(concat!("../../assets/", $file_name)) as &[u8],
        );
        insert_assets_helper!($assets; $($rest)*);
    };
    ($assets:ident;) => {};
}

/// Initializes the global asset store with embedded assets.
///
/// This function must be called once before using `get_asset()` to retrieve assets.
/// It uses the `insert_assets!` macro to embed assets at compile time.
///
/// # Asset Syntax
/// The macro supports several formats for including assets:
///
/// - `"file.ext"` - Include asset from root assets folder with filename as key
/// - `"file.ext" as "key"` - Include asset with custom key name
/// - `"folder" / "file.ext"` - Include asset from subfolder with filename as key
/// - `"folder" / "file.ext" as "key"` - Include asset from subfolder with custom key
///
/// # Examples
/// ```rust
/// insert_assets!(
///     "logo.png",                           // Key: "logo.png"
///     "background.jpg" as "bg",             // Key: "bg"
///     "icons" / "play.png",                 // Key: "play.png"
///     "fonts" / "Arial.ttf" as "main_font", // Key: "main_font"
/// );
/// ```
///
/// # Panics
/// Panics if called more than once, as the asset store can only be initialized once.
pub fn initialize_assets() {
    insert_assets!(
        // Icons
        "icons" / "close.png",
        "icons" / "minimize.png",
        "icons" / "maximize.png",
        "icons" / "settings.png",
        "icons" / "shuffle.png",
        "icons" / "repeat.png",
        "icons" / "skip-back.png",
        "icons" / "play.png",
        "icons" / "pause.png",
        "icons" / "skip-forward.png",
        "icons" / "volume.png",

        // General Art
        "album_art.png",
        "frostify_logo.png",
        "test.png",

        // Fonts
        "fonts" / "CenturyGothic.ttf" as "CenturyGothic",
        "fonts" / "CenturyGothicBold.ttf" as "CenturyGothicBold",
    );
}

pub fn get_asset(file_name: &str) -> Option<&'static [u8]> {
    ASSET_STORE
        .get()
        .expect("Asset store not initialized")
        .get(file_name)
        .copied()
}
