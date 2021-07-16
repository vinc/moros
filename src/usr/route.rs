use crate::{sys, usr, print};
use alloc::string::ToString;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        print!("{:<19} {}\n", "Destination", "Gateway");
        iface.routes_mut().update(|storage| {
            for (cidr, route) in storage.iter() {
                print!("{:<19} {}\n", cidr.to_string(), route.via_router);
            }
        });
        usr::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not find network interface\n");
        usr::shell::ExitCode::CommandError
    }
}
