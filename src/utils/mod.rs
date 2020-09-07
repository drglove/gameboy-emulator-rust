#[allow(dead_code)]
pub fn dump_bytes(bytes: &[u8], filename: &str) {
    std::fs::write(filename, bytes).unwrap();
}
