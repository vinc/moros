use crate::{kernel, print, user};
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::str::{self, FromStr};
use core::time::Duration;
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    let mut args: Vec<String> = args.iter().map(ToOwned::to_owned).map(ToOwned::to_owned).collect();

    // Split <host> and <port>
    if args.len() == 2 {
        if let Some(i) = args[1].find(':') {
            let arg = args[1].clone();
            let (host, path) = arg.split_at(i);
            args[1] = host.to_string();
            args.push(path[1..].to_string());
        }
    }

    if args.len() != 3 {
        print!("Usage: tcp <host> <port>\n");
        return user::shell::ExitCode::CommandError;
    }

    let host = &args[1];
    let port: u16 = args[2].parse().expect("Could not parse port");
    let timeout = 5.0;
    let request = "";

    let address = if host.ends_with(char::is_numeric) {
        IpAddress::from_str(&host).expect("invalid address format")
    } else {
        match user::host::resolve(&host) {
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

    if let Some(ref mut iface) = *kernel::net::rtl8139::IFACE.lock() {
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

        let time = kernel::clock::uptime();
        loop {
            if kernel::clock::uptime() - time > timeout {
                print!("Timeout reached\n");
                return user::shell::ExitCode::CommandError;
            }

            let timestamp = Instant::from_millis((kernel::clock::realtime() * 1000.0) as i64);
            match iface.poll(&mut sockets, timestamp) {
                Err(smoltcp::Error::Unrecognized) => {}
                Err(e) => {
                    print!("Network Error: {}\n", e);
                }
                Ok(_) => {}
            }

            {
                let mut socket = sockets.get::<TcpSocket>(tcp_handle);

                state = match state {
                    State::Connect if !socket.is_active() => {
                        let local_port = 49152 + kernel::random::rand16().expect("random port") % 16384;
                        print!("Connecting to {}:{}\n", address, port);
                        if socket.connect((address, port), local_port).is_err() {
                            print!("Could not connect to {}:{}\n", address, port);
                            return user::shell::ExitCode::CommandError;
                        }
                        State::Request
                    }
                    State::Request if socket.may_send() => {
                        if request.len() > 0 {
                            socket.send_slice(request.as_ref()).expect("cannot send");
                        }
                        State::Response
                    }
                    State::Response if socket.can_recv() => {
                        socket.recv(|data| {
                            let contents = String::from_utf8_lossy(data);
                            for line in contents.lines() {
                                print!("{}\n", line);
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
