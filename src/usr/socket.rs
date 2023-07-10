use crate::{sys, usr, debug};
use crate::api::console::Style;
use crate::api::io;
use crate::api::fs::IO;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::sys::fs::OpenFlag;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::str::{self, FromStr};
use smoltcp::wire::IpAddress;

fn print_prompt() {
    print!("{}>{} ", Style::color("Cyan"), Style::reset());
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut listen = false;
    let mut prompt = false;
    let mut verbose = false;
    let mut read_only = false;
    let mut args: Vec<&str> = args.iter().filter_map(|arg| {
        match *arg {
            "-l" | "--listen" => {
                listen = true;
                None
            }
            "-p" | "--prompt" => {
                prompt = true;
                None
            }
            "-r" | "--read-only" => {
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
    if prompt {
        println!("MOROS Socket v0.2.0\n");
    }

    let required_args_count = if listen { 2 } else { 3 };

    if args.len() == required_args_count - 1 {
        if let Some(i) = args[1].find(':') { // Split <host> and <port>
            let (host, path) = args[1].split_at(i);
            args[1] = host;
            args.push(&path[1..]);
        }
    }

    if args.len() != required_args_count {
        help();
        return Err(ExitCode::UsageError);
    }

    let host = if listen { "0.0.0.0" } else { args[1] };
    let port: u16 = args[required_args_count - 1].parse().expect("Could not parse port");

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

    let mut line = String::new();
    if prompt {
        print_prompt();
    }

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
                syscall::close(handle);
                return Ok(());
            }

            let list = vec![(stdin, IO::Read), (handle, IO::Read)];
            if let Some((h, _)) = syscall::poll(&list) {
                if h == stdin {
                    match io::stdin().read_char() {
                        Some('\n') => {
                            line.push_str("\r\n");
                            syscall::write(handle, &line.as_bytes());
                            line.clear();
                            if prompt {
                                print_prompt();
                            }
                        }
                        Some(c) => {
                            line.push(c);
                        }
                        None => {}
                    }
                } else {
                    let mut data = vec![0; 2048];
                    if let Some(bytes) = syscall::read(handle, &mut data) {
                        data.resize(bytes, 0);
                        syscall::write(stdout, &data);
                    }
                }
            }
        }
    } else {
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} socket {}[<host>] <port>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-l{1}, {0}--listen{1}             Listen to a local port", csi_option, csi_reset);
    println!("  {0}-v{1}, {0}--verbose{1}            Increase verbosity", csi_option, csi_reset);
    println!("  {0}-p{1}, {0}--prompt{1}             Display prompt", csi_option, csi_reset);
    println!("  {0}-r{1}, {0}--read-only{1}          Read only connexion", csi_option, csi_reset);
    println!("  {0}-i{1}, {0}--interval <time>{1}    Wait <time> between packets", csi_option, csi_reset);
}
