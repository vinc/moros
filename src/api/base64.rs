use alloc::vec::Vec;

pub struct Base64;

impl Base64 {
    pub fn encode(s: &[u8]) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(s.len() * 4 / 3 + 4, 0); // Resize to base64 + padding
        let bytes_written = base64::encode_config_slice(
            s, base64::STANDARD_NO_PAD, &mut buf
        );
        buf.resize(bytes_written, 0); // Resize back to actual size
        buf
    }

    pub fn decode(s: &[u8]) -> Result<Vec<u8>, ()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(s.len(), 0);
        let bytes_written = base64::decode_config_slice(
            s, base64::STANDARD_NO_PAD, &mut buf
        ).map_err(|_| ())?;
        buf.resize(bytes_written, 0);
        Ok(buf)
    }
}

#[test_case]
fn test_base64() {
    let encoded = b"SGVsbG8sIFdvcmxkIQ";
    let decoded = b"Hello, World!";
    assert_eq!(Base64::encode(decoded), encoded.to_vec());
    assert_eq!(Base64::decode(encoded), Ok(decoded.to_vec()));
}
