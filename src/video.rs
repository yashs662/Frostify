//! Pure-Rust H.264/MP4 decode for Spotify Canvas clips.
//!
//! A Canvas is a short looping video (`re_mp4` demuxes the container,
//! `openh264` decodes the H.264 stream). No system `ffmpeg` dependency.
//!
//! MP4 stores each NAL unit length-prefixed (AVCC); openh264 wants
//! start-code-prefixed Annex-B. We convert each sample to Annex-B once
//! up front and keep the (still-compressed) access units in memory — a
//! Canvas is a few seconds at a small resolution, so the encoded clip is
//! a few MB. Decoding fully into RGBA would be hundreds of MB, so frames
//! are produced one at a time on demand and the clip loops forever.
//!
//! Samples are fed in decode order. Spotify Canvas clips are simple
//! (baseline/main profile, no B-frames in practice) so decode order
//! matches display order; we don't reorder by composition timestamp.

use std::time::Duration;

use openh264::OpenH264API;
use openh264::decoder::{Decoder, DecoderConfig, Flush};
use openh264::formats::YUVSource;
use re_mp4::{Mp4, StsdBoxContent, TrackKind};

/// One decoded frame: tightly-packed RGBA8 (`width * height * 4`) plus how
/// long it should stay on screen before the next one.
pub struct VideoFrame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub duration: Duration,
}

/// Demuxed H.264 Canvas clip + an openh264 decoder. Yields frames in
/// presentation order via [`CanvasVideo::next_frame`], looping at the end.
pub struct CanvasVideo {
    decoder: Decoder,
    /// Annex-B access units (one per sample, in decode order) + each
    /// sample's display duration.
    samples: Vec<(Vec<u8>, Duration)>,
    /// SPS/PPS as Annex-B, fed once up front and again on each loop so the
    /// decoder resets cleanly at the wrap.
    headers: Vec<u8>,
    next: usize,
    // Diagnostics (one-shot, first pass): decode outcome tally.
    diag_ok: u32,
    diag_none: u32,
    diag_err: u32,
    diag_done: bool,
}

const START_CODE: [u8; 4] = [0, 0, 0, 1];

impl CanvasVideo {
    /// Demux `mp4_bytes`, pick the first H.264 video track, and prime the
    /// decoder with its parameter sets. `None` if the bytes don't parse,
    /// carry no AVC video track, or the decoder can't initialise.
    pub fn open(mp4_bytes: &[u8]) -> Option<Self> {
        let mp4 = Mp4::read_bytes(mp4_bytes).ok()?;

        // First AVC (H.264) video track.
        let track = mp4
            .tracks()
            .values()
            .find(|t| t.kind == Some(TrackKind::Video))?;
        let avc1 = match &track.trak(&mp4).mdia.minf.stbl.stsd.contents {
            StsdBoxContent::Avc1(avc1) => avc1,
            _ => return None, // not H.264 — we only decode AVC Canvas clips
        };

        // NAL length prefix size (1..4 bytes), and SPS/PPS → Annex-B.
        let length_size = (avc1.avcc.length_size_minus_one & 0x3) as usize + 1;
        let mut headers = Vec::new();
        for nal in avc1
            .avcc
            .sequence_parameter_sets
            .iter()
            .chain(&avc1.avcc.picture_parameter_sets)
        {
            headers.extend_from_slice(&START_CODE);
            headers.extend_from_slice(&nal.bytes);
        }
        if headers.is_empty() {
            return None;
        }

        // Convert each sample's length-prefixed NALs to Annex-B.
        let mut samples = Vec::with_capacity(track.samples.len());
        for sample in &track.samples {
            let bytes = mp4_bytes.get(sample.byte_range())?;
            let au = avcc_to_annex_b(bytes, length_size)?;
            if au.is_empty() {
                continue;
            }
            // Clamp to a sane range: a corrupt/zero timescale or a bogus
            // last-sample duration can yield 0s (busy-spin) or thousands of
            // seconds (the thread sleeps forever, freezing on one frame).
            let secs = (sample.duration as f64 / track.timescale.max(1) as f64).clamp(0.01, 1.0);
            samples.push((au, Duration::from_secs_f64(secs)));
        }
        if samples.is_empty() {
            return None;
        }

        // NoFlush: the decoder is fed a continuous stream and looped, so it
        // must keep reference-frame state between AUs. The crate default
        // (`Flush::Flush`) flushes after every decode, which corrupts that
        // state mid-stream and errors every P-frame (only I-frames survive).
        let api = OpenH264API::from_source();
        let config = DecoderConfig::new().flush_after_decode(Flush::NoFlush);
        let mut decoder = Decoder::with_api_config(api, config).ok()?;
        // Prime with parameter sets (yields no picture).
        let _ = decoder.decode(&headers);

        Some(Self {
            decoder,
            samples,
            headers,
            next: 0,
            diag_ok: 0,
            diag_none: 0,
            diag_err: 0,
            diag_done: false,
        })
    }

    /// Number of demuxed samples (≈ frames) in the clip.
    pub fn frame_count(&self) -> usize {
        self.samples.len()
    }

    /// Decode and return the next frame, looping back to the start at the
    /// end of the clip. `None` only if the whole clip fails to yield a
    /// single decodable picture (corrupt stream) — callers fall back to art.
    pub fn next_frame(&mut self) -> Option<VideoFrame> {
        // Bound the scan so a fully-undecodable clip can't spin forever.
        let mut tries = self.samples.len() + 1;
        while tries > 0 {
            tries -= 1;
            if self.next >= self.samples.len() {
                if !self.diag_done {
                    log::info!(
                        "canvas decode pass: ok={} none={} err={} of {} samples",
                        self.diag_ok,
                        self.diag_none,
                        self.diag_err,
                        self.samples.len()
                    );
                    self.diag_done = true;
                }
                self.next = 0;
                // Re-feed parameter sets so the loop restart is clean.
                let _ = self.decoder.decode(&self.headers);
            }
            let idx = self.next;
            self.next += 1;
            let (au, dur) = &self.samples[idx];
            let dur = *dur;
            match self.decoder.decode(au) {
                Ok(Some(yuv)) => {
                    if !self.diag_done {
                        self.diag_ok += 1;
                    }
                    let (w, h) = yuv.dimensions();
                    let mut rgba = vec![0u8; w * h * 4];
                    yuv.write_rgba8(&mut rgba);
                    return Some(VideoFrame {
                        rgba,
                        width: w as u32,
                        height: h as u32,
                        duration: dur,
                    });
                }
                // Need more data, or a recoverable bitstream hiccup — try
                // the next access unit.
                Ok(None) => {
                    if !self.diag_done {
                        self.diag_none += 1;
                    }
                    continue;
                }
                Err(_) => {
                    if !self.diag_done {
                        self.diag_err += 1;
                    }
                    continue;
                }
            }
        }
        None
    }
}

/// Rewrite a length-prefixed (AVCC) sample into start-code-prefixed
/// (Annex-B) bytes. `None` if a declared NAL length runs past the buffer.
fn avcc_to_annex_b(mut buf: &[u8], length_size: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(buf.len() + 8);
    while buf.len() >= length_size {
        let mut len = 0usize;
        for &b in &buf[..length_size] {
            len = (len << 8) | b as usize;
        }
        buf = &buf[length_size..];
        let nal = buf.get(..len)?;
        out.extend_from_slice(&START_CODE);
        out.extend_from_slice(nal);
        buf = &buf[len..];
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avcc_single_nal_to_annex_b() {
        // 4-byte length (3) + payload AB CD EF.
        let sample = [0, 0, 0, 3, 0xAB, 0xCD, 0xEF];
        let out = avcc_to_annex_b(&sample, 4).unwrap();
        assert_eq!(out, [0, 0, 0, 1, 0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn avcc_two_nals_to_annex_b() {
        let sample = [0, 0, 0, 2, 0x11, 0x22, 0, 0, 0, 1, 0x33];
        let out = avcc_to_annex_b(&sample, 4).unwrap();
        assert_eq!(out, [0, 0, 0, 1, 0x11, 0x22, 0, 0, 0, 1, 0x33]);
    }

    #[test]
    fn avcc_truncated_length_is_none() {
        // Declares 10 bytes but only 2 follow.
        let sample = [0, 0, 0, 10, 0xAB, 0xCD];
        assert!(avcc_to_annex_b(&sample, 4).is_none());
    }

    #[test]
    fn open_garbage_is_none() {
        assert!(CanvasVideo::open(&[0xFF; 16]).is_none());
    }
}
