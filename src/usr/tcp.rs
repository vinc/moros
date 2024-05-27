use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys::console;
use crate::sys::fs::OpenFlag;
use crate::usr;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::str;
use core::str::FromStr;
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut verbose = false;
    let args: Vec<&str> = args.iter().filter_map(|arg| match *arg {
        "-v" | "--verbose" => {
            verbose = true;
            None
        }
        _ => Some(*arg),
    }).collect();

    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }

    let (host, port) = match args[1].split_once(':') {
        Some((h, p)) => (h, p),
        None => {
            help();
            return Err(ExitCode::UsageError);
        }
    };
    let port: u16 = match port.parse() {
        Ok(n) => n,
        Err(_) => {
            eprint!("Could not parse port");
            return Err(ExitCode::UsageError);
        }
    };
    let addr = if host.ends_with(char::is_numeric) {
        IpAddress::from_str(host).expect("invalid address format")
    } else {
        match usr::host::resolve(host) {
            Ok(ip_addr) => ip_addr,
            Err(e) => {
                error!("Could not resolve host {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };

    let socket_path = "/dev/net/tcp";
    let buf_len = if let Some(info) = syscall::info(socket_path) {
        info.size() as usize
    } else {
        error!("Could not open '{}'", socket_path);
        return Err(ExitCode::Failure);
    };

    let flags = OpenFlag::Device as usize;
    if let Some(handle) = syscall::open(socket_path, flags) {
        if syscall::connect(handle, addr, port).is_err() {
            error!("Could not connect to {}:{}", addr, port);
            syscall::close(handle);
            return Err(ExitCode::Failure);
        }
        if verbose {
            debug!("Connected to {}:{}", addr, port);
        }
        loop {
            if console::end_of_text() || console::end_of_transmission() {
                eprintln!();
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }
            let mut data = vec![0; buf_len];
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
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} tcp {}<host>:<port>{1}",
        csi_title, csi_reset, csi_option
    );
}
