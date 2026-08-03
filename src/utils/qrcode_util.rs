use qrcode::{EcLevel, QrCode};

/// Render a QR code as terminal text.
///
/// `half` (default): half-block glyphs ('▀'/'▄'/'█'), two module rows per
/// terminal row — smallest and square; crisp on well-behaved terminals.
/// `block`: only '█' + space, one module per cell — for terminals whose
/// fonts / line-spacing misrender half-block glyphs.
pub fn render_qr(data: &str, style: &str) -> Result<String, Box<dyn std::error::Error>> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::L)?;
    let colors = code.to_colors();
    let width = code.width();
    let quiet = 1;
    let blank = " ".repeat(width + quiet * 2);

    let mut output = String::new();
    if style == "block" {
        for _ in 0..quiet {
            output.push_str(&blank);
            output.push('\n');
        }
        for y in 0..width {
            output.push_str(&" ".repeat(quiet));
            for x in 0..width {
                output.push(if colors[y * width + x] == qrcode::Color::Dark {
                    '█'
                } else {
                    ' '
                });
            }
            output.push_str(&" ".repeat(quiet));
            output.push('\n');
        }
    } else {
        output.push_str(&blank);
        output.push('\n');
        let mut y = 0;
        while y < width {
            output.push(' ');
            for x in 0..width {
                let top = colors[y * width + x] == qrcode::Color::Dark;
                let bottom = if y + 1 < width {
                    colors[(y + 1) * width + x] == qrcode::Color::Dark
                } else {
                    false
                };
                output.push(match (top, bottom) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                });
            }
            output.push(' ');
            output.push('\n');
            y += 2;
        }
    }
    output.push_str(&blank);
    output.push('\n');

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
