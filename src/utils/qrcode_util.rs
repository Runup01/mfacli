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

/// Decode a QR code from an image file.
///
/// If the QR code is too small / low-resolution to detect at native size,
/// retries with 2x/3x/4x upscaled variants (Nearest + Triangle filters)
/// before giving up.
pub fn decode_from_image(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::open(path)?;

    if let Some(content) = try_decode(&img) {
        return Ok(content);
    }

    for factor in [2u32, 3, 4] {
        for filter in [
            image::imageops::FilterType::Nearest,
            image::imageops::FilterType::Triangle,
        ] {
            let upscaled = image::imageops::resize(
                &img,
                img.width() * factor,
                img.height() * factor,
                filter,
            );
            let upscaled = image::DynamicImage::ImageRgba8(upscaled);
            if let Some(content) = try_decode(&upscaled) {
                return Ok(content);
            }
        }
    }

    Err("No QR code found in image (tried native + 2x/3x/4x upscaled)".into())
}

/// Single decode attempt on one image (detect all grids, return first decodable).
fn try_decode(img: &image::DynamicImage) -> Option<String> {
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    for grid in prepared.detect_grids() {
        if let Ok((_meta, content)) = grid.decode() {
            return Some(content);
        }
    }
    None
}
