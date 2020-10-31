use crate::{kernel, print, user};
use core::str::FromStr;
use smoltcp::wire::IpCidr;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref mut iface) = *kernel::net::IFACE.lock() {
        if args.len() == 1 {
            print!("Link: {}\n", iface.ethernet_addr());
            for ip_cidr in iface.ip_addrs() {
                print!("Addr: {}/{}\n", ip_cidr.address(), ip_cidr.prefix_len());
            }
            print!("RX packets: {}\n", iface.device().stats.rx_packets_count());
            print!("TX packets: {}\n", iface.device().stats.tx_packets_count());
            print!("RX bytes: {}\n", iface.device().stats.rx_bytes_count());
            print!("TX bytes: {}\n", iface.device().stats.tx_bytes_count());
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
                                addrs.iter_mut().nth(0).map(|addr| {
                                    *addr = cidr;
                                });
                            });
                        },
                    }
                },
                _ => {
                    return error("invalid command");
                }
            }
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        error("could not find network interface")
    }
}

fn error(message: &str) -> user::shell::ExitCode {
    print!("Error: {}\n", message);
    user::shell::ExitCode::CommandError
}
