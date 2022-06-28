use crate::{api, sys, usr};
use crate::api::console;
use crate::api::fs;
use crate::api::syscall;
use crate::api::process::ExitCode;
use crate::sys::cmos::CMOS;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::convert::TryInto;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        return Err(ExitCode::Failure);
    }

    let mut path = args[1];

    // The commands `read /usr/alice/` and `read /usr/alice` are equivalent,
    // but `read /` should not be modified.
    if path.len() > 1 {
        path = path.trim_end_matches('/');
    }

    // TODO: Create device drivers for `/dev` and `/net` hardcoded commands
    match path {
        "/dev/rtc" => {
            let rtc = CMOS::new().rtc();
            println!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                rtc.year, rtc.month, rtc.day,
                rtc.hour, rtc.minute, rtc.second
            );
            Ok(())
        },
        _ => {
            if path.starts_with("/net/") {
                // Examples:
                // > read /net/http/example.com/articles
                // > read /net/http/example.com:8080/articles/index.html
                // > read /net/daytime/time.nist.gov
                // > read /net/tcp/time.nist.gov:13
                let parts: Vec<_> = path.split('/').collect();
                if parts.len() < 4 {
                    eprintln!("Usage: read /net/http/<host>/<path>");
                    Err(ExitCode::Failure)
                } else {
                    match parts[2] {
                        "tcp" => {
                            let host = parts[3];
                            usr::tcp::main(&["tcp", host])
                        }
                        "daytime" => {
                            let host = parts[3];
                            let port = "13";
                            usr::tcp::main(&["tcp", host, port])
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
            } else if let Some(info) = syscall::info(path) {
                if info.is_file() {
                    if let Ok(contents) = api::fs::read_to_string(path) {
                        print!("{}", contents);
                        Ok(())
                    } else {
                        error!("Could not read '{}'", path);
                        Err(ExitCode::Failure)
                    }
                } else if info.is_dir() {
                    usr::list::main(args)
                } else if info.is_device() {
                    let is_char_device = info.size() == 4; // TODO: Improve device detection
                    let is_float_device = info.size() == 8; // TODO: Improve device detection
                    loop {
                        if sys::console::end_of_text() || sys::console::end_of_transmission() {
                            println!();
                            return Ok(());
                        }
                        if let Ok(bytes) = fs::read_to_bytes(path) {
                            if is_char_device && bytes.len() == 1 {
                                match bytes[0] as char {
                                    console::ETX_KEY => {
                                        println!("^C");
                                        return Ok(());
                                    }
                                    console::EOT_KEY => {
                                        println!("^D");
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                            if is_float_device {
                                if bytes.len() == 8 {
                                    println!("{:.6}", f64::from_be_bytes(bytes[0..8].try_into().unwrap()));
                                    return Ok(());
                                }
                            }
                            for b in bytes {
                                print!("{}", b as char);
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
                error!("File not found '{}'", path);
                Err(ExitCode::Failure)
            }
        }
    }
}
