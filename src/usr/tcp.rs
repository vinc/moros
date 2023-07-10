use crate::{sys, usr};
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys::fs::OpenFlag;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::str;
use core::str::FromStr;
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut verbose = false;
    let mut args: Vec<&str> = args.iter().filter_map(|arg| {
        match *arg {
            "-v" | "--verbose" => {
                verbose = true;
                None
            }
            _ => {
                Some(*arg)
            }
        }
    }).collect();

    // Split <host> and <port>
    if args.len() == 2 {
        if let Some(i) = args[1].find(':') {
            let (host, path) = args[1].split_at(i);
            args[1] = host;
            args.push(&path[1..]);
        }
    }

    if args.len() != 3 {
        help();
        return Err(ExitCode::UsageError);
    }

    let host = &args[1];
    let port: u16 = args[2].parse().expect("Could not parse port");
    let addr = if host.ends_with(char::is_numeric) {
        IpAddress::from_str(host).expect("invalid address format")
    } else {
        match usr::host::resolve(host) {
            Ok(ip_addr) => {
                ip_addr
            }
            Err(e) => {
                error!("Could not resolve host {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };

    let flags = OpenFlag::Device as usize;
    if let Some(handle) = syscall::open("/dev/net/tcp", flags) {
        if syscall::connect(handle, addr, port).is_err() {
            error!("Could not connect to {}:{}", addr, port);
            syscall::close(handle);
            return Err(ExitCode::Failure);
        }
        loop {
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                eprintln!();
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }
            let mut data = vec![0; 2048];
            if let Some(bytes) = syscall::read(handle, &mut data) {
                if bytes == 0 {
                    break;
                }
                data.resize(bytes, 0);
                syscall::write(1, &data);
            } else {
                error!("Could not read from {}:{}", addr, port);
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }
        }
        syscall::close(handle);
        Ok(())
    } else {
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} tcp {}<host> <port>{1}", csi_title, csi_reset, csi_option);
}
