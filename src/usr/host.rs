use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::rng;
use crate::api::syscall;
use crate::sys::fs::OpenFlag;
use crate::usr;
use alloc::vec;
use alloc::vec::Vec;
use bit_field::BitField;
use core::convert::TryInto;
use core::str;
use core::str::FromStr;
use smoltcp::wire::{IpAddress, Ipv4Address};

// See RFC 1035 for implementation details

#[repr(u16)]
enum QueryType {
    A = 1,
    // NS = 2,
    // MD = 3,
    // MF = 4,
    // CNAME = 5,
    // SOA = 6,
    // MX = 15,
    // TXT = 16,
}

#[repr(u16)]
enum QueryClass {
    IN = 1,
}

#[derive(Debug)]
#[repr(u16)]
pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,

    UnknownError,
    NetworkError,
}

struct Message {
    pub datagram: Vec<u8>,
}

const FLAG_RD: u16 = 0x0100; // Recursion desired

impl Message {
    pub fn from(datagram: &[u8]) -> Self {
        Self {
            datagram: Vec::from(datagram),
        }
    }

    pub fn query(qname: &str, qtype: QueryType, qclass: QueryClass) -> Self {
        let mut datagram = Vec::new();

        let id = rng::get_u16();
        for b in id.to_be_bytes().iter() {
            datagram.push(*b); // Transaction ID
        }
        for b in FLAG_RD.to_be_bytes().iter() {
            datagram.push(*b); // Flags
        }
        for b in (1 as u16).to_be_bytes().iter() {
            datagram.push(*b); // Questions
        }
        for _ in 0..6 {
            datagram.push(0); // Answer + Authority + Additional
        }
        for label in qname.split('.') {
            datagram.push(label.len() as u8); // QNAME label length
            for b in label.bytes() {
                datagram.push(b); // QNAME label bytes
            }
        }
        datagram.push(0); // Root null label
        for b in (qtype as u16).to_be_bytes().iter() {
            datagram.push(*b); // QTYPE
        }
        for b in (qclass as u16).to_be_bytes().iter() {
            datagram.push(*b); // QCLASS
        }

        Self { datagram }
    }

    pub fn id(&self) -> u16 {
        u16::from_be_bytes(self.datagram[0..2].try_into().unwrap())
    }

    pub fn header(&self) -> u16 {
        u16::from_be_bytes(self.datagram[2..4].try_into().unwrap())
    }

    pub fn is_response(&self) -> bool {
        self.header().get_bit(15)
    }

    pub fn code(&self) -> ResponseCode {
        match self.header().get_bits(11..15) {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::UnknownError,
        }
    }
}

fn dns_address() -> Option<IpAddress> {
    if let Some(servers) = usr::net::get_config("dns") {
        if let Some((server, _)) = servers.split_once(',') {
            if let Ok(addr) = IpAddress::from_str(server) {
                return Some(addr);
            }
        }
    }
    None
}

pub fn resolve(name: &str) -> Result<IpAddress, ResponseCode> {
    let addr = dns_address().unwrap_or(IpAddress::v4(8, 8, 8, 8));
    let port = 53;
    let query = Message::query(name, QueryType::A, QueryClass::IN);

    let socket_path = "/dev/net/udp";
    let buf_len = if let Some(info) = syscall::info(socket_path) {
        info.size() as usize
    } else {
        return Err(ResponseCode::NetworkError);
    };

    let flags = OpenFlag::Device as usize;
    if let Some(handle) = syscall::open(socket_path, flags) {
        if syscall::connect(handle, addr, port).is_err() {
            syscall::close(handle);
            return Err(ResponseCode::NetworkError);
        }
        if syscall::write(handle, &query.datagram).is_none() {
            syscall::close(handle);
            return Err(ResponseCode::NetworkError);
        }
        loop {
            let mut data = vec![0; buf_len];
            if let Some(bytes) = syscall::read(handle, &mut data) {
                if bytes < 28 {
                    break;
                }
                data.resize(bytes, 0);

                let message = Message::from(&data);
                if message.id() == query.id() && message.is_response() {
                    syscall::close(handle);
                    //usr::hex::print_hex(&message.datagram);
                    return match message.code() {
                        ResponseCode::NoError => {
                            // TODO: Parse the datagram instead of extracting
                            // the last 4 bytes
                            let n = message.datagram.len();
                            let data = &message.datagram[(n - 4)..];
                            let ipv4 = Ipv4Address::from_bytes(data);
                            if ipv4.is_unspecified() {
                                Err(ResponseCode::NameError) // FIXME
                            } else {
                                Ok(IpAddress::from(ipv4))
                            }
                        }
                        code => Err(code),
                    };
                }
            } else {
                break;
            }
        }
        syscall::close(handle);
    }
    Err(ResponseCode::NetworkError)
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    // TODO: Add `--server <address>` option
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    let domain = args[1];
    match resolve(domain) {
        Ok(addr) => {
            println!("{}", addr);
            Ok(())
        }
        Err(e) => {
            error!("Could not resolve host: {:?}", e);
            Err(ExitCode::Failure)
        }
    }
}

fn help() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} host {}<domain>{1}",
        csi_title, csi_reset, csi_option
    );
}
