use crate::{kernel, print, user};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    match pathname {
        "/dev/rtc" => {
            user::date::main(&["date", "--iso-8601"])
        },
        "/dev/clk/realtime" => {
            user::date::main(&["date", "--raw"])
        },
        "/dev/clk/uptime" => {
            user::uptime::main(&["uptime", "--raw"])
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
                    user::shell::ExitCode::CommandError
                } else {
                    match parts[2] {
                        "tcp" => {
                            let host = parts[3];
                            user::tcp::main(&["tcp", host])
                        }
                        "daytime" => {
                            let host = parts[3];
                            let port = "13";
                            user::tcp::main(&["tcp", host, port])
                        }
                        "http" => {
                            let host = parts[3];
                            let path = "/".to_owned() + &parts[4..].join("/");
                            user::http::main(&["http", host, &path])
                        }
                        _ => {
                            print!("Error: unknown protocol '{}'\n", parts[2]);
                            user::shell::ExitCode::CommandError
                        }
                    }
                }
            } else if pathname.ends_with('/') {
                user::list::main(args)
            } else if let Some(file) = kernel::fs::File::open(pathname) {
                print!("{}", file.read_to_string());
                user::shell::ExitCode::CommandSuccessful
            } else {
                print!("File not found '{}'\n", pathname);
                user::shell::ExitCode::CommandError
            }
        }
    }
}
