use std::sync::Arc;

use librespot_connect::{ConnectConfig, Spirc};
use librespot_core::Session;
use librespot_core::authentication::Credentials;
use librespot_core::config::DeviceType;
use librespot_core::dealer::Subscription;
use librespot_playback::audio_backend::Sink;
use librespot_playback::config::{AudioFormat, PlayerConfig};
use librespot_playback::mixer::softmixer::SoftMixer;
use librespot_playback::mixer::{Mixer, MixerConfig};
use librespot_playback::player::Player;

use crate::errors::AuthError;
use crate::null_sink::NullSink;

/// Output of the Spirc bootstrap. The caller is expected to spawn
/// `spirc_task` on a tokio runtime — without that the device registration
/// and dealer subscriptions go nowhere. `cluster_sub` streams
/// `hm://connect-state/v1/cluster` updates in parallel to Spirc's own
/// internal handling.
pub struct SpircBootstrap {
    pub spirc: Spirc,
    pub spirc_task: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
    pub cluster_sub: Subscription,
}

pub async fn start(session: Session, credentials: Credentials) -> Result<SpircBootstrap, AuthError> {
    // External cluster subscription must land BEFORE Spirc's own
    // dealer subs to be sure we register first in the listener map.
    // (Dealer allows multi-subscribe; this is belt-and-braces.)
    let cluster_sub = session
        .dealer()
        .add_listen_for("hm://connect-state/v1/cluster")
        .map_err(|e| AuthError::Server(format!("dealer cluster subscribe: {e}")))?;

    let connect_config = ConnectConfig {
        name: "Frostify".to_string(),
        device_type: DeviceType::Computer,
        ..Default::default()
    };

    let mixer: Arc<dyn Mixer> = Arc::new(
        SoftMixer::open(MixerConfig::default())
            .map_err(|e| AuthError::Server(format!("softmixer open: {e}")))?,
    );
    let volume_getter = mixer.get_soft_volume();

    let player = Player::new(
        PlayerConfig::default(),
        session.clone(),
        volume_getter,
        move || {
            Box::new(<NullSink as librespot_playback::audio_backend::Open>::open(
                None,
                AudioFormat::default(),
            )) as Box<dyn Sink>
        },
    );

    let (spirc, task) = Spirc::new(connect_config, session, credentials, player, mixer)
        .await
        .map_err(|e| AuthError::Server(format!("spirc init: {e}")))?;

    Ok(SpircBootstrap {
        spirc,
        spirc_task: Box::pin(task),
        cluster_sub,
    })
}
