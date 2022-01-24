use crate::{api, sys, usr};
use crate::api::fs;
use crate::api::syscall;
use crate::sys::cmos::CMOS;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let mut path = args[1];

    // The commands `read /usr/alice/` and `read /usr/alice` are equivalent,
    // but `read /` should not be modified.
    if path.len() > 1 {
        path = path.trim_end_matches('/');
    }

    match path {
        "/dev/rtc" => {
            let rtc = CMOS::new().rtc();
            println!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                rtc.year, rtc.month, rtc.day,
                rtc.hour, rtc.minute, rtc.second
            );
            usr::shell::ExitCode::CommandSuccessful
        },
        "/dev/clk/realtime" => {
            println!("{:.6}", syscall::realtime());
            usr::shell::ExitCode::CommandSuccessful
        },
        "/dev/clk/uptime" => {
            println!("{:.6}", syscall::uptime());
            usr::shell::ExitCode::CommandSuccessful
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
                    usr::shell::ExitCode::CommandError
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
                            eprintln!("Error: unknown protocol '{}'", parts[2]);
                            usr::shell::ExitCode::CommandError
                        }
                    }
                }
            } else if let Some(info) = syscall::info(path) {
                if info.is_file() {
                    if let Ok(contents) = api::fs::read_to_string(path) {
                        print!("{}", contents);
                        usr::shell::ExitCode::CommandSuccessful
                    } else {
                        eprintln!("Could not read '{}'", path);
                        usr::shell::ExitCode::CommandError
                    }
                } else if info.is_dir() {
                    usr::list::main(args)
                } else if info.is_device() {
                    loop {
                        if sys::console::end_of_text() {
                            println!();
                            return usr::shell::ExitCode::CommandSuccessful;
                        }
                        if let Ok(bytes) = fs::read_to_bytes(path) {
                            for b in bytes {
                                print!("{}", b as char);
                            }
                        } else {
                            eprintln!("Could not read '{}'", path);
                            return usr::shell::ExitCode::CommandError;
                        }
                    }
                } else {
                    eprintln!("Could not read type of '{}'", path);
                    usr::shell::ExitCode::CommandError
                }
            } else {
                eprintln!("File not found '{}'", path);
                usr::shell::ExitCode::CommandError
            }
        }
    }
}
