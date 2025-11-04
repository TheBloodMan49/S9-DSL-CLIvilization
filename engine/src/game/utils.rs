use anyhow::{anyhow, Result};

pub fn hash_tmb(text: String) -> u32 {
    let mut hash: u32 = 2166136261; // FNV offset basis

    for byte in text.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619); // FNV prime
    }

    hash
}

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

pub fn write_to_file(filename: &str, content: &str) -> Result<()>{
    // Create output/ directory if it doesn't exist
    std::fs::create_dir_all("output")?;
    let filepath = format!("output/{}", filename);
    if let Err(e) = std::fs::write(&filepath, content) {
        //TODO: log
        return Err(anyhow!("Failed to write to file {}: {}", filepath, e));
    }
    //TODO: log success
    Ok(())
}
