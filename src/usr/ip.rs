use crate::{sys, usr};
use crate::api::console::Style;

use core::str::FromStr;
use smoltcp::wire::IpCidr;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("LightCyan");
    let csi_reset = Style::reset();
    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        if args.len() == 1 {
            println!("{}Link:{} {}", csi_color, csi_reset, iface.hardware_addr());
            for ip_cidr in iface.ip_addrs() {
                println!("{}Addr:{} {}/{}", csi_color, csi_reset, ip_cidr.address(), ip_cidr.prefix_len());
            }
            let stats = iface.device().stats.clone();
            println!("{}RX:{}   {} packets ({} bytes)", csi_color, csi_reset, stats.rx_packets_count(), stats.rx_bytes_count());
            println!("{}TX:{}   {} packets ({} bytes)", csi_color, csi_reset, stats.tx_packets_count(), stats.tx_bytes_count());
        } else {
            match args[1] {
                "set" => {
                    if args.len() == 2 {
                        return error("address missing");
                    }
                    match IpCidr::from_str(args[2]) {
                        Err(_) => {
                            return error("could not parse address");
                        },
                        Ok(cidr) => {
                            iface.update_ip_addrs(|addrs| {
                                if let Some(addr) = addrs.iter_mut().next() {
                                    *addr = cidr;
                                }
                            });
                        },
                    }
                },
                _ => {
                    return error("invalid command");
                }
            }
        }
        usr::shell::ExitCode::CommandSuccessful
    } else {
        error("could not find network interface")
    }
}

fn error(message: &str) -> usr::shell::ExitCode {
    eprintln!("Error: {}", message);
    usr::shell::ExitCode::CommandError
}
