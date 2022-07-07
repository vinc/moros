use crate::{sys, usr};
use crate::api::console::Style;
use crate::api::clock;
use crate::api::process::ExitCode;
use crate::api::random;
use crate::api::syscall;
use alloc::string::{String, ToString};
use alloc::vec;
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

enum SessionState { Connect, Request, Response }
enum ResponseState { Headers, Body }

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
    let n = args.len();
    for i in 1..n {
        match args[i] {
            "-h" | "--help" => {
                return help();
            }
            "--verbose" => {
                is_verbose = true;
            }
            _ => {
                if args[i].starts_with("-") {
                    error!("Invalid option '{}'", args[i]);
                    return Err(ExitCode::UsageError);
                } else if host.is_empty() {
                    host = args[i];
                } else if path.is_empty() {
                    path = args[i];
                } else {
                    error!("Too many arguments");
                    return Err(ExitCode::UsageError);
                }
            }
        }
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

    let address = if url.host.ends_with(char::is_numeric) {
        IpAddress::from_str(&url.host).expect("invalid address format")
    } else {
        match usr::host::resolve(&url.host) {
            Ok(ip_addr) => {
                ip_addr
            }
            Err(e) => {
                error!("Could not resolve host: {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };

    let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

    let mut session_state = SessionState::Connect;

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        let tcp_handle = iface.add_socket(tcp_socket);
        let timeout = 5.0;
        let started = clock::realtime();
        let mut response_state = ResponseState::Headers;
        loop {
            if clock::realtime() - started > timeout {
                error!("Timeout reached");
                iface.remove_socket(tcp_handle);
                return Err(ExitCode::Failure);
            }
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                eprintln!();
                iface.remove_socket(tcp_handle);
                return Err(ExitCode::Failure);
            }
            let timestamp = Instant::from_micros((clock::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let (socket, cx) = iface.get_socket_and_context::<TcpSocket>(tcp_handle);

            session_state = match session_state {
                SessionState::Connect if !socket.is_active() => {
                    let local_port = 49152 + random::get_u16() % 16384;
                    if is_verbose {
                        print!("{}", csi_verbose);
                        println!("* Connecting to {}:{}", address, url.port);
                        print!("{}", csi_reset);
                    }
                    if socket.connect(cx, (address, url.port), local_port).is_err() {
                        error!("Could not connect to {}:{}", address, url.port);
                        iface.remove_socket(tcp_handle);
                        return Err(ExitCode::Failure);
                    }
                    SessionState::Request
                }
                SessionState::Request if socket.may_send() => {
                    let http_get = "GET ".to_string() + &url.path + " HTTP/1.1\r\n";
                    let http_host = "Host: ".to_string() + &url.host + "\r\n";
                    let http_ua = "User-Agent: MOROS/".to_string() + env!("CARGO_PKG_VERSION") + "\r\n";
                    let http_connection = "Connection: close\r\n".to_string();
                    if is_verbose {
                        print!("{}", csi_verbose);
                        print!("> {}", http_get);
                        print!("> {}", http_host);
                        print!("> {}", http_ua);
                        print!("> {}", http_connection);
                        println!(">");
                        print!("{}", csi_reset);
                    }
                    socket.send_slice(http_get.as_ref()).expect("cannot send");
                    socket.send_slice(http_host.as_ref()).expect("cannot send");
                    socket.send_slice(http_ua.as_ref()).expect("cannot send");
                    socket.send_slice(http_connection.as_ref()).expect("cannot send");
                    socket.send_slice(b"\r\n").expect("cannot send");
                    SessionState::Response
                }
                SessionState::Response if socket.can_recv() => {
                    socket.recv(|data| {
                        let n = data.len();
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
                                    let line = String::from_utf8_lossy(&data[i..j]); // TODO: check i == j
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
                                    syscall::write(1, &data[i..n]);
                                    break;
                                }
                            }
                        }
                        (data.len(), ())
                    }).unwrap();
                    SessionState::Response
                }
                SessionState::Response if !socket.may_recv() => {
                    break;
                }
                _ => session_state
            };

            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
        iface.remove_socket(tcp_handle);
        Ok(())
    } else {
        Err(ExitCode::Failure)
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} http {}<options> <url>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-v{1}, {0}--verbose{1}    Increase verbosity", csi_option, csi_reset);
    Ok(())
}
