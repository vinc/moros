use core::str::{self, FromStr};
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec;
use crate::{print, kernel, user};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;

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
    if args.len() != 3 {
        print!("Usage: http <server> <path>\n");
        return user::shell::ExitCode::CommandError;
    }

    let is_verbose = true;

    if let Some(ref mut iface) = *kernel::rtl8139::IFACE.lock() {
        let url = "http://".to_owned() + args[1] + args[2];
        let url = URL::parse(&url).expect("invalid URL format");
        let address = IpAddress::from_str(&url.host).expect("invalid address format");

        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

        let mut sockets = SocketSet::new(vec![]);
        let tcp_handle = sockets.add(tcp_socket);

        enum State { Connect, Request, Response };
        let mut state = State::Connect;

        loop {
            let timestamp = Instant::from_millis((kernel::clock::clock_monotonic() * 1000.0) as i64);
            match iface.poll(&mut sockets, timestamp) {
                Ok(_) => {},
                Err(e) => {
                    print!("poll error: {}\n", e);
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
                        socket.connect((address, url.port), local_port).unwrap();
                        State::Request
                    }
                    State::Request if socket.may_send() => {
                        let http_get = "GET ".to_owned() + &url.path + " HTTP/1.1\r\n";
                        let http_host = "Host: ".to_owned() + &url.host + "\r\n";
                        let http_connection = "Connection: close\r\n".to_owned();
                        if is_verbose {
                            print!("> {}", http_get);
                            print!("> {}", http_host);
                            print!("> {}", http_connection);
                            print!(">\n");
                        }
                        socket.send_slice(http_get.as_ref()).expect("cannot send");
                        socket.send_slice(http_host.as_ref()).expect("cannot send");
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
                                }
                                if is_verbose && is_header {
                                    print!("< {}\n", line);
                                } else {
                                    print!("{}\n", line);
                                }
                            }
                            (data.len(), ())
                        }).unwrap();
                        State::Response
                    }
                    State::Response if !socket.may_recv() => {
                        break
                    }
                    _ => state
                }
            }

            if let Some(wait_duration) = iface.poll_delay(&sockets, timestamp) {
                kernel::time::sleep(wait_duration.millis() as f64 / 1000.0);
            }
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        user::shell::ExitCode::CommandError
    }
}
