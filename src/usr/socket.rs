use crate::{sys, usr, debug};
use crate::api::console::Style;
use crate::api::io;
use crate::api::fs::IO;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys::fs::OpenFlag;
use crate::sys::net::SocketStatus;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use bit_field::BitField;
use core::str::{self, FromStr};
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut listen = false;
    let mut verbose = false;
    let mut read_only = false;
    let args: Vec<&str> = args.iter().filter_map(|arg| {
        match *arg {
            "-l" | "--listen" => {
                listen = true;
                None
            }
            "-r" | "--read" => {
                read_only = true;
                None
            }
            "-v" | "--verbose" => {
                verbose = true;
                None
            }
            _ => {
                Some(*arg)
            }
        }
    }).collect();

    if verbose {
        println!("MOROS Socket v0.2.0\n");
    }

    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    let (host, port) = match args[1].split_once(':') {
        Some((h, p)) => (h, p),
        None => ("0.0.0.0", args[1]),
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
            Ok(ip_addr) => {
                ip_addr
            }
            Err(e) => {
                error!("Could not resolve host: {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };

    let stdin = 0;
    let stdout = 1;
    let flags = OpenFlag::Device as usize;
    if let Some(handle) = syscall::open("/dev/net/tcp", flags) {
        if listen {
            if syscall::listen(handle, port).is_err() {
                error!("Could not listen to {}:{}", addr, port);
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }

        } else {
            if syscall::connect(handle, addr, port).is_err() {
                error!("Could not connect to {}:{}", addr, port);
                syscall::close(handle);
                return Err(ExitCode::Failure);
            }
        }

        loop {
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                println!();
                break;
            }

            let list = vec![(stdin, IO::Read), (handle, IO::Read)];
            if let Some((h, _)) = syscall::poll(&list) {
                if h == stdin {
                    let line = io::stdin().read_line().replace("\n", "\r\n");
                    syscall::write(handle, &line.as_bytes());
                } else {
                    let mut data = vec![0; 2048];
                    if let Some(bytes) = syscall::read(handle, &mut data) {
                        data.resize(bytes, 0);
                        syscall::write(stdout, &data);
                    }
                }
            } else {
                syscall::sleep(0.01);
                let mut data = vec![0; 1]; // 1 byte status read
                match syscall::read(handle, &mut data) {
                    Some(1) if !data[0].get_bit(SocketStatus::MayRecv as usize) => {
                        break; // recv closed
                    }
                    _ => continue,
                }
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
    println!("{}Usage:{} socket {}[<host>:]<port>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-l{1}, {0}--listen{1}    Listen to a local port", csi_option, csi_reset);
    println!("  {0}-v{1}, {0}--verbose{1}   Increase verbosity", csi_option, csi_reset);
    println!("  {0}-r{1}, {0}--read{1}      Read only connexion", csi_option, csi_reset);
}
