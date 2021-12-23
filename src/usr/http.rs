use crate::{sys, usr};
use crate::api::syscall;
use crate::api::random;
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::str::{self, FromStr};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;

#[derive(Debug)]
struct URL {
    pub host: String,
    pub port: u16,
    pub path: String,
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

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    // Parse command line options
    let mut is_verbose = false;
    let mut args: Vec<String> = args.iter().map(ToOwned::to_owned).map(ToOwned::to_owned).filter(|arg| {
        if arg == "--verbose" {
            is_verbose = true;
        }
        !arg.starts_with("--")
    }).collect();

    // Split <host> and <path>
    if args.len() == 2 {
        if let Some(i) = args[1].find('/') {
            let arg = args[1].clone();
            let (host, path) = arg.split_at(i);
            args[1] = host.to_string();
            args.push(path.to_string());
        } else {
            args.push("/".to_string());
        }
    }

    if args.len() != 3 {
        eprintln!("Usage: http <host> <path>");
        return usr::shell::ExitCode::CommandError;
    }

    let url = "http://".to_owned() + &args[1] + &args[2];
    let url = URL::parse(&url).expect("invalid URL format");

    let address = if url.host.ends_with(char::is_numeric) {
        IpAddress::from_str(&url.host).expect("invalid address format")
    } else {
        match usr::host::resolve(&url.host) {
            Ok(ip_addr) => {
                ip_addr
            }
            Err(e) => {
                eprintln!("Could not resolve host: {:?}", e);
                return usr::shell::ExitCode::CommandError;
            }
        }
    };

    let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

    enum State { Connect, Request, Response }
    let mut state = State::Connect;

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        match iface.ipv4_addr() {
            None => {
                eprintln!("Error: Interface not ready");
                return usr::shell::ExitCode::CommandError;
            }
            Some(ip_addr) if ip_addr.is_unspecified() => {
                eprintln!("Error: Interface not ready");
                return usr::shell::ExitCode::CommandError;
            }
            _ => {}
        }

        let tcp_handle = iface.add_socket(tcp_socket);

        let mut is_header = true;
        let timeout = 5.0;
        let started = syscall::realtime();
        loop {
            if syscall::realtime() - started > timeout {
                eprintln!("Timeout reached");
                return usr::shell::ExitCode::CommandError;
            }
            if sys::console::end_of_text() {
                eprintln!();
                return usr::shell::ExitCode::CommandError;
            }
            let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                eprintln!("Network Error: {}", e);
            }

            let (socket, cx) = iface.get_socket_and_context::<TcpSocket>(tcp_handle);

            state = match state {
                State::Connect if !socket.is_active() => {
                    let local_port = 49152 + random::get_u16() % 16384;
                    if is_verbose {
                        println!("* Connecting to {}:{}", address, url.port);
                    }
                    if socket.connect(cx, (address, url.port), local_port).is_err() {
                        eprintln!("Could not connect to {}:{}", address, url.port);
                        iface.remove_socket(tcp_handle);
                        return usr::shell::ExitCode::CommandError;
                    }
                    State::Request
                }
                State::Request if socket.may_send() => {
                    let http_get = "GET ".to_string() + &url.path + " HTTP/1.1\r\n";
                    let http_host = "Host: ".to_string() + &url.host + "\r\n";
                    let http_ua = "User-Agent: MOROS/".to_string() + env!("CARGO_PKG_VERSION") + "\r\n";
                    let http_connection = "Connection: close\r\n".to_string();
                    if is_verbose {
                        print!("> {}", http_get);
                        print!("> {}", http_host);
                        print!("> {}", http_ua);
                        print!("> {}", http_connection);
                        println!(">");
                    }
                    socket.send_slice(http_get.as_ref()).expect("cannot send");
                    socket.send_slice(http_host.as_ref()).expect("cannot send");
                    socket.send_slice(http_ua.as_ref()).expect("cannot send");
                    socket.send_slice(http_connection.as_ref()).expect("cannot send");
                    socket.send_slice(b"\r\n").expect("cannot send");
                    State::Response
                }
                State::Response if socket.can_recv() => {
                    socket.recv(|data| {
                        let contents = String::from_utf8_lossy(data);
                        for line in contents.lines() {
                            if is_header {
                                if line.is_empty() {
                                    is_header = false;
                                }
                                if is_verbose {
                                    println!("< {}", line);
                                }
                            } else {
                                println!("{}", line);
                            }
                        }
                        (data.len(), ())
                    }).unwrap();
                    State::Response
                }
                State::Response if !socket.may_recv() => {
                    break;
                }
                _ => state
            };

            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
        iface.remove_socket(tcp_handle);
        usr::shell::ExitCode::CommandSuccessful
    } else {
        usr::shell::ExitCode::CommandError
    }
}
