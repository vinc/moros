use crate::sys;
use alloc::format;
use crate::api::clock;
use crate::api::console::Style;
use crate::api::fs;
use crate::api::syscall;
use crate::api::process;
use crate::sys::net::EthernetDeviceIO;

use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec;
use core::str::FromStr;
use smoltcp::wire::{EthernetFrame, PrettyPrinter, IpCidr, Ipv4Address};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::phy::Device;

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() == 1 {
        help();
        return Err(process::EXIT_FAILURE);
    }

    match args[1] {
        "-h" | "--help" => {
            help();
            return Ok(());
        }
        "config" => {
            if args.len() < 3 {
                print_config("mac");
                print_config("ip");
                print_config("gw");
                print_config("dns");
            } else if args[2] == "-h" || args[2] == "--help" {
                help_config();
                return Ok(());
            } else if args.len() < 4 {
                print_config(args[2]);
            } else {
                set_config(args[2], args[3]);
            }
        }
        "stat" => {
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                let stats = iface.device().stats();
                let csi_color = Style::color("LightCyan");
                let csi_reset = Style::reset();
                println!("{}rx:{} {} packets ({} bytes)", csi_color, csi_reset, stats.rx_packets_count(), stats.rx_bytes_count());
                println!("{}tx:{} {} packets ({} bytes)", csi_color, csi_reset, stats.tx_packets_count(), stats.tx_bytes_count());
            } else {
                error!("Network error");
            }
        }
        "monitor" => {
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                iface.device_mut().config().enable_debug();

                let mtu = iface.device().capabilities().max_transmission_unit;
                let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
                let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
                let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
                let tcp_handle = iface.add_socket(tcp_socket);

                loop {
                    if sys::console::end_of_text() || sys::console::end_of_transmission() {
                        println!();
                        iface.remove_socket(tcp_handle);
                        return Ok(());
                    }
                    syscall::sleep(0.1);

                    let timestamp = Instant::from_micros((clock::realtime() * 1000000.0) as i64);
                    if let Err(e) = iface.poll(timestamp) {
                        error!("Network Error: {}", e);
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
            } else {
                error!("Network error");
            }
        }
        _ => {
            error!("Invalid command");
            return Err(process::EXIT_FAILURE);
        }
    }
    Ok(())
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} net {}<command>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}config{}   Configure network", csi_option, csi_reset);
    println!("  {}monitor{}  Monitor network", csi_option, csi_reset);
    println!("  {}stat{}     Display network status", csi_option, csi_reset);
}

fn help_config() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} net config {}<attribute> <value>{}", csi_title, csi_reset, csi_option, csi_reset);
    println!();
    println!("{}Attributes:{}", csi_title, csi_reset);
    println!("  {}mac{}  MAC Address", csi_option, csi_reset);
    println!("  {}ip{}   IP Address", csi_option, csi_reset);
    println!("  {}gw{}   Gateway Address", csi_option, csi_reset);
    println!("  {}dns{}  Domain Name Servers", csi_option, csi_reset);
}

fn print_config(attribute: &str) {
    let csi_color = Style::color("LightCyan");
    let csi_reset = Style::reset();
    if let Some(value) = get_config(attribute) {
        let width = 4 - attribute.len();
        println!("{}{}:{}{:width$}{}", csi_color, attribute, csi_reset, "", value, width = width);
    }
}

const DNS_FILE: &str = "/ini/dns";

pub fn get_config(attribute: &str) -> Option<String> {
    match attribute {
        "mac" => {
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                return Some(iface.hardware_addr().to_string());
            } else {
                error!("Network error");
            }
            None
        }
        "ip" => {
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                if let Some(ip_cidr) = iface.ip_addrs().iter().next() {
                    return Some(format!("{}/{}", ip_cidr.address(), ip_cidr.prefix_len()));
                }
            } else {
                error!("Network error");
            }
            None
        }
        "gw" => {
            let mut res = None;
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                iface.routes_mut().update(|storage| {
                    if let Some((_, route)) = storage.iter().next() {
                        res = Some(route.via_router.to_string());
                    }
                });
            } else {
                error!("Network error");
            }
            res
        }
        "dns" => {
            if let Ok(value) = fs::read_to_string(DNS_FILE) {
                let servers = value.trim();
                if servers.split(',').all(|s| Ipv4Address::from_str(s).is_ok()) {
                    Some(servers.to_string())
                } else {
                    error!("Could not parse '{}'", servers);
                    None
                }
            } else {
                error!("Could not read '{}'", DNS_FILE);
                None
            }
        }
        _ => {
            error!("Invalid config attribute");
            None
        }
    }
}

pub fn set_config(attribute: &str, value: &str) {
    match attribute {
        "debug" => {
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                match value {
                    "1" | "true" => iface.device_mut().config().enable_debug(),
                    "0" | "false" => iface.device_mut().config().disable_debug(),
                    _ => error!("Invalid config value"),
                }
            } else {
                error!("Network error");
            }
        }
        "ip" => {
            if let Ok(ip) = IpCidr::from_str(value) {
                if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                    iface.update_ip_addrs(|addrs| {
                        if let Some(addr) = addrs.iter_mut().next() {
                            *addr = ip;
                        }
                    });
                } else {
                    error!("Network error");
                }
            } else {
                error!("Could not parse address");
            }
        }
        "gw" => {
            if let Some(ref mut iface) = *sys::net::IFACE.lock() {
                if value == "0.0.0.0" {
                    iface.routes_mut().remove_default_ipv4_route();
                } else if let Ok(ip) = Ipv4Address::from_str(value) {
                    iface.routes_mut().add_default_ipv4_route(ip).unwrap();
                } else {
                    error!("Could not parse address");
                }
            } else {
                error!("Network error");
            }
        }
        "dns" => {
            let servers = value.trim();
            if servers.split(',').all(|s| Ipv4Address::from_str(s).is_ok()) {
                if fs::write(DNS_FILE, format!("{}\n", servers).as_bytes()).is_err() {
                    error!("Could not write to '{}'", DNS_FILE);
                }
            } else {
                error!("Could not parse '{}'", servers);
            }
        }
        _ => {
            error!("Invalid config key");
        }
    }
}
