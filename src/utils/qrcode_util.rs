use qrcode::{EcLevel, QrCode};

/// Render a QR code as terminal text using Unicode half-block characters.
/// Uses EcLevel::L (lowest error correction) for smallest possible QR size.
pub fn render_to_terminal(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::L)?;
    let colors = code.to_colors();
    let width = code.width();

    let mut output = String::new();
    let quiet = 1; // minimal quiet zone for compact display

    // Top quiet zone
    for _ in 0..quiet {
        output.push_str(&" ".repeat(width + quiet * 2));
        output.push('\n');
    }

    let mut y = 0;
    while y < width {
        output.push_str(&" ".repeat(quiet)); // left quiet zone

        for x in 0..width {
            let top = colors[y * width + x] == qrcode::Color::Dark;
            let bottom = if y + 1 < width {
                colors[(y + 1) * width + x] == qrcode::Color::Dark
            } else {
                false
            };

            match (top, bottom) {
                (true, true) => output.push('█'),
                (true, false) => output.push('▀'),
                (false, true) => output.push('▄'),
                (false, false) => output.push(' '),
            }
        }

        output.push_str(&" ".repeat(quiet)); // right quiet zone
        output.push('\n');
        y += 2;
    }

    // Bottom quiet zone
    for _ in 0..quiet {
        output.push_str(&" ".repeat(width + quiet * 2));
        output.push('\n');
    }

    Ok(output)
}

/// Decode a QR code from an image file
pub fn decode_from_image(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::open(path)?;
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);

    let grids = prepared.detect_grids();
    if grids.is_empty() {
        return Err("No QR code found in image".into());
    }

    let (_meta, content) = grids[0].decode()?;
    Ok(content)
}
