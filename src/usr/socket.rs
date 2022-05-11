use crate::{sys, usr, debug};
use crate::api::console::Style;
use crate::api::io;
use crate::api::syscall;
use crate::api::random;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::str::{self, FromStr};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer, TcpState};
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let mut verbose = false;
    let mut interactive = false;
    let mut args: Vec<&str> = args.iter().filter_map(|arg| {
        match *arg {
            "-v" | "--verbose" => {
                verbose = true;
                None
            }
            "-i" | "--interactive" => {
                interactive = true;
                None
            }
            _ => {
                Some(*arg)
            }
        }
    }).collect();
    if interactive {
        println!("MOROS Socket v0.1.0");
    }

    // Split <host> and <port>
    if args.len() == 2 {
        if let Some(i) = args[1].find(':') {
            let arg = args[1].clone();
            let (host, path) = arg.split_at(i);
            args[1] = host;
            args.push(&path[1..]);
        }
    }

    if args.len() != 3 {
        help();
        return usr::shell::ExitCode::CommandError;
    }

    let host = &args[1];
    let port: u16 = args[2].parse().expect("Could not parse port");

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

    #[derive(Debug)]
    enum State { Connecting, Sending, Receiving }
    let mut state = State::Connecting;

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        let tcp_handle = iface.add_socket(tcp_socket);

        loop {
            if sys::console::end_of_text() {
                eprintln!();
                iface.remove_socket(tcp_handle);
                return usr::shell::ExitCode::CommandError;
            }
            let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let (socket, cx) = iface.get_socket_and_context::<TcpSocket>(tcp_handle);
            if verbose {
                debug!("*********************************");
                debug!("APP State: {:?}", state);
                debug!("TCP State: {:?}", socket.state());
                debug!("is active: {}", socket.is_active());
                debug!("is open: {}", socket.is_open());
                debug!("can recv: {}", socket.can_recv());
                debug!("can send: {}", socket.can_send());
                debug!("may recv: {}", socket.may_recv());
                debug!("may send: {}", socket.may_send());
            }

            state = match state {
                State::Connecting if !socket.is_active() => {
                    let local_port = 49152 + random::get_u16() % 16384;
                    if verbose {
                        debug!("Connecting to {}:{}", address, port);
                    }
                    if socket.connect(cx, (address, port), local_port).is_err() {
                        error!("Could not connect to {}:{}", address, port);
                        return usr::shell::ExitCode::CommandError;
                    }
                    State::Receiving
                }
                State::Sending if socket.can_recv() => {
                    if verbose {
                        debug!("Sending -> Receiving");
                    }
                    State::Receiving
                }
                State::Sending if socket.can_send() && socket.may_recv() => {
                    if interactive {
                        if verbose {
                            debug!("Sending ...");
                        }

                        // Print prompt
                        print!("{}>{} ", Style::color("Cyan"), Style::reset());

                        let data = io::stdin().read_line().trim_end().to_string();
                        socket.send_slice(data.as_ref()).expect("cannot send");
                        socket.send_slice(b"\r\n").expect("cannot send");
                        if verbose {
                            debug!("Sent '{}\\r\\n'", data);
                        }
                    }
                    State::Receiving
                }
                State::Receiving if socket.can_recv() => {
                    if verbose {
                        debug!("Receiving ...");
                    }
                    socket.recv(|data| {
                        let contents = String::from_utf8_lossy(data);
                        for line in contents.lines() {
                            println!("{}", line);
                        }
                        (data.len(), ())
                    }).unwrap();
                    State::Receiving
                }
                _ if socket.state() == TcpState::SynSent || socket.state() == TcpState::SynReceived => {
                    state
                }
                State::Receiving if !socket.may_recv() => {
                    if verbose {
                        debug!("Break from response");
                    }
                    break;
                }
                State::Receiving if socket.can_send() => {
                    if verbose {
                        debug!("Receiving -> Sending");
                    }
                    State::Sending
                }
                _ if !socket.is_active() => {
                    if verbose {
                        debug!("Break from inactive");
                    }
                    break;
                }
                _ => state
            };

            if interactive {
                syscall::sleep(0.1);
            }
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

fn help() -> usr::shell::ExitCode {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} socket {}<host> <port>{1}", csi_title, csi_reset, csi_option);
    // TODO: Add `-i` and `-v` options
    usr::shell::ExitCode::CommandSuccessful
}
