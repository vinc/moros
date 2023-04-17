use crate::{sys, usr, debug};
use crate::api::clock;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use smoltcp::socket::{Dhcpv4Event, Dhcpv4Socket};
use smoltcp::time::Instant;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut verbose = false;
    let dhcp_config;

    for arg in args {
        match *arg {
            "-h" | "--help" => return help(),
            "-v" | "--verbose" => verbose = true,
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
        let started = clock::realtime();
        loop {
            if clock::realtime() - started > timeout {
                error!("Timeout reached");
                iface.remove_socket(dhcp_handle);
                return Err(ExitCode::Failure);
            }
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                eprintln!();
                iface.remove_socket(dhcp_handle);
                return Err(ExitCode::Failure);
            }

            let timestamp = Instant::from_micros((clock::realtime() * 1000000.0) as i64);
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
        return Err(ExitCode::Failure);
    }

    if let Some(config) = dhcp_config {
        //debug!("{:#?}", config);
        usr::shell::exec(&format!("net config ip {}", config.address)).ok();
        usr::shell::exec("net config ip").ok();

        if let Some(router) = config.router {
            usr::shell::exec(&format!("net config gw {}", router)).ok();
        } else {
            usr::shell::exec("net config gw 0.0.0.0").ok();
        }
        usr::shell::exec("net config gw").ok();

        let dns: Vec<_> = config.dns_servers.iter().filter_map(|s| *s).map(|s| s.to_string()).collect();
        if !dns.is_empty() {
            usr::shell::exec(&format!("net config dns {}", dns.join(","))).ok();
        }
        usr::shell::exec("net config dns").ok();

        return Ok(());
    }

    Err(ExitCode::Failure)
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} dhcp {}<options>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-v{1}, {0}--verbose{1}              Increase verbosity", csi_option, csi_reset);
    Ok(())
}
