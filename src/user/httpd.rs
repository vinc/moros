use crate::{kernel, print, user};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use smoltcp::socket::TcpSocket;
use smoltcp::time::Instant;
use smoltcp::socket::TcpSocketBuffer;
use smoltcp::socket::SocketSet;
use time::OffsetDateTime;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    if let Some(ref mut iface) = *kernel::net::IFACE.lock() {
        match iface.ipv4_addr() {
            None => {
                print!("Error: Interface not ready\n");
                return user::shell::ExitCode::CommandError;
            }
            Some(ip_addr) if ip_addr.is_unspecified() => {
                print!("Error: Interface not ready\n");
                return user::shell::ExitCode::CommandError;
            }
            _ => {}
        }

        print!("HTTP Server listening on 0.0.0.0:80\n");

        let mut sockets = SocketSet::new(vec![]);
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = sockets.add(tcp_socket);

        let mut tcp_active = false;
        loop {
            let timestamp = Instant::from_millis((kernel::clock::realtime() * 1000.0) as i64);
            match iface.poll(&mut sockets, timestamp) {
                Ok(_) => {},
                Err(e) => {
                    print!("poll error: {}\n", e);
                }
            }

            {
                let mut socket = sockets.get::<TcpSocket>(tcp_handle);
                if !socket.is_open() {
                    socket.listen(80).unwrap();
                }

                if socket.is_active() && !tcp_active {
                    //print!("tcp:80 connected\n");
                } else if !socket.is_active() && tcp_active {
                    //print!("tcp:80 disconnected\n");
                }
                tcp_active = socket.is_active();

                let addr = socket.remote_endpoint().addr;
                if socket.may_recv() {
                    let res = socket.recv(|buffer| {
                        let mut res = String::new();
                        let req = String::from_utf8_lossy(buffer);
                        if req.len() > 0 {
                            for line in req.lines() {
                                if line.starts_with("GET") {
                                    let req_line: Vec<_> = line.split(" ").collect();
                                    //let method = req_line[0];
                                    let req_target = req_line[1];
                                    //let http_version = req_line[2];
                                    let date = strftime("%d/%b/%Y:%H:%M:%S %z");
                                    let code;
                                    let size;
                                    if let Some(file) = kernel::fs::File::open(req_target) {
                                        let body = file.read_to_string().replace("\n", "\r\n");
                                        code = 200;
                                        size = body.len();
                                        res.push_str("HTTP/1.0 200 OK\r\n");
                                        res.push_str("Server: MOROSHTTP/0.1\r\n");
                                        res.push_str(&format!("Date: {}\r\n", strftime("%a, %d %b %Y %H:%M:%S GMT")));
                                        res.push_str("Content-Type: text/plain; charset=utf-8\r\n");
                                        res.push_str(&format!("Content-Length: {}\r\n", body.len()));
                                        res.push_str("\r\n");
                                        res.push_str(&body);
                                    } else {
                                        code = 404;
                                        size = 0;
                                        res.push_str("HTTP/1.0 404 File not found\r\n");
                                        res.push_str("Server: MOROSHTTP/0.1\r\n");
                                        res.push_str(&format!("Date: {}\r\n", strftime("%a, %d %b %Y %H:%M:%S GMT")));
                                        res.push_str("Content-Type: text/plain; charset=utf-8\r\n");
                                        res.push_str("Content-Length: 0\r\n");
                                        res.push_str("\r\n");
                                    }
                                    print!("{} - - [{}] \"{}\" {} {}\n", addr, date, line, code, size);
                                }
                            }
                        }
                        (buffer.len(), res)
                    }).unwrap();
                    if socket.can_send() && res.len() > 0 {
                        socket.send_slice(res.as_bytes()).unwrap();
                        socket.close();
                    }
                } else if socket.may_send() {
                    //print!("tcp:80 close\n");
                    socket.close();
                }
            }

            if let Some(wait_duration) = iface.poll_delay(&sockets, timestamp) {
                let wait_duration: Duration = wait_duration.into();
                kernel::time::sleep(wait_duration.as_secs_f64());
            }
        }
    } else {
        print!("Error: Could not find network interface\n");
        user::shell::ExitCode::CommandError
    }
}

fn strftime(format: &str) -> String {
    let timestamp = kernel::clock::realtime();
    OffsetDateTime::from_unix_timestamp(timestamp as i64).format(format)
}
