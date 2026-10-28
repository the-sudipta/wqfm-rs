//! The wQFM-rs icon, embedded as a PNG and decoded at start.

const PNG: &[u8] = include_bytes!("../assets/wqfm-rs-256.png");

/// RGBA pixels and size of the icon.
pub fn rgba() -> Option<(Vec<u8>, u32, u32)> {
    let decoder = png::Decoder::new(std::io::Cursor::new(PNG));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some((buf, info.width, info.height))
}
