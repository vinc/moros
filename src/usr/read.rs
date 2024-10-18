use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys::console;
use crate::{api, usr};

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::vec::Vec;
use core::convert::TryInto;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }
    let mut path = args[1];

    // The commands `read /usr/alice/` and `read /usr/alice` are equivalent,
    // but `read /` should not be modified.
    if path.len() > 1 {
        path = path.trim_end_matches('/');
    }

    // TODO: Create device drivers for `/net` hardcoded commands
    if path.starts_with("/net/") {
        let csi_option = Style::color("aqua");
        let csi_title = Style::color("yellow");
        let csi_reset = Style::reset();
        // Examples:
        // > read /net/http/example.com/articles
        // > read /net/http/example.com:8080/articles/index.html
        // > read /net/daytime/time.nist.gov
        // > read /net/tcp/time.nist.gov:13
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() < 4 {
            println!(
                "{}Usage:{} read {}/net/<proto>/<host>[:<port>]/<path>{1}",
                csi_title, csi_reset, csi_option
            );
            Err(ExitCode::Failure)
        } else {
            let host = parts[3];
            match parts[2] {
                "tcp" => {
                    if host.contains(':') {
                        usr::tcp::main(&["tcp", host])
                    } else {
                        error!("Missing port number");
                        Err(ExitCode::Failure)
                    }
                }
                "daytime" => {
                    if host.contains(':') {
                        usr::tcp::main(&["tcp", host])
                    } else {
                        usr::tcp::main(&["tcp", &format!("{}:13", host)])
                    }
                }
                "http" => {
                    let host = parts[3];
                    let path = "/".to_owned() + &parts[4..].join("/");
                    usr::http::main(&["http", host, &path])
                }
                _ => {
                    error!("Unknown protocol '{}'", parts[2]);
                    Err(ExitCode::Failure)
                }
            }
        }
    } else if path.ends_with(".bmp") {
        usr::render::main(args)
    } else if let Some(info) = syscall::info(path) {
        if info.is_file() {
            if let Ok(buf) = api::fs::read_to_bytes(path) {
                syscall::write(1, &buf);
                Ok(())
            } else {
                error!("Could not read '{}'", path);
                Err(ExitCode::Failure)
            }
        } else if info.is_dir() {
            usr::list::main(args)
        } else if info.is_device() {
            // TODO: Add a way to read the device file to get its type directly
            // instead of relying on the various device file sizes. We could
            // maybe allow `sys::fs::file::File::open()` to open a Device file
            // as a regular file and read the type in the first byte of the
            // file.
            let n = info.size();
            let is_char_device = n == 4;
            let is_float_device = n == 8;
            let is_block_device = n > 8;
            loop {
                if console::end_of_text() || console::end_of_transmission() {
                    println!();
                    return Ok(());
                }
                if let Ok(bytes) = fs::read_to_bytes(path) {
                    if is_char_device && bytes.len() == 1 {
                        match bytes[0] as char {
                            api::console::ETX_KEY => {
                                println!("^C");
                                return Ok(());
                            }
                            api::console::EOT_KEY => {
                                println!("^D");
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    if is_float_device && bytes.len() == 8 {
                        let f = f64::from_be_bytes(bytes[0..8].try_into().
                            unwrap());
                        println!("{:.6}", f);
                        return Ok(());
                    }
                    for b in bytes {
                        print!("{}", b as char);
                    }
                    if is_block_device {
                        println!();
                        return Ok(());
                    }
                } else {
                    error!("Could not read '{}'", path);
                    return Err(ExitCode::Failure);
                }
            }
        } else {
            error!("Could not read type of '{}'", path);
            Err(ExitCode::Failure)
        }
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} read {}<path>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Paths:{}", csi_title, csi_reset);
    println!("  {0}<dir>/{1}     Read directory", csi_option, csi_reset);
    println!("  {0}<file>{1}     Read file", csi_option, csi_reset);
}
