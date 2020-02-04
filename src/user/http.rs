use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use core::str::{self, FromStr};
use core::time::Duration;
use crate::{print, kernel, user};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
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
            None => (server, "80"),
        };
        Some(Self {
            host: host.into(),
            port: port.parse().unwrap_or(80),
            path: path.into(),
        })
    }
}

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    let mut is_verbose = false;
    let args: Vec<_> = args.into_iter().filter(|arg| {
        let arg = arg.to_string();
        if arg == "--verbose" {
            is_verbose = true;
        }
        !arg.starts_with("--")
    }).collect();
    if args.len() != 3 {
        print!("Usage: http <server> <path>\n");
        return user::shell::ExitCode::CommandError;
    }

    let url = "http://".to_owned() + args[1] + args[2];
    let url = URL::parse(&url).expect("invalid URL format");

    let address = if url.host.ends_with(char::is_numeric) {
        IpAddress::from_str(&url.host).expect("invalid address format")
    } else {
        match user::host::resolve(&url.host) {
            Ok(ip_addr) => {
                ip_addr
            }
            Err(e) => {
                print!("Could not resolve host: {:?}\n", e);
                return user::shell::ExitCode::CommandError;
            }
        }
    };

    let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

    let mut sockets = SocketSet::new(vec![]);
    let tcp_handle = sockets.add(tcp_socket);

    enum State { Connect, Request, Response };
    let mut state = State::Connect;

    if let Some(ref mut iface) = *kernel::rtl8139::IFACE.lock() {
        match iface.ipv4_addr() {
            None => {
                print!("Interface not ready\n");
                return user::shell::ExitCode::CommandError;
            }
            Some(ip_addr) if ip_addr.is_unspecified() => {
                print!("Interface not ready\n");
                return user::shell::ExitCode::CommandError;
            }
            _ => {}
        }

        let timeout = 5.0;
        let time = kernel::clock::uptime();
        loop {
            if kernel::clock::uptime() - time > timeout {
                print!("Timeout reached\n");
                return user::shell::ExitCode::CommandError;
            }

            let timestamp = Instant::from_millis((kernel::clock::realtime() * 1000.0) as i64);
            match iface.poll(&mut sockets, timestamp) {
                Ok(_) => {},
                Err(e) => {
                    print!("Interface polling error: {}\n", e);
                    return user::shell::ExitCode::CommandError;
                }
            }

            {
                let mut socket = sockets.get::<TcpSocket>(tcp_handle);

                state = match state {
                    State::Connect if !socket.is_active() => {
                        let local_port = 49152 + kernel::random::rand16().expect("random port") % 16384;
                        if is_verbose {
                            print!("* Connecting to {}:{}\n", address, url.port);
                        }
                        if socket.connect((address, url.port), local_port).is_err() {
                            print!("Could not connect to {}:{}\n", address, url.port);
                            return user::shell::ExitCode::CommandError;
                        }
                        State::Request
                    }
                    State::Request if socket.may_send() => {
                        let http_get = "GET ".to_owned() + &url.path + " HTTP/1.1\r\n";
                        let http_host = "Host: ".to_owned() + &url.host + "\r\n";
                        let http_ua = "User-Agent: MOROS/".to_owned() + env!("CARGO_PKG_VERSION") + "\r\n";
                        let http_connection = "Connection: close\r\n".to_owned();
                        if is_verbose {
                            print!("> {}", http_get);
                            print!("> {}", http_host);
                            print!("> {}", http_ua);
                            print!("> {}", http_connection);
                            print!(">\n");
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
                            let contents = str::from_utf8(data).unwrap_or("invalid UTF-8");
                            let mut is_header = true;
                            for line in contents.lines() {
                                if line.len() == 0 {
                                    is_header = false;
                                    if !is_verbose {
                                        continue
                                    }
                                }
                                if is_header {
                                    if is_verbose {
                                        print!("< {}\n", line);
                                    }
                                } else {
                                    print!("{}\n", line);
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
                }
            }

            if let Some(wait_duration) = iface.poll_delay(&sockets, timestamp) {
                let wait_duration: Duration = wait_duration.into();
                kernel::time::sleep(libm::fmin(wait_duration.as_secs_f64(), timeout));
            }
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        user::shell::ExitCode::CommandError
    }
}
