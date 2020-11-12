use crate::{kernel, print, user};
//use smoltcp::wire::Ipv4Address;
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 1 {
        print!("Usage: net <command>\n");
        return user::shell::ExitCode::CommandError;
    }

    if let Some(ref mut iface) = *kernel::net::IFACE.lock() {
        match args[1] {
            "config" => {
                if args.len() < 4 {
                    print!("Usage: net config <key> <value>\n");
                    return user::shell::ExitCode::CommandError;
                }
                match args[2] {
                    "debug" => {
                        iface.device_mut().debug_mode = match args[3] {
                            "1" | "on" | "enable" => true,
                            "0" | "off" | "disable" => false,
                            _ => {
                                print!("Invalid config value\n");
                                return user::shell::ExitCode::CommandError;
                            }
                        }
                    }
                    _ => {
                        print!("Invalid config key\n");
                        return user::shell::ExitCode::CommandError;
                    }
                }
            }
            "monitor" => {
                iface.device_mut().debug_mode = true;

                let mut server_rx_buffer = [0; 2048];
                let mut server_tx_buffer = [0; 2048];
                let server_socket = TcpSocket::new(
                    TcpSocketBuffer::new(&mut server_rx_buffer[..]),
                    TcpSocketBuffer::new(&mut server_tx_buffer[..]),
                );

                let mut sockets_storage = [None, None];
                let mut sockets = SocketSet::new(&mut sockets_storage[..]);
                let _server_handle = sockets.add(server_socket);

                loop {
                    if kernel::console::abort() {
                        print!("\n");
                        return user::shell::ExitCode::CommandSuccessful;
                    }

                    let now = kernel::clock::uptime();
                    match iface.poll(&mut sockets, Instant::from_millis((now * 1000.0) as i64)) {
                        Ok(true) => {
                            //print!("{}\n", "-".repeat(66));
                            //print!("Polling result: Ok(true)\n");
                        },
                        Ok(false) => {
                            //print!("{}\n", "-".repeat(66));
                            //print!("Polling Result: Ok(false)\n\n");
                        },
                        Err(_) => {
                            //print!("{}\n", "-".repeat(66));
                            //print!("polling result: err({})\n", e);
                        }
                    }
                    kernel::time::sleep(0.1);
                }
            }
            _ => {
                print!("Invalid command\n");
                return user::shell::ExitCode::CommandError;
            }
        }
    }
    user::shell::ExitCode::CommandSuccessful
}
