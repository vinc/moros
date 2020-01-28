use alloc::vec;
use crate::{print, kernel, user};
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::socket::{SocketSet, RawSocketBuffer, RawPacketMetadata};
use smoltcp::time::Instant;
use smoltcp::wire::{Ipv4Address, IpCidr, Ipv4Cidr};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref mut iface) = *kernel::rtl8139::IFACE.lock() {
        let mut iface = iface;
        let mut sockets = SocketSet::new(vec![]);
        let dhcp_rx_buffer = RawSocketBuffer::new(
            [RawPacketMetadata::EMPTY; 1],
            vec![0; 900]
        );
        let dhcp_tx_buffer = RawSocketBuffer::new(
            [RawPacketMetadata::EMPTY; 1],
            vec![0; 600]
        );
        let now = Instant::from_millis((kernel::clock::clock_monotonic() * 1000.0) as i64);
        let mut dhcp = Dhcpv4Client::new(&mut sockets, dhcp_rx_buffer, dhcp_tx_buffer, now);
        let mut prev_cidr = Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0);
        loop {
            let now = Instant::from_millis((kernel::clock::clock_monotonic() * 1000.0) as i64);
            iface.poll(&mut sockets, now).map(|_| ()).unwrap_or_else(|e| print!("Poll: {:?}\n", e));
            let config = dhcp.poll(&mut iface, &mut sockets, now).unwrap_or_else(|e| {
                print!("DHCP: {:?}\n", e);
                None
            });
            let mut success = false;
            config.map(|config| {
                print!("DHCP config: {:?}\n", config);
                match config.address {
                    Some(cidr) => if cidr != prev_cidr {
                        iface.update_ip_addrs(|addrs| {
                            addrs.iter_mut().nth(0).map(|addr| {
                                *addr = IpCidr::Ipv4(cidr);
                            });
                        });
                        prev_cidr = cidr;
                        print!("Assigned a new IPv4 address: {}\n", cidr);
                    }
                    _ => {}
                }

                config.router.map(|router| {
                    iface.routes_mut().add_default_ipv4_route(router.into()).unwrap()
                });
                iface.routes_mut().update(|routes_map| {
                    routes_map.get(&IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)).map(|default_route| {
                        print!("Default gateway: {}\n", default_route.via_router);
                    });
                });

                if config.dns_servers.iter().any(|s| s.is_some()) {
                    print!("DNS servers:\n");
                    for dns_server in config.dns_servers.iter().filter_map(|s| *s) {
                        print!("- {}\n", dns_server);
                    }
                }

                success = true;
            });

            if success {
                return user::shell::ExitCode::CommandSuccessful;
            }
        }
    }

    user::shell::ExitCode::CommandError
}
