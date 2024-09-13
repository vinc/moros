use crate::api::fs;
use crate::api::base64::Base64;
use crate::api::console::Style;
use crate::api::process::ExitCode;

use alloc::string::String;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError)
    }

    let path = args[1];

    if let Ok(mut buf) = fs::read_to_bytes(path) {
        buf.pop_if(|b| *b == b'\n');
        if let Ok(buf) = Base64::decode(&buf) {
            let decoded = String::from_utf8(buf).unwrap();
            println!("{}", decoded);
            return Ok(())
        }
    }

    error!("Could not decode '{}'", path);
    Err(ExitCode::Failure)
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} decode {}<file>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
