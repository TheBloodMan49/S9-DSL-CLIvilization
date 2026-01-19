use anyhow::{Result, anyhow};

/// Hash a string using FNV-1a algorithm.
/// 
/// This is a simple non-cryptographic hash function useful for generating
/// deterministic seeds from text input.
/// 
/// # Arguments
/// * `text` - The string to hash
/// 
/// # Returns
/// A 32-bit hash value
/// 
/// Hash string using FNV-1a for deterministic seed generation. Non-cryptographic but fast and collision-resistant for game purposes.
pub fn hash_tmb(text: String) -> u32 {
    let mut hash: u32 = 2166136261; // FNV offset basis

    for byte in text.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(16777619); // FNV prime
    }

    hash
}

/// Convert HSV color values to RGB.
/// 
/// # Arguments
/// * `h` - Hue (0.0 to 360.0)
/// * `s` - Saturation (0.0 to 1.0)
/// * `v` - Value/Brightness (0.0 to 1.0)
/// 
/// # Returns
/// A tuple of (red, green, blue) values from 0 to 255
/// 
/// Convert HSV to RGB using standard color wheel math. Handles all hue ranges with continuous transitions.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h {
        h if (0.0..60.0).contains(&h) => (c, x, 0.0),
        h if (60.0..120.0).contains(&h) => (x, c, 0.0),
        h if (120.0..180.0).contains(&h) => (0.0, c, x),
        h if (180.0..240.0).contains(&h) => (0.0, x, c),
        h if (240.0..300.0).contains(&h) => (x, 0.0, c),
        h if (300.0..360.0).contains(&h) => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;

    (r, g, b)
}

/// Parse an HTML hex color string into a ratatui Color.
/// 
/// # Arguments
/// * `s` - Color string in the format "#RRGGBB"
/// 
/// # Returns
/// A ratatui Color, or white if the format is invalid
/// 
/// Parse HTML hex color (#RRGGBB) with graceful fallback to white. Tolerates invalid input without panicking.
pub fn str_to_color(s: &str) -> ratatui::style::Color {
    // Str is in html hex format: #RRGGBB
    if s.len() != 7 || !s.starts_with('#') {
        ratatui::style::Color::White
    } else {
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
        ratatui::style::Color::Rgb(r, g, b)
    }
}

/// Write to output/ directory with automatic creation. Rich error context aids debugging file I/O failures.
pub fn write_to_file(filename: &str, content: &str) -> Result<()> {
    // Create output/ directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all("output") {
        log::error!("Failed to create output/ directory: {e}");
        return Err(anyhow!("Failed to create output/ directory: {e}"));
    }
    let filepath = format!("output/{filename}");
    if let Err(e) = std::fs::write(&filepath, content) {
        log::error!("Failed to write to file {filepath}: {e}");
        return Err(anyhow!("Failed to write to file {filepath}: {e}"));
    }
    log::info!("Wrote file {filepath}");
    Ok(())
}
