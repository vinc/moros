use crate::{sys, usr, debug};
use crate::api::syscall;
use alloc::string::ToString;
use alloc::vec::Vec;
use smoltcp::socket::{Dhcpv4Event, Dhcpv4Socket};
use smoltcp::time::Instant;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let mut verbose = false;
    let dhcp_config;

    for arg in args {
        match *arg {
            "-v" | "--verbose" => {
                verbose = true;
            }
            _ => {}
        }
    }

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        let dhcp_socket = Dhcpv4Socket::new();
        let dhcp_handle = iface.add_socket(dhcp_socket);
        if verbose {
            debug!("DHCP Discover transmitted");
        }
        let timeout = 30.0;
        let started = syscall::realtime();
        loop {
            if syscall::realtime() - started > timeout {
                error!("Timeout reached");
                iface.remove_socket(dhcp_handle);
                return usr::shell::ExitCode::CommandError;
            }
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                eprintln!();
                iface.remove_socket(dhcp_handle);
                return usr::shell::ExitCode::CommandError;
            }

            let timestamp = Instant::from_micros((syscall::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let event = iface.get_socket::<Dhcpv4Socket>(dhcp_handle).poll();
            match event {
                None => {}
                Some(Dhcpv4Event::Configured(config)) => {
                    dhcp_config = Some(config);
                    if verbose {
                        debug!("DHCP Offer received");
                    }
                    iface.remove_socket(dhcp_handle);
                    break;
                }
                Some(Dhcpv4Event::Deconfigured) => {
                }
            }

            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
    } else {
        error!("Network Error");
        return usr::shell::ExitCode::CommandError;
    }

    if let Some(config) = dhcp_config {
        //debug!("{:#?}", config);
        usr::net::main(&["net", "config", "ip", &config.address.to_string()]);
        usr::net::main(&["net", "config", "ip"]);

        if let Some(router) = config.router {
            usr::net::main(&["net", "config", "gw", &router.to_string()]);
        } else {
            usr::net::main(&["net", "config", "gw", "0.0.0.0"]);
        }
        usr::net::main(&["net", "config", "gw"]);

        let dns: Vec<_> = config.dns_servers.iter().filter_map(|s| *s).map(|s| s.to_string()).collect();
        if !dns.is_empty() {
            usr::net::main(&["net", "config", "dns", &dns.join(",")]);
        }
        usr::net::main(&["net", "config", "dns"]);

        return usr::shell::ExitCode::CommandSuccessful;
    }

    usr::shell::ExitCode::CommandError
}
