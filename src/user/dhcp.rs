use alloc::vec;
use alloc::vec::Vec;
use alloc::string::ToString;
use core::time::Duration;
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

        let timestamp = Instant::from_millis((kernel::clock::clock_monotonic() * 1000.0) as i64);
        let mut dhcp = Dhcpv4Client::new(&mut sockets, dhcp_rx_buffer, dhcp_tx_buffer, timestamp);

        let mut prev_cidr = match iface.ip_addrs().first() {
            Some(IpCidr::Ipv4(ip_addr)) => ip_addr.clone().into(),
            _ => Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0),
        };

        print!("DHCP Discover transmitted\n");
        let time = kernel::clock::clock_monotonic();
        loop {
            if kernel::clock::clock_monotonic() - time > 60.0 {
                print!("Timeout reached\n");
                return user::shell::ExitCode::CommandError;
            }

            let timestamp = Instant::from_millis((kernel::clock::clock_monotonic() * 1000.0) as i64);
            iface.poll(&mut sockets, timestamp).map(|_| ()).unwrap_or_else(|e| print!("Error: {:?}\n", e));
            let config = dhcp.poll(&mut iface, &mut sockets, timestamp).unwrap_or_else(|e| {
                print!("DHCP: {:?}\n", e);
                None
            });
            let mut success = false;
            config.map(|config| {
                print!("DHCP Offer received\n");
                //print!("DHCP config: {:?}\n", config);
                match config.address {
                    Some(cidr) => if cidr != prev_cidr {
                        iface.update_ip_addrs(|addrs| {
                            addrs.iter_mut().nth(0).map(|addr| {
                                *addr = IpCidr::Ipv4(cidr);
                            });
                        });
                        prev_cidr = cidr;
                        print!("Leased: {}\n", cidr);
                    }
                    _ => {}
                }

                config.router.map(|router| {
                    iface.routes_mut().add_default_ipv4_route(router.into()).unwrap()
                });
                iface.routes_mut().update(|routes_map| {
                    routes_map.get(&IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)).map(|default_route| {
                        print!("Router: {}\n", default_route.via_router);
                    });
                });

                let dns_servers: Vec<_> = config.dns_servers.iter().filter_map(|s| *s).map(|s| s.to_string()).collect();
                if dns_servers.len() > 0 {
                    print!("DNS: {}\n", dns_servers.join(", "));
                }

                success = true;
            });

            if success {
                return user::shell::ExitCode::CommandSuccessful;
            }

            let mut timeout = dhcp.next_poll(timestamp);
            iface.poll_delay(&sockets, timestamp).map(|sockets_timeout| timeout = sockets_timeout);
            let wait_duration: Duration = timeout.into();
            kernel::time::sleep(wait_duration.as_secs_f64());
        }
    }

    user::shell::ExitCode::CommandError
}
