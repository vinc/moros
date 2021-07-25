use crate::{sys, usr};
use alloc::string::ToString;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        println!("{:<19} {}", "Destination", "Gateway");
        iface.routes_mut().update(|storage| {
            for (cidr, route) in storage.iter() {
                println!("{:<19} {}", cidr.to_string(), route.via_router);
            }
        });
        usr::shell::ExitCode::CommandSuccessful
    } else {
        println!("Could not find network interface");
        usr::shell::ExitCode::CommandError
    }
}
