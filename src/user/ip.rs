use crate::{kernel, print, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref iface) = *kernel::net::IFACE.lock() {
        print!("Link: {}\n", iface.ethernet_addr());
        for ip_cidr in iface.ip_addrs() {
            print!("Addr: {}/{}\n", ip_cidr.address(), ip_cidr.prefix_len());
        }
        print!("RX packets: {}\n", iface.device().rx_packets_count());
        print!("TX packets: {}\n", iface.device().tx_packets_count());
        print!("RX bytes: {}\n", iface.device().rx_bytes_count());
        print!("TX bytes: {}\n", iface.device().tx_bytes_count());
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not find network interface\n");
        user::shell::ExitCode::CommandError
    }
}
