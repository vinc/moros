use core::str;
use alloc::vec;
use alloc::vec::Vec;
use crate::{print, kernel, user};
use smoltcp::socket::{SocketSet, UdpSocket, UdpSocketBuffer, UdpPacketMetadata};
use smoltcp::time::Instant;
use smoltcp::wire::{IpEndpoint, IpAddress};

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

struct Query {
    pub datagram: Vec<u8>
}

const FLAG_RD: u16 = 0x0100; // Recursion desired

impl Query {
    pub fn new(qname: &str, qtype: QueryType, qclass: QueryClass) -> Self {
        let mut datagram = Vec::new();

        let id = kernel::random::rand16().expect("random id");
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

        Self {
            datagram
        }
    }
}

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        print!("Usage: host <name>\n");
        return user::shell::ExitCode::CommandError;
    }

    //let is_verbose = true;

    if let Some(ref mut iface) = *kernel::rtl8139::IFACE.lock() {
        let dns_address = IpAddress::v4(192, 168, 1, 3);
        let dns_port = 53;
        let server = IpEndpoint::new(dns_address, dns_port);

        let local_port = 49152 + kernel::random::rand16().expect("random port") % 16384;
        let client = IpEndpoint::new(IpAddress::Unspecified, local_port);

        let qname = args[1];
        let qtype = QueryType::A;
        let qclass = QueryClass::IN;
        let query = Query::new(qname, qtype, qclass);

        let udp_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 512]);
        let udp_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 512]);
        let udp_socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);

        let mut sockets = SocketSet::new(vec![]);
        let udp_handle = sockets.add(udp_socket);

        enum State { Bind, Query, Reply };
        let mut state = State::Bind;

        loop {
            let timestamp = Instant::from_millis((kernel::clock::clock_monotonic() * 1000.0) as i64);
            match iface.poll(&mut sockets, timestamp) {
                Ok(_) => {},
                Err(e) => {
                    print!("poll error: {}\n", e);
                }
            }

            {
                let mut socket = sockets.get::<UdpSocket>(udp_handle);

                state = match state {
                    State::Bind if !socket.is_open() => {
                        socket.bind(client).unwrap();
                        State::Query
                    }
                    State::Query if socket.can_send() => {
                        socket.send_slice(&query.datagram, server).expect("cannot send");
                        State::Reply
                    }
                    State::Reply if socket.can_recv() => {
                        let (data, _) = socket.recv().expect("cannot receive");
                        user::hex::print_hex(data);
                        break;
                    }
                    _ => state
                }
            }

            if let Some(wait_duration) = iface.poll_delay(&sockets, timestamp) {
                kernel::time::sleep(wait_duration.millis() as f64 / 1000.0);
            }
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        user::shell::ExitCode::CommandError
    }
}
