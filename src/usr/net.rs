use crate::api::clock;
use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys;
use crate::sys::console;
use crate::sys::net;
use crate::sys::net::EthernetDeviceIO;
use alloc::format;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use core::str::FromStr;
use smoltcp::iface::SocketSet;
use smoltcp::phy::Device;
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetFrame, IpCidr, Ipv4Address, PrettyPrinter};

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    match *args.get(1).unwrap_or(&"") {
        "-h" | "--help" => {
            help();
            return Ok(());
        }
        "c" | "config" => {
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
        "s" | "stat" => {
            stat();
        }
        "m" | "monitor" => {
            monitor();
        }
        _ => {
            error!("Invalid command");
            return Err(ExitCode::Failure);
        }
    }
    Ok(())
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} net {}<command>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Commands:{}", csi_title, csi_reset);
    println!("  {}config{}   Configure network", csi_option, csi_reset);
    println!("  {}monitor{}  Monitor network", csi_option, csi_reset);
    println!(
        "  {}stat{}     Display network status",
        csi_option, csi_reset
    );
}

fn help_config() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} net config {}<attribute> <value>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    println!();
    println!("{}Attributes:{}", csi_title, csi_reset);
    println!("  {}mac{}  MAC Address", csi_option, csi_reset);
    println!("  {}ip{}   IP Address", csi_option, csi_reset);
    println!("  {}gw{}   Gateway Address", csi_option, csi_reset);
    println!("  {}dns{}  Domain Name Servers", csi_option, csi_reset);
}

fn print_config(attribute: &str) {
    let csi_color = Style::color("aqua");
    let csi_reset = Style::reset();
    if let Some(value) = get_config(attribute) {
        let width = 4 - attribute.len();
        println!(
            "{}{}:{}{:width$}{}",
            csi_color,
            attribute,
            csi_reset,
            "",
            value,
            width = width
        );
    }
}

const DNS_FILE: &str = "/ini/dns";

fn dns_config() -> Option<String> {
    warning!("This command is deprecated, use /dev/net/dns instead");
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

fn gw_config() -> Option<String> {
    warning!("This command is deprecated, use /dev/net/gw instead");
    let mut res = None;
    if let Some((ref mut iface, _)) = *sys::net::NET.lock() {
        iface.routes_mut().update(|storage| {
            if let Some(route) = storage.iter().next() {
                res = Some(route.via_router.to_string());
            }
        });
    } else {
        error!("Network error");
    }
    res
}

fn ip_config() -> Option<String> {
    warning!("This command is deprecated, use /dev/net/ip instead");
    if let Some((ref mut iface, _)) = *sys::net::NET.lock() {
        if let Some(ip_cidr) = iface.ip_addrs().iter().next() {
            return Some(format!(
                "{}/{}", ip_cidr.address(), ip_cidr.prefix_len()
            ));
        }
    } else {
        error!("Network error");
    }
    None
}

fn mac_config() -> Option<String> {
    warning!("This command is deprecated, use /dev/net/mac instead");
    if let Some((ref mut iface, _)) = *sys::net::NET.lock() {
        return Some(iface.hardware_addr().to_string());
    } else {
        error!("Network error");
    }
    None
}

pub fn get_config(attribute: &str) -> Option<String> {
    match attribute {
        "dns" => dns_config(),
        "gw" => gw_config(),
        "ip" => ip_config(),
        "mac" => mac_config(),
        _ => {
            error!("Invalid config attribute");
            None
        }
    }
}

pub fn set_config(attribute: &str, value: &str) {
    match attribute {
        "debug" => {
            if let Some((_, ref mut device)) = *sys::net::NET.lock() {
                match value {
                    "1" | "true" => device.config().enable_debug(),
                    "0" | "false" => device.config().disable_debug(),
                    _ => error!("Invalid config value"),
                }
            } else {
                error!("Network error");
            }
        }
        "ip" => {
            warning!("This command is deprecated, use /dev/net/ip instead");
            if let Ok(addr) = IpCidr::from_str(value) {
                if let Some((ref mut iface, _)) = *sys::net::NET.lock() {
                    iface.update_ip_addrs(|addrs| {
                        addrs.clear();
                        addrs.push(addr).unwrap();
                        log!("NET IP {}", value);
                    });
                } else {
                    error!("Network error");
                }
            } else {
                error!("Could not parse address");
            }
        }
        "gw" => {
            warning!("This command is deprecated, use /dev/net/gw instead");
            if let Some((ref mut iface, _)) = *sys::net::NET.lock() {
                if value == "0.0.0.0" {
                    iface.routes_mut().remove_default_ipv4_route();
                } else if let Ok(ip) = Ipv4Address::from_str(value) {
                    iface.routes_mut().add_default_ipv4_route(ip).unwrap();
                    log!("NET GW {}", value);
                } else {
                    error!("Could not parse address");
                }
            } else {
                error!("Network error");
            }
        }
        "dns" => {
            warning!("This command is deprecated, use /ini/dns instead");
            let servers = value.trim();
            if servers.split(',').all(|s| Ipv4Address::from_str(s).is_ok()) {
                let s = format!("{}\n", servers);
                if fs::write(DNS_FILE, s.as_bytes()).is_ok() {
                    log!("NET DNS {}", servers);
                } else {
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

pub fn stat() {
    warning!("This command is deprecated, use /dev/net/usage instead");
    if let Some((_, ref mut device)) = *sys::net::NET.lock() {
        let stats = device.stats();
        let csi_color = Style::color("aqua");
        let csi_reset = Style::reset();
        println!(
            "{}rx:{} {} packets ({} bytes)",
            csi_color,
            csi_reset,
            stats.rx_packets_count(),
            stats.rx_bytes_count()
        );
        println!(
            "{}tx:{} {} packets ({} bytes)",
            csi_color,
            csi_reset,
            stats.tx_packets_count(),
            stats.tx_bytes_count()
        );
    } else {
        error!("Network error");
    }
}

fn monitor() {
    if let Some((ref mut iface, ref mut device)) = *net::NET.lock() {
        device.config().enable_debug();

        let mtu = device.capabilities().max_transmission_unit;
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; mtu]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; mtu]);
        let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
        let mut sockets = SocketSet::new(vec![]);
        let tcp_handle = sockets.add(tcp_socket);

        loop {
            if console::end_of_text() || console::end_of_transmission() {
                println!();
                return;
            }
            syscall::sleep(0.1);

            let ms = (clock::epoch_time() * 1000000.0) as i64;
            let time = Instant::from_micros(ms);
            iface.poll(time, device, &mut sockets);
            let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
            if socket.may_recv() {
                socket.recv(|buffer| {
                    let recvd_len = buffer.len();
                    let data = buffer.to_owned();
                    let pp = PrettyPrinter::<EthernetFrame<&[u8]>>::new(
                        "", &buffer
                    );
                    debug!("{}", pp);
                    (recvd_len, data)
                }).unwrap();
            }
        }
    } else {
        error!("Network error");
    }
}
