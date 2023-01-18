pub fn hex_to_rgb(hex: &String) -> [u8; 3] {
    let mut rgb = [0; 3];
    let hex = hex.as_str().strip_prefix("#").unwrap_or("000000");
    for i in 0..3 {
        rgb[i] = u8::from_str_radix(&hex[i * 2..(i * 2) + 2], 16).unwrap_or(0);
    }
    rgb
}
