pub fn make_header(config: &crate::config::Config, code_size: u16, sprite_size: u16) -> Vec<u8> {
    const HEADER_SIZE: usize = 128;
    let mut header = vec![0; HEADER_SIZE];

    header[0x00] = b'A';
    header[0x01] = b'Y';
    header[0x02] = b'A';

    header[0x04] = 1;

    assert!(config.name.len() <= 63);
    for (i, c) in config.name.chars().enumerate() {
        header[0x05 + i] = c as u8;
    }

    header[0x44] = 0x80;
    header[0x45] = 0x00;

    let [lower, upper] = u16::to_le_bytes(code_size);
    header[0x46] = lower;
    header[0x47] = upper;

    let [lower, upper] = u16::to_le_bytes(code_size + HEADER_SIZE as u16);
    header[0x48] = lower;
    header[0x49] = upper;

    let [lower, upper] = u16::to_le_bytes(sprite_size);
    header[0x4A] = lower;
    header[0x4B] = upper;

    header
}
