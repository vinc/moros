use crate::api::base64::Base64;
use crate::api::fs;
use crate::api::ini;
use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::api::time;
use crate::sys::fs::{IO, OpenFlag};
use crate::sys::net::SocketStatus;
use crate::usr::host;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use bit_field::BitField;
use core::str::FromStr;
use smoltcp::wire::IpAddress;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() < 2 || args.len() == 2 && args[1] == "send" {
        error!("Missing command");
        return Err(ExitCode::UsageError);
    }
    if args[1] == "send" {
        smtp(&args[1..])
    } else {
        pop3(&args[1..])
    }
}

pub fn smtp(args: &[&str]) -> Result<(), ExitCode> {
    let config = if let Ok(buf) = fs::read_to_string("/ini/mail.ini") {
        if let Some(ini) = ini::parse(&buf) {
            ini
        } else {
            error!("Could not parse config file");
            return Err(ExitCode::Failure);
        }
    } else {
        error!("Could not read config file");
        return Err(ExitCode::Failure);
    };
    let user = config.get("user").unwrap();
    let pass = config.get("pass").unwrap();
    let smtp = config.get("smtp").unwrap();
    let (host, port) = smtp.split_once(":").unwrap();
    let addr = if host.ends_with(char::is_numeric) {
        match IpAddress::from_str(&host) {
            Ok(ip_addr) => ip_addr,
            Err(_) => {
                error!("Invalid address format");
                return Err(ExitCode::Failure);
            }
        }
    } else {
        match host::resolve(&host) {
            Ok(ip_addr) => ip_addr,
            Err(e) => {
                error!("Could not resolve host: {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };
    let port = port.parse().unwrap();

    let mut data = String::new();
    data.push_str(&format!("From: <{}>\r\n", user));
    data.push_str(&format!("To: <{}>\r\n", args[1]));
    let date = time::now_utc().format("%a, %d %b %Y %H:%M:%S GMT");
    data.push_str(&format!("Date: <{}>\r\n", date));
    data.push_str("Subject: ");
    print!("Subject: ");
    loop {
        let line = io::stdin().read_line().replace("\n", "\r\n");
        data.push_str(&line);
        if line == ".\r\n" {
            break;
        }
    }

    let socket_path = "/dev/net/tcp";
    let size = if let Some(info) = syscall::info(socket_path) {
        info.size() as usize
    } else {
        error!("Could not open '{}'", socket_path);
        return Err(ExitCode::Failure);
    };
    let flags = OpenFlag::Device as u8;
    if let Some(handle) = syscall::open(socket_path, flags) {
        if syscall::connect(handle, addr, port).is_err() {
            error!("Could not connect to {}:{}", addr, port);
            syscall::close(handle);
            return Err(ExitCode::Failure);
        }
        recv(handle, size);
        send(handle, &format!("HELO {}\n", host));
        recv(handle, size);
        send(handle, &format!("AUTH LOGIN\n"));
        recv(handle, size);
        let encoded_user = String::from_utf8(Base64::encode_with_pad(&user.as_bytes())).unwrap();
        send(handle, &format!("{}\n", encoded_user));
        recv(handle, size);
        let encoded_pass = String::from_utf8(Base64::encode_with_pad(&pass.as_bytes())).unwrap();
        send(handle, &format!("{}\n", encoded_pass));
        recv(handle, size);
        send(handle, &format!("MAIL FROM:<{}>\n", user));
        recv(handle, size);
        send(handle, &format!("RCPT TO:<{}>\n", args[1]));
        recv(handle, size);
        send(handle, &format!("DATA\n"));
        recv(handle, size);
        send(handle, &data);
        recv(handle, size);
        send(handle, &format!("QUIT\n"));
        loop {
            if recv(handle, size) == 0 {
                break;
            }
        }
        syscall::close(handle);
    }
    Ok(())
}

pub fn pop3(args: &[&str]) -> Result<(), ExitCode> {
    let config = if let Ok(buf) = fs::read_to_string("/ini/mail.ini") {
        if let Some(ini) = ini::parse(&buf) {
            ini
        } else {
            error!("Could not parse config file");
            return Err(ExitCode::Failure);
        }
    } else {
        error!("Could not read config file");
        return Err(ExitCode::Failure);
    };
    let cmd = args.join(" ");
    let user = config.get("user").unwrap();
    let pass = config.get("pass").unwrap();
    let pop3 = config.get("pop3").unwrap();
    let (host, port) = pop3.split_once(":").unwrap();
    let addr = if host.ends_with(char::is_numeric) {
        match IpAddress::from_str(&host) {
            Ok(ip_addr) => ip_addr,
            Err(_) => {
                error!("Invalid address format");
                return Err(ExitCode::Failure);
            }
        }
    } else {
        match host::resolve(&host) {
            Ok(ip_addr) => ip_addr,
            Err(e) => {
                error!("Could not resolve host: {:?}", e);
                return Err(ExitCode::Failure);
            }
        }
    };
    let port = port.parse().unwrap();

    let socket_path = "/dev/net/tcp";
    let size = if let Some(info) = syscall::info(socket_path) {
        info.size() as usize
    } else {
        error!("Could not open '{}'", socket_path);
        return Err(ExitCode::Failure);
    };
    let flags = OpenFlag::Device as u8;
    if let Some(handle) = syscall::open(socket_path, flags) {
        if syscall::connect(handle, addr, port).is_err() {
            error!("Could not connect to {}:{}", addr, port);
            syscall::close(handle);
            return Err(ExitCode::Failure);
        }
        recv(handle, size);
        send(handle, &format!("USER {}\n", user));
        recv(handle, size);
        send(handle, &format!("PASS {}\n", pass));
        recv(handle, size);
        send(handle, &format!("{}\n", cmd));
        send(handle, "QUIT\n");
        loop {
            if recv(handle, size) == 0 {
                break;
            }
        }
        syscall::close(handle);
    }
    Ok(())
}

fn poll(handle: usize, len: usize) {
    loop {
        let list = vec![(handle, IO::Read)];
        if syscall::poll(&list).is_some() {
            recv(handle, len);
        } else if is_closed(handle) {
            break;
        } else {
            syscall::sleep(0.01);
        }
    }
}

fn send(handle: usize, data: &str) {
    syscall::write(handle, data.as_bytes());
}

fn recv(handle: usize, len: usize) -> usize {
    let mut buf = vec![0; len];
    if let Some(bytes) = syscall::read(handle, &mut buf) {
        buf.resize(bytes, 0);
        syscall::write(1, &buf); // Write to stdout
        bytes
    } else {
        0
    }
}

fn is_closed(handle: usize) -> bool {
    let mut buf = vec![0; 1]; // 1 byte status read
    if let Some(bytes) = syscall::read(handle, &mut buf) {
        if bytes == 1 {
            return !buf[0].get_bit(SocketStatus::MayRecv as usize);
        }
    }
    true
}
