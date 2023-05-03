use crate::api::process::ExitCode;

use alloc::string::String;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        Err(ExitCode::UsageError)
    } else {
        let buf = encode(args[1].as_bytes());
        let encoded = String::from_utf8(buf).unwrap();
        println!("{}", encoded);
        Ok(())
    }
}

pub fn encode(s: &[u8]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(s.len() * 4 / 3 + 4, 0); // Resize to base64 + padding
    let bytes_written = base64::encode_config_slice(s, base64::STANDARD_NO_PAD, &mut buf);
    buf.resize(bytes_written, 0); // Resize back to actual size
    buf
}

pub fn decode(s: &[u8]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(s.len(), 0);
    let bytes_written = base64::decode_config_slice(s, base64::STANDARD_NO_PAD, &mut buf).unwrap();
    buf.resize(bytes_written, 0);
    buf
}
