pub(crate) fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        panic!("Hex length should be 6 characters long");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).expect("Hex parser throw error");
    let g = u8::from_str_radix(&hex[2..4], 16).expect("Hex parser throw error");
    let b = u8::from_str_radix(&hex[4..6], 16).expect("Hex parser throw error");

    (r, g, b)
}