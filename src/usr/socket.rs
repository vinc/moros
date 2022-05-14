use crate::{sys, usr, debug};
use crate::api::console::Style;
use crate::api::io;
use crate::api::syscall;
use crate::api::random;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::str::{self, FromStr};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer, TcpState};
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let mut prompt = false;
    let mut verbose = false;
    let mut read_only = false;
    let mut interval = 0.0;
    let mut next_arg_is_interval = false;
    let mut args: Vec<&str> = args.iter().filter_map(|arg| {
        match *arg {
            "-p" | "--prompt" => {
                prompt = true;
                None
            }
            "-r" | "--read-only" => {
                read_only = true;
                None
            }
            "-v" | "--verbose" => {
                verbose = true;
                None
            }
            "-i" | "--interval" => {
                next_arg_is_interval = true;
                None
            }
            _ if next_arg_is_interval => {
                next_arg_is_interval = false;
                if let Ok(i) = arg.parse() {
                    interval = i;
                }
                None
            }
            _ => {
                Some(*arg)
            }
        }
    }).collect();
    if prompt {
        println!("MOROS Socket v0.1.0\n");
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
                    if !read_only {
                        if verbose {
                            debug!("Sending ...");
                        }
                        if prompt {
                            // Print prompt
                            print!("{}>{} ", Style::color("Cyan"), Style::reset());
                        }
                        let line = io::stdin().read_line();
                        if line.is_empty() {
                            socket.close();
                        } else {
                            let line = line.replace("\n", "\r\n");
                            socket.send_slice(line.as_ref()).expect("cannot send");
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
                        print!("{}", contents.replace("\r\n", "\n"));
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

            if interval > 0.0 {
                syscall::sleep(interval);
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
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-v{1}, {0}--verbose{1}    Increase verbosity", csi_option, csi_reset);
    println!("  {0}-p{1}, {0}--prompt{1}     Display prompt", csi_option, csi_reset);
    println!("  {0}-r{1}, {0}--read-only{1}  One way connexion to a server", csi_option, csi_reset);
    usr::shell::ExitCode::CommandSuccessful
}
