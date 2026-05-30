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
//! We read `color_dark` (field 3) → its `rgb` (field 1): the dark
//! vibrant colour the official client tints the now-playing UI with.

/// Decode `color_dark.rgb` from a serialized `ColorResult` into RGBA
/// `[r, g, b, 1.0]` (each 0..=1). `None` if the field is missing or the
/// bytes don't parse — the caller falls back to the art-derived accent.
pub fn parse_color_dark(bytes: &[u8]) -> Option<[f32; 4]> {
    // color_dark = field 3, wire type 2 (length-delimited sub-message).
    let sub = field_submessage(bytes, 3)?;
    // Color.rgb = field 1, wire type 0 (varint), packed 0xRRGGBB.
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

    /// Hand-encode `ColorResult { color_dark: Color { rgb } }`.
    fn encode(rgb: u32) -> Vec<u8> {
        let mut inner = vec![0x08]; // field 1, varint
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
        let mut out = vec![0x1A, inner.len() as u8]; // field 3, len-delimited
        out.extend(inner);
        out
    }

    #[test]
    fn decodes_pure_blue() {
        let accent = parse_color_dark(&encode(0x0000FF)).unwrap();
        assert!(accent[0].abs() < 1e-6 && accent[1].abs() < 1e-6);
        assert!((accent[2] - 1.0).abs() < 1e-6);
        assert_eq!(accent[3], 1.0);
    }

    #[test]
    fn decodes_mixed_color() {
        // 0x7D765E — the dark colour from the live capture.
        let accent = parse_color_dark(&encode(0x7D765E)).unwrap();
        assert!((accent[0] - 0x7D as f32 / 255.0).abs() < 1e-6);
        assert!((accent[1] - 0x76 as f32 / 255.0).abs() < 1e-6);
        assert!((accent[2] - 0x5E as f32 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn missing_field_is_none() {
        // A ColorResult carrying only color_raw (field 1) → no dark.
        assert!(parse_color_dark(&[0x0A, 0x02, 0x08, 0x01]).is_none());
    }

    #[test]
    fn garbage_is_none_not_panic() {
        assert!(parse_color_dark(&[0xFF, 0xFF, 0xFF]).is_none());
        assert!(parse_color_dark(&[]).is_none());
    }
}
