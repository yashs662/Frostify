//! Minimal decoder for the `EXTRACTED_COLOR` extended-metadata payload.
//!
//! librespot exposes the `/extended-metadata` endpoint and the
//! `ExtensionKind::EXTRACTED_COLOR` enum, but it does **not** compile
//! `extracted_colors.proto` into Rust types (the file ships in
//! librespot-protocol but is absent from its build list). Rather than
//! stand up our own protobuf codegen for a 4-field message, we
//! hand-decode the one field we need straight off the wire. Schema
//! (stable since Spotify ~1.2.x):
//!
//! ```proto
//! message ColorResult { Color color_raw = 1; Color color_light = 2;
//!                       Color color_dark = 3; Status status = 5; }
//! message Color { int32 rgb = 1; bool is_fallback = 2; }
//! ```
//!
//! We decode all three variants: `raw` is the dominant vibrant colour,
//! `light` is tuned for use over dark surfaces, `dark` is the background
//! tint the official client washes the now-playing UI with. Consumers
//! pick per context (see `widgets::color::chrome_accent`) — tinting
//! chrome icons with `dark` is how you get invisible-on-black accents.

/// The three extracted variants of a cover's palette. Any may be absent.
/// Serialized as-is into the JSON disk cache (facts, not derived policy —
/// the chrome's chosen/lifted accent is computed on read so tuning the
/// contrast rules never fights a 30-day cache).
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtractedColors {
    pub raw: Option<[f32; 4]>,
    pub light: Option<[f32; 4]>,
    pub dark: Option<[f32; 4]>,
}

/// Decode a serialized `ColorResult` into its variants, RGBA `[r, g, b,
/// 1.0]` (each 0..=1). `None` if nothing parses — the caller falls back
/// to the art-derived accent.
pub fn parse_colors(bytes: &[u8]) -> Option<ExtractedColors> {
    let c = ExtractedColors {
        raw: parse_variant(bytes, 1),
        light: parse_variant(bytes, 2),
        dark: parse_variant(bytes, 3),
    };
    (c.raw.is_some() || c.light.is_some() || c.dark.is_some()).then_some(c)
}

/// One `Color` sub-message (`field`: 1 = raw, 2 = light, 3 = dark) →
/// its `rgb` varint (field 1, packed 0xRRGGBB) as RGBA.
fn parse_variant(bytes: &[u8], field: u64) -> Option<[f32; 4]> {
    let sub = field_submessage(bytes, field)?;
    let rgb = field_varint(&sub, 1)?;
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    Some([r, g, b, 1.0])
}

/// Read a base-128 varint at `*pos`, advancing it. `None` on truncation
/// or an over-long (>64-bit) encoding.
fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    while *pos < buf.len() {
        let byte = buf[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

/// Skip the value of a field with the given wire type, advancing `pos`.
/// `None` if the value runs past the buffer or uses an unsupported wire
/// type (groups — not present in this schema).
fn skip_value(buf: &[u8], pos: &mut usize, wire: u64) -> Option<()> {
    match wire {
        0 => {
            read_varint(buf, pos)?;
        }
        1 => *pos = pos.checked_add(8)?,
        5 => *pos = pos.checked_add(4)?,
        2 => {
            let len = read_varint(buf, pos)? as usize;
            *pos = pos.checked_add(len)?;
        }
        _ => return None,
    }
    if *pos > buf.len() { None } else { Some(()) }
}

/// Bytes of the first length-delimited (wire type 2) field matching
/// `field`. `None` if absent.
fn field_submessage(buf: &[u8], field: u64) -> Option<Vec<u8>> {
    let mut pos = 0;
    while pos < buf.len() {
        let tag = read_varint(buf, &mut pos)?;
        let (fnum, wire) = (tag >> 3, tag & 0x7);
        if fnum == field && wire == 2 {
            let len = read_varint(buf, &mut pos)? as usize;
            let end = pos.checked_add(len)?;
            return buf.get(pos..end).map(|s| s.to_vec());
        }
        skip_value(buf, &mut pos, wire)?;
    }
    None
}

/// First varint (wire type 0) field matching `field`. `None` if absent.
fn field_varint(buf: &[u8], field: u64) -> Option<u64> {
    let mut pos = 0;
    while pos < buf.len() {
        let tag = read_varint(buf, &mut pos)?;
        let (fnum, wire) = (tag >> 3, tag & 0x7);
        if fnum == field && wire == 0 {
            return read_varint(buf, &mut pos);
        }
        skip_value(buf, &mut pos, wire)?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-encode one `Color { rgb }` sub-message under `field`.
    fn encode_variant(field: u8, rgb: u32) -> Vec<u8> {
        let mut inner = vec![0x08]; // Color.rgb: field 1, varint
        let mut v = rgb as u64;
        loop {
            let mut byte = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 {
                byte |= 0x80;
            }
            inner.push(byte);
            if v == 0 {
                break;
            }
        }
        let mut out = vec![field << 3 | 2, inner.len() as u8];
        out.extend(inner);
        out
    }

    #[test]
    fn decodes_all_three_variants() {
        let mut bytes = encode_variant(1, 0xFF0000);
        bytes.extend(encode_variant(2, 0x00FF00));
        bytes.extend(encode_variant(3, 0x0000FF));
        let c = parse_colors(&bytes).unwrap();
        assert_eq!(c.raw.unwrap(), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(c.light.unwrap(), [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(c.dark.unwrap(), [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn decodes_mixed_color() {
        // 0x7D765E — the dark colour from the live capture.
        let c = parse_colors(&encode_variant(3, 0x7D765E)).unwrap();
        let dark = c.dark.unwrap();
        assert!((dark[0] - 0x7D as f32 / 255.0).abs() < 1e-6);
        assert!((dark[1] - 0x76 as f32 / 255.0).abs() < 1e-6);
        assert!((dark[2] - 0x5E as f32 / 255.0).abs() < 1e-6);
        assert!(c.raw.is_none() && c.light.is_none());
    }

    #[test]
    fn partial_payload_keeps_present_variants() {
        // Only color_raw present → raw decoded, others None.
        let c = parse_colors(&encode_variant(1, 0x123456)).unwrap();
        assert!(c.raw.is_some() && c.light.is_none() && c.dark.is_none());
    }

    #[test]
    fn garbage_is_none_not_panic() {
        assert!(parse_colors(&[0xFF, 0xFF, 0xFF]).is_none());
        assert!(parse_colors(&[]).is_none());
    }
}
