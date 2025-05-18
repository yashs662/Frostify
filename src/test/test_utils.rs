use crate::{app::AppEvent, ui::asset::initialize_assets};
use std::sync::Once;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub fn get_event_sender() -> UnboundedSender<AppEvent> {
    let (event_tx, _) = unbounded_channel::<AppEvent>();
    event_tx
}

static INIT: Once = Once::new();

/// Call this function at the beginning of every test that requires initialized assets
pub fn setup_asset_store_for_testing() {
    // This will make sure our assets are initialized exactly once
    // across all tests, even when running in parallel
    INIT.call_once(|| {
        // Initialize the assets
        initialize_assets();
    });
}
