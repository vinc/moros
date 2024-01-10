use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys::console;
use crate::sys::fs::OpenFlag;
use crate::usr;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use core::str::{self, FromStr};
use smoltcp::wire::IpAddress;

#[derive(Debug)]
struct URL {
    pub host: String,
    pub port: u16,
    pub path: String,
}

enum ResponseState {
    Headers,
    Body,
}

impl URL {
    pub fn parse(url: &str) -> Option<Self> {
        if !url.starts_with("http://") {
            return None;
        }
        let url = &url[7..];
        let (server, path) = match url.find('/') {
            Some(i) => url.split_at(i),
            None => (url, "/"),
        };
        let (host, port) = match server.find(':') {
            Some(i) => server.split_at(i),
            None => (server, ":80"),
        };
        let port = &port[1..];
        Some(Self {
            host: host.into(),
            port: port.parse().unwrap_or(80),
            path: path.into(),
        })
    }
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_verbose = Style::color("LightBlue");
    let csi_reset = Style::reset();

    // Parse command line options
    let mut is_verbose = false;
    let mut host = "";
    let mut path = "";
    let mut timeout = 5.0;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                return help();
            }
            "-v" | "--verbose" => {
                is_verbose = true;
            }
            "-t" | "--timeout" => {
                if i + 1 < n {
                    timeout = args[i + 1].parse().unwrap_or(timeout);
                    i += 1;
                } else {
                    error!("Missing timeout seconds");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {
                if args[i].starts_with('-') {
                    error!("Invalid option '{}'", args[i]);
                    return Err(ExitCode::UsageError);
                } else if host.is_empty() {
                    host = args[i].
                        trim_start_matches("http://").
                        trim_start_matches("https://");
                } else if path.is_empty() {
                    path = args[i];
                } else {
                    error!("Too many arguments");
                    return Err(ExitCode::UsageError);
                }
            }
        }
        i += 1;
    }

    if host.is_empty() && path.is_empty() {
        error!("Missing URL");
        return Err(ExitCode::UsageError);
    } else if path.is_empty() {
        if let Some(i) = host.find('/') {
            (host, path) = host.split_at(i);
        } else {
            path = "/"
        }
    }

    let url = "http://".to_string() + host + path;
    let url = URL::parse(&url).expect("invalid URL format");
    let port = url.port;
    let addr = if url.host.ends_with(char::is_numeric) {
        match IpAddress::from_str(&url.host) {
            Ok(ip_addr) => ip_addr,
            Err(_) => {
                error!("Invalid address format");
                return Err(ExitCode::UsageError);
            }
        }
    } else {
        match usr::host::resolve(&url.host) {
            Ok(ip_addr) => ip_addr,
            Err(e) => {
                error!("Could not resolve host: {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };

    let socket_path = "/dev/net/tcp";
    let buf_len = if let Some(info) = syscall::info(socket_path) {
        info.size() as usize
    } else {
        error!("Could not open '{}'", socket_path);
        return Err(ExitCode::Failure);
    };

    let flags = OpenFlag::Device as usize;
    if let Some(handle) = syscall::open(socket_path, flags) {
        if syscall::connect(handle, addr, port).is_err() {
            error!("Could not connect to {}:{}", addr, port);
            syscall::close(handle);
            return Err(ExitCode::Failure);
        }
        let req = vec![
            format!("GET {} HTTP/1.1\r\n", url.path),
            format!("Host: {}\r\n", url.host),
            format!("User-Agent: MOROS/{}\r\n", env!("CARGO_PKG_VERSION")),
            format!("Connection: close\r\n"),
            format!("\r\n"),
        ];
        if is_verbose {
            print!("{}", csi_verbose);
            for line in &req {
                print!("> {}", line);
            }
            print!("{}", csi_reset);
        }
        let req = req.join("");
        syscall::write(handle, req.as_bytes());

        let mut response_state = ResponseState::Headers;
        loop {
            if console::end_of_text() || console::end_of_transmission() {
                eprintln!();
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }
            let mut data = vec![0; buf_len];
            if let Some(n) = syscall::read(handle, &mut data) {
                if n == 0 {
                    break;
                }
                data.resize(n, 0);
                let mut i = 0;
                while i < n {
                    match response_state {
                        ResponseState::Headers => {
                            let mut j = i;
                            while j < n {
                                if data[j] == b'\n' {
                                    break;
                                }
                                j += 1;
                            }
                            // TODO: check i == j
                            let line = String::from_utf8_lossy(&data[i..j]);
                            if is_verbose {
                                if i == 0 {
                                    print!("{}", csi_verbose);
                                }
                                println!("< {}", line);
                            }
                            if line.trim().is_empty() {
                                if is_verbose {
                                    print!("{}", csi_reset);
                                }
                                response_state = ResponseState::Body;
                            }
                            i = j + 1;
                        }
                        ResponseState::Body => {
                            // NOTE: The buffer may not be convertible to a
                            // UTF-8 string so we write it to STDOUT directly
                            // instead of using print.
                            syscall::write(1, &data[i..n]);
                            break;
                        }
                    }
                }
            } else {
                error!("Could not read from {}:{}", addr, port);
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }
        }
        syscall::close(handle);
        Ok(())
    } else {
        Err(ExitCode::Failure)
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} http {}<options> <url>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-v{1}, {0}--verbose{1}              Increase verbosity",
        csi_option, csi_reset
    );
    println!(
        "  {0}-t{1}, {0}--timeout <seconds>{1}    Request timeout",
        csi_option, csi_reset
    );
    Ok(())
}
