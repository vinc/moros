use crate::{sys, usr};
use crate::api::console::Style;
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

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
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
        help();
        return usr::shell::ExitCode::CommandError;
    }

    let host = &args[1];
    let port: u16 = args[2].parse().expect("Could not parse port");
    let request = "";

    let address = if host.ends_with(char::is_numeric) {
        IpAddress::from_str(host).expect("invalid address format")
    } else {
        match usr::host::resolve(host) {
            Ok(ip_addr) => {
                ip_addr
            }
            Err(e) => {
                error!("Could not resolve host: {:?}", e);
                return usr::shell::ExitCode::CommandError;
            }
        }
    };

    let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

    enum State { Connect, Request, Response }
    let mut state = State::Connect;

    if let Some(ref mut interface) = *sys::net::INTERFACE.lock() {
        let tcp_handle = interface.iface.add_socket(tcp_socket);

        let timeout = 5.0;
        let started = syscall::realtime();
        loop {
            if syscall::realtime() - started > timeout {
                error!("Timeout reached");
                interface.iface.remove_socket(tcp_handle);
                return usr::shell::ExitCode::CommandError;
            }
            if sys::console::end_of_text() {
                eprintln!();
                interface.iface.remove_socket(tcp_handle);
                return usr::shell::ExitCode::CommandError;
            }
            let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
            if let Err(e) = interface.iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let (socket, cx) = interface.iface.get_socket_and_context::<TcpSocket>(tcp_handle);

            state = match state {
                State::Connect if !socket.is_active() => {
                    let local_port = 49152 + random::get_u16() % 16384;
                    println!("Connecting to {}:{}", address, port);
                    if socket.connect(cx, (address, port), local_port).is_err() {
                        error!("Could not connect to {}:{}", address, port);
                        return usr::shell::ExitCode::CommandError;
                    }
                    State::Request
                }
                State::Request if socket.may_send() => {
                    if !request.is_empty() {
                        socket.send_slice(request.as_ref()).expect("cannot send");
                    }
                    State::Response
                }
                State::Response if socket.can_recv() => {
                    socket.recv(|data| {
                        let contents = String::from_utf8_lossy(data);
                        for line in contents.lines() {
                            println!("{}", line);
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

            if let Some(wait_duration) = interface.iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
        interface.iface.remove_socket(tcp_handle);
        usr::shell::ExitCode::CommandSuccessful
    } else {
        usr::shell::ExitCode::CommandError
    }
}

fn help() -> usr::shell::ExitCode {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} tcp {}<host> <port>{1}", csi_title, csi_reset, csi_option);
    usr::shell::ExitCode::CommandSuccessful
}
