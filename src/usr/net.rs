use crate::{sys, usr, debug};
use crate::api::syscall;
use crate::api::console::Style;
use alloc::vec;
use alloc::borrow::ToOwned;
use smoltcp::wire::{EthernetFrame, PrettyPrinter};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::phy::Device;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        help();
        return usr::shell::ExitCode::CommandError;
    }

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        match args[1] {
            "-h" | "--help" => {
                return help();
            }
            "config" => {
                if args.len() < 4 {
                    eprintln!("Usage: net config <key> <value>");
                    return usr::shell::ExitCode::CommandError;
                }
                match args[2] {
                    "debug" => {
                        iface.device_mut().debug_mode = match args[3] {
                            "1" | "true" => true,
                            "0" | "false" => false,
                            _ => {
                                eprintln!("Invalid config value");
                                return usr::shell::ExitCode::CommandError;
                            }
                        }
                    }
                    _ => {
                        eprintln!("Invalid config key");
                        return usr::shell::ExitCode::CommandError;
                    }
                }
            }
            "monitor" => {
                iface.device_mut().debug_mode = true;

                let mtu = iface.device().capabilities().max_transmission_unit;
                let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
                let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
                let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
                let tcp_handle = iface.add_socket(tcp_socket);

                loop {
                    if sys::console::end_of_text() {
                        println!();
                        return usr::shell::ExitCode::CommandSuccessful;
                    }
                    syscall::sleep(0.1);

                    let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
                    if let Err(e) = iface.poll(timestamp) {
                        eprintln!("Network Error: {}", e);
                    }

                    let socket = iface.get_socket::<TcpSocket>(tcp_handle);
                    if socket.may_recv() {
                        socket.recv(|buffer| {
                            let recvd_len = buffer.len();
                            let data = buffer.to_owned();
                            debug!("{}", PrettyPrinter::<EthernetFrame<&[u8]>>::new("", &buffer));
                            (recvd_len, data)
                        }).unwrap();
                    }
                }
            }
            _ => {
                eprintln!("Invalid command");
                return usr::shell::ExitCode::CommandError;
            }
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn help() -> usr::shell::ExitCode {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} net {}<command>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}config{}     Configure network", csi_option, csi_reset);
    println!("  {}monitor{}    Monitor network", csi_option, csi_reset);
    usr::shell::ExitCode::CommandSuccessful
}
