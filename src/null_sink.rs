use librespot_playback::audio_backend::{Open, Sink, SinkResult};
use librespot_playback::config::AudioFormat;
use librespot_playback::convert::Converter;
use librespot_playback::decoder::AudioPacket;

/// Discards every decoded audio packet. Frostify registers as a Connect
/// device for state visibility; local audio playback comes
/// much later. Until then, librespot's `Player` still wants
/// somewhere to send PCM — this is /dev/null in struct form.
pub struct NullSink;

impl Open for NullSink {
    fn open(_device: Option<String>, _format: AudioFormat) -> Self {
        NullSink
    }
}

impl Sink for NullSink {
    fn write(&mut self, _packet: AudioPacket, _converter: &mut Converter) -> SinkResult<()> {
        Ok(())
    }
}
