use alloc::vec::Vec;
use base64::prelude::*;

pub struct Base64;

impl Base64 {
    pub fn encode(s: &[u8]) -> Vec<u8> {
        BASE64_STANDARD_NO_PAD.encode(s).as_bytes().to_vec()
    }

    pub fn decode(s: &[u8]) -> Result<Vec<u8>, ()> {
        BASE64_STANDARD_NO_PAD.decode(s).map_err(|_| ())
    }
}

#[test_case]
fn test_base64() {
    let tests = [
        (b"abcdefghijklm", b"YWJjZGVmZ2hpamtsbQ"),
        (b"Hello, World!", b"SGVsbG8sIFdvcmxkIQ"),
        (b"~~~~~, ?????!", b"fn5+fn4sID8/Pz8/IQ"),
    ];
    for (decoded, encoded) in tests {
        assert_eq!(Base64::encode(decoded), encoded.to_vec());
        assert_eq!(Base64::decode(encoded), Ok(decoded.to_vec()));
    }
}
