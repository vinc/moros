use alloc::string::ToString;
use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref mut iface) = *kernel::rtl8139::IFACE.lock() {
        print!("{:<19} {}\n", "Destination", "Gateway");
        iface.routes_mut().update(|storage| {
            for (cidr, route) in storage.iter() {
                print!("{:<19} {}\n", cidr.to_string(), route.via_router);
            }
        });
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not find network interface\n");
        user::shell::ExitCode::CommandError
    }
}
