use crate::{print, kernel, user};
//use smoltcp::wire::Ipv4Address;
use smoltcp::time::Instant;
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref mut iface) = *kernel::rtl8139::IFACE.lock() {
        iface.device_mut().debug_mode = true;

        let mut server_rx_buffer = [0; 2048];
        let mut server_tx_buffer = [0; 2048];
        let server_socket = TcpSocket::new(
            TcpSocketBuffer::new(&mut server_rx_buffer[..]),
            TcpSocketBuffer::new(&mut server_tx_buffer[..])
        );

        /*
        match server_socket.connect((Ipv4Address::new(10, 0, 2, 2), 8000), 4242) {
            Ok(_) => {
                print!("Socket connected\n");
            },
            Err(e) => {
                print!("Socket error: {}\n", e);
            },
        }
        if server_socket.is_open() {
            print!("Socket is open\n");
            if server_socket.can_send() {
                print!("Socket can send\n");
                let buf = "Hello, World!".as_bytes();
                server_socket.send_slice(&buf);
            } else {
                print!("Socket cannot send\n");
            }
        } else {
            print!("Socket is not open\n");
        }
        */

        let mut sockets_storage = [None, None];
        let mut sockets = SocketSet::new(&mut sockets_storage[..]);
        let _server_handle = sockets.add(server_socket);

        loop {
            let now = kernel::clock::clock_monotonic();
            match iface.poll(&mut sockets, Instant::from_millis((now * 1000.0) as i64)) {
                Ok(true) => {
                    print!("------------------------------------------------------------------\n");
                    print!("Polling result: Ok(true)\n");
                },
                Ok(false) => {
                    //print!("------------------------------------------------------------------\n");
                    //print!("Polling Result: Ok(false)\n\n");
                },
                Err(e) => {
                    print!("------------------------------------------------------------------\n");
                    print!("polling result: err({})\n", e);
                }
            }
            kernel::time::sleep(1.0);
        }
    }
    user::shell::ExitCode::CommandSuccessful
}
