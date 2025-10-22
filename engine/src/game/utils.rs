pub fn hash_tmb(text: String) -> u32 {
    let mut hash: u32 = 2166136261; // FNV offset basis

    for byte in text.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619); // FNV prime
    }

    hash
}
