use crate::{sys, usr, debug};
use crate::api::console::Style;
use crate::api::syscall;
use crate::alloc::string::ToString;
use alloc::vec::Vec;
use smoltcp::socket::{Dhcpv4Event, Dhcpv4Socket};
use smoltcp::time::Instant;
use smoltcp::wire::{IpCidr, Ipv4Address, Ipv4Cidr};

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("LightCyan");
    let csi_reset = Style::reset();

    // TODO: Add `--verbose` option

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        let dhcp_socket = Dhcpv4Socket::new();
        let dhcp_handle = iface.add_socket(dhcp_socket);

        let previous_address = match iface.ip_addrs().first() {
            Some(IpCidr::Ipv4(ip_addr)) => *ip_addr,
            _ => Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0),
        };

        debug!("DHCP Discover transmitted");
        let timeout = 30.0;
        let started = syscall::realtime();
        loop {
            if syscall::realtime() - started > timeout {
                eprintln!("Timeout reached");
                iface.remove_socket(dhcp_handle);
                return usr::shell::ExitCode::CommandError;
            }
            if sys::console::end_of_text() {
                eprintln!();
                iface.remove_socket(dhcp_handle);
                return usr::shell::ExitCode::CommandError;
            }

            let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                eprintln!("Network Error: {}", e);
            }

            let event = iface.get_socket::<Dhcpv4Socket>(dhcp_handle).poll();
            match event {
                None => {}
                Some(Dhcpv4Event::Configured(config)) => {
                    debug!("DHCP Offer received");
                    if config.address != previous_address {
                        iface.update_ip_addrs(|addrs| {
                            if let Some(addr) = addrs.iter_mut().next() {
                                *addr = IpCidr::Ipv4(config.address);
                            }
                        });
                        println!("{}IP Address:{} {}", csi_color, csi_reset, config.address);
                    }

                    if let Some(router) = config.router {
                        println!("{}Gateway:{}    {}", csi_color, csi_reset, router);
                        iface.routes_mut().add_default_ipv4_route(router).unwrap();
                    } else {
                        println!("{}Gateway:{}    none", csi_color, csi_reset);
                        iface.routes_mut().remove_default_ipv4_route();
                    }

                    // TODO: save DNS servers in `/ini/dns` and use them with `host` command
                    let dns_servers: Vec<_> = config.dns_servers.iter().filter_map(|s| *s).map(|s| s.to_string()).collect();
                    if !dns_servers.is_empty() {
                        println!("{}DNS:{}        {}", csi_color, csi_reset, dns_servers.join(", "));
                    }

                    iface.remove_socket(dhcp_handle);
                    return usr::shell::ExitCode::CommandSuccessful;
                }
                Some(Dhcpv4Event::Deconfigured) => {
                }
            }

            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
    }

    usr::shell::ExitCode::CommandError
}
