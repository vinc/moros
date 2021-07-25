use crate::{sys, usr};
use crate::api::syscall;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::socket::{RawPacketMetadata, RawSocketBuffer, SocketSet};
use smoltcp::time::Instant;
use smoltcp::wire::{IpCidr, Ipv4Address, Ipv4Cidr};

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        let mut iface = iface;
        let mut sockets = SocketSet::new(vec![]);
        let dhcp_rx_buffer = RawSocketBuffer::new([RawPacketMetadata::EMPTY; 1], vec![0; 900]);
        let dhcp_tx_buffer = RawSocketBuffer::new([RawPacketMetadata::EMPTY; 1], vec![0; 600]);

        let timestamp = Instant::from_millis((syscall::realtime() * 1000.0) as i64);
        let mut dhcp = Dhcpv4Client::new(&mut sockets, dhcp_rx_buffer, dhcp_tx_buffer, timestamp);

        let prev_cidr = match iface.ip_addrs().first() {
            Some(IpCidr::Ipv4(ip_addr)) => *ip_addr,
            _ => Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0),
        };

        println!("DHCP Discover transmitted");
        let timeout = 30.0;
        let started = syscall::realtime();
        loop {
            if syscall::realtime() - started > timeout {
                println!("Timeout reached");
                return usr::shell::ExitCode::CommandError;
            }
            if sys::console::end_of_text() {
                println!();
                return usr::shell::ExitCode::CommandError;
            }
            let timestamp = Instant::from_millis((syscall::realtime() * 1000.0) as i64);
            match iface.poll(&mut sockets, timestamp) {
                Err(smoltcp::Error::Unrecognized) => {}
                Err(e) => {
                    println!("Network Error: {}", e);
                }
                Ok(_) => {}
            }
            let res = dhcp.poll(&mut iface, &mut sockets, timestamp).unwrap_or_else(|e| {
                println!("DHCP Error: {:?}", e);
                None
            });
            if let Some(config) = res {
                println!("DHCP Offer received");
                if let Some(cidr) = config.address {
                    if cidr != prev_cidr {
                        iface.update_ip_addrs(|addrs| {
                            if let Some(addr) = addrs.iter_mut().next() {
                                *addr = IpCidr::Ipv4(cidr);
                            }
                        });
                        println!("Leased: {}", cidr);
                    }
                }

                config.router.map(|router| {
                    iface.routes_mut().add_default_ipv4_route(router).unwrap()
                });
                iface.routes_mut().update(|routes_map| {
                    let unspecified = IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0);
                    if let Some(default_route) = routes_map.get(&unspecified) {
                        println!("Router: {}", default_route.via_router);
                    }
                });

                let dns_servers: Vec<_> = config.dns_servers.iter().filter_map(|s| *s).map(|s| s.to_string()).collect();
                if !dns_servers.is_empty() {
                    println!("DNS: {}", dns_servers.join(", "));
                }

                return usr::shell::ExitCode::CommandSuccessful;
            }

            if let Some(wait_duration) = iface.poll_delay(&sockets, timestamp) {
                let wait_duration: Duration = wait_duration.into();
                syscall::sleep(wait_duration.as_secs_f64());
            }
        }
    }

    usr::shell::ExitCode::CommandError
}
