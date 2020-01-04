use crate::{print, user};
use heapless::{String, Vec};
use heapless::consts::*;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        user::shell::ExitCode::CommandError
    } else {
        let buf = encode(args[1].as_bytes());
        let encoded = String::from_utf8(buf).unwrap();
        print!("{}\n", encoded);
        user::shell::ExitCode::CommandSuccessful
    }
}

pub fn encode(s: &[u8]) -> Vec<u8, U1024> {
    let mut buf = Vec::<u8, U1024>::new();
    buf.resize(s.len() * 4 / 3 + 4, 0).unwrap(); // Resize to base64 + padding
    let bytes_written = base64::encode_config_slice(s, base64::STANDARD_NO_PAD, &mut buf);
    buf.resize(bytes_written, 0).unwrap(); // Resize back to actual size
    buf
}

pub fn decode(s: &[u8]) -> Vec<u8, U1024> {
    let mut buf = Vec::<u8, U1024>::new();
    buf.resize(s.len(), 0).unwrap();
    let bytes_written = base64::decode_config_slice(s, base64::STANDARD_NO_PAD, &mut buf).unwrap();
    buf.resize(bytes_written, 0).unwrap();
    buf
}
