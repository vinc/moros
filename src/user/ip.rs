use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref iface) = *kernel::rtl8139::IFACE.lock() {
        print!("Link: {}\n", iface.ethernet_addr());
        for ip_cidr in iface.ip_addrs() {
            print!("Addr: {}/{}\n", ip_cidr.address(), ip_cidr.prefix_len());
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not find network interface\n");
        user::shell::ExitCode::CommandError
    }
}
