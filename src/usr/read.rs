use crate::{sys, usr};
use crate::api::syscall;
use crate::sys::cmos::CMOS;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    match pathname {
        "/dev/rtc" => {
            let rtc = CMOS::new().rtc();
            print!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}\n",
                rtc.year, rtc.month, rtc.day,
                rtc.hour, rtc.minute, rtc.second
            );
            usr::shell::ExitCode::CommandSuccessful
        },
        "/dev/clk/realtime" => {
            print!("{:.6}\n", syscall::realtime());
            usr::shell::ExitCode::CommandSuccessful
        },
        "/dev/clk/uptime" => {
            print!("{:.6}\n", syscall::uptime());
            usr::shell::ExitCode::CommandSuccessful
        },
        "/dev/random" => {
            loop {
                // Generate ASCII graphic chars
                let i = (sys::random::get_u32() % (0x72 - 0x20)) + 0x20;
                if let Some(c) = core::char::from_u32(i) {
                    print!("{}", c);
                }
                if sys::console::abort() {
                    print!("\n");
                    return usr::shell::ExitCode::CommandSuccessful;
                }
            }
        },
        _ => {
            if pathname.starts_with("/net/") {
                // Examples:
                // > read /net/http/example.com/articles
                // > read /net/http/example.com:8080/articles/index.html
                // > read /net/daytime/time.nist.gov
                // > read /net/tcp/time.nist.gov:13
                let parts: Vec<_> = pathname.split('/').collect();
                if parts.len() < 4 {
                    print!("Usage: read /net/http/<host>/<path>\n");
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
                            print!("Error: unknown protocol '{}'\n", parts[2]);
                            usr::shell::ExitCode::CommandError
                        }
                    }
                }
            } else if pathname.ends_with('/') {
                usr::list::main(args)
            } else if let Some(mut file) = sys::fs::File::open(pathname) {
                print!("{}", file.read_to_string());
                usr::shell::ExitCode::CommandSuccessful
            } else {
                print!("File not found '{}'\n", pathname);
                usr::shell::ExitCode::CommandError
            }
        }
    }
}
