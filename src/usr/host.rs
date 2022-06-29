use crate::{sys, usr};
use crate::api::clock;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::random;
use crate::api::syscall;
use alloc::vec;
use alloc::vec::Vec;
use bit_field::BitField;
use core::convert::TryInto;
use core::str;
use core::str::FromStr;
use smoltcp::socket::{UdpPacketMetadata, UdpSocket, UdpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, IpEndpoint, Ipv4Address};

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

        let id = random::get_u16();
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

    /*
    pub fn is_query(&self) -> bool {
        !self.is_response()
    }
    */

    pub fn rcode(&self) -> ResponseCode {
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
    let dns_port = 53;
    let dns_address = dns_address().unwrap_or(IpAddress::v4(8, 8, 8, 8));
    let server = IpEndpoint::new(dns_address, dns_port);

    let local_port = 49152 + random::get_u16() % 16384;
    let client = IpEndpoint::new(IpAddress::Unspecified, local_port);

    let qname = name;
    let qtype = QueryType::A;
    let qclass = QueryClass::IN;
    let query = Message::query(qname, qtype, qclass);

    let udp_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 1024]);
    let udp_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 1024]);
    let udp_socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);

    #[derive(Debug)]
    enum State { Bind, Query, Response }
    let mut state = State::Bind;
    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        match iface.ipv4_addr() {
            None => {
                return Err(ResponseCode::NetworkError);
            }
            Some(ip_addr) if ip_addr.is_unspecified() => {
                return Err(ResponseCode::NetworkError);
            }
            _ => {}
        }

        let udp_handle = iface.add_socket(udp_socket);

        let timeout = 5.0;
        let started = clock::realtime();
        loop {
            if clock::realtime() - started > timeout {
                iface.remove_socket(udp_handle);
                return Err(ResponseCode::NetworkError);
            }
            let timestamp = Instant::from_micros((clock::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let socket = iface.get_socket::<UdpSocket>(udp_handle);

            state = match state {
                State::Bind if !socket.is_open() => {
                    socket.bind(client).unwrap();
                    State::Query
                }
                State::Query if socket.can_send() => {
                    socket.send_slice(&query.datagram, server).expect("cannot send");
                    State::Response
                }
                State::Response if socket.can_recv() => {
                    let (data, _) = socket.recv().expect("cannot receive");
                    let message = Message::from(data);
                    if message.id() == query.id() && message.is_response() {
                        iface.remove_socket(udp_handle);
                        return match message.rcode() {
                            ResponseCode::NoError => {
                                // TODO: Parse the datagram instead of
                                // extracting the last 4 bytes.
                                //let rdata = message.answer().rdata();
                                let n = message.datagram.len();
                                let rdata = &message.datagram[(n - 4)..];

                                Ok(IpAddress::from(Ipv4Address::from_bytes(rdata)))
                            }
                            rcode => {
                                Err(rcode)
                            }
                        }
                    }
                    state
                }
                _ => state
            };

            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);

            }
        }
    } else {
        Err(ResponseCode::NetworkError)
    }
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
            println!("{} has address {}", domain, addr);
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
    println!("{}Usage:{} host {}<domain>{1}", csi_title, csi_reset, csi_option);
}
