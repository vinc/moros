use crate::api::clock;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::debug;
use crate::sys::console;
use crate::sys::net;
use crate::usr::shell;

use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::iface::SocketSet;
use smoltcp::socket::dhcpv4;
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

    if let Some((ref mut iface, ref mut device)) = *net::NET.lock() {
        let dhcp_socket = dhcpv4::Socket::new();
        let mut sockets = SocketSet::new(vec![]);
        let dhcp_handle = sockets.add(dhcp_socket);
        if verbose {
            debug!("DHCP Discover transmitted");
        }
        let timeout = 30.0;
        let started = clock::realtime();
        loop {
            if clock::realtime() - started > timeout {
                error!("Timeout reached");
                return Err(ExitCode::Failure);
            }
            if console::end_of_text() || console::end_of_transmission() {
                eprintln!();
                return Err(ExitCode::Failure);
            }

            let ms = (clock::realtime() * 1000000.0) as i64;
            let time = Instant::from_micros(ms);
            iface.poll(time, device, &mut sockets);
            let event = sockets.get_mut::<dhcpv4::Socket>(dhcp_handle).poll();

            match event {
                None => {}
                Some(dhcpv4::Event::Configured(config)) => {
                    dhcp_config = Some(
                        (config.address, config.router, config.dns_servers)
                    );
                    if verbose {
                        debug!("DHCP Offer received");
                    }
                    break;
                }
                Some(dhcpv4::Event::Deconfigured) => {}
            }

            if let Some(d) = iface.poll_delay(time, &sockets) {
                syscall::sleep((d.total_micros() as f64) / 1000000.0);
            }
        }
    } else {
        error!("Network Error");
        return Err(ExitCode::Failure);
    }

    if let Some((address, router, dns_servers)) = dhcp_config {
        shell::exec(&format!("net config ip {}", address)).ok();
        shell::exec("net config ip").ok();

        if let Some(router) = router {
            shell::exec(&format!("net config gw {}", router)).ok();
        } else {
            shell::exec("net config gw 0.0.0.0").ok();
        }
        shell::exec("net config gw").ok();

        let dns: Vec<_> = dns_servers.iter().map(|s| s.to_string()).collect();
        if !dns.is_empty() {
            shell::exec(&format!("net config dns {}", dns.join(","))).ok();
        }
        shell::exec("net config dns").ok();

        return Ok(());
    }

    Err(ExitCode::Failure)
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} dhcp {}<options>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-v{1}, {0}--verbose{1}              Increase verbosity",
        csi_option, csi_reset
    );
    Ok(())
}
