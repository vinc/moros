use crate::{kernel, print, user};
use crate::kernel::console::Style;
use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use smoltcp::socket::TcpSocket;
use smoltcp::time::Instant;
use smoltcp::socket::TcpSocketBuffer;
use smoltcp::socket::SocketSet;
use smoltcp::phy::Device;
use time::OffsetDateTime;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let port = 80;

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

        let csi_color = Style::color("Yellow");
        let csi_reset = Style::reset();
        print!("{}HTTP Server listening on 0.0.0.0:{}{}\n", csi_color, port, csi_reset);

        let mtu = iface.device().capabilities().max_transmission_unit;
        let mut sockets = SocketSet::new(vec![]);
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = sockets.add(tcp_socket);

        let mut send_queue: VecDeque<Vec<u8>> = VecDeque::new();
        let mut tcp_active = false;
        loop {
            if kernel::console::abort() {
                print!("\n");
                return user::shell::ExitCode::CommandSuccessful;
            }

            let timestamp = Instant::from_millis((kernel::clock::realtime() * 1000.0) as i64);
            //print!("{}\n", timestamp);
            match iface.poll(&mut sockets, timestamp) {
                Ok(_) => {},
                Err(_) => {
                    //print!("poll error: {}\n", e);
                }
            }

            {
                let mut socket = sockets.get::<TcpSocket>(tcp_handle);
                if !socket.is_open() {
                    socket.listen(port).unwrap();
                }

                let addr = socket.remote_endpoint().addr;
                //let port = socket.remote_endpoint().port;

                if socket.is_active() && !tcp_active {
                    //print!("tcp:80 {}:{} connected\n", addr, port);
                } else if !socket.is_active() && tcp_active {
                    //print!("tcp:80 {}:{} disconnected\n", addr, port);
                }
                tcp_active = socket.is_active();

                if socket.may_recv() {
                    //print!("tcp:80 {}:{} may recv\n", addr, port);
                    let res = socket.recv(|buffer| {
                        let mut res = String::new();
                        let req = String::from_utf8_lossy(buffer);
                        if req.len() > 0 {
                            let mut verb = "";
                            let mut path = "";
                            let mut header = true;
                            let mut contents = String::new();
                            for (i, line) in req.lines().enumerate() {
                                //print!("{}: '{}'\n", i, line);
                                if i == 0 {
                                    let fields: Vec<_> = line.split(" ").collect();
                                    if fields.len() >= 2 {
                                        verb = fields[0];
                                        path = fields[1];
                                    }
                                } else if line.is_empty() {
                                    header = false;
                                } else if !header {
                                    contents.push_str(&format!("{}\n", line));
                                }
                            }

                            let date = strftime("%d/%b/%Y:%H:%M:%S %z");
                            let code;
                            let mime;
                            let mut body;
                            match verb {
                                "GET" => {
                                    if path.len() > 1 && path.ends_with("/") {
                                        code = 301;
                                        res.push_str("HTTP/1.0 301 Moved Permanently\r\n");
                                        res.push_str(&format!("Location: {}\r\n", path.trim_end_matches('/')));
                                        body = format!("<h1>Moved Permanently</h1>\r\n");
                                        mime = "text/html";
                                    } else if let Some(file) = kernel::fs::File::open(path) {
                                        code = 200;
                                        res.push_str("HTTP/1.0 200 OK\r\n");
                                        body = file.read_to_string().replace("\n", "\r\n");
                                        mime = "text/plain";
                                    } else if let Some(dir) = kernel::fs::Dir::open(path) {
                                        code = 200;
                                        res.push_str("HTTP/1.0 200 OK\r\n");
                                        body = format!("<h1>Index of {}</h1>\r\n", path);
                                        let mut files: Vec<_> = dir.read().collect();
                                        files.sort_by_key(|f| f.name());
                                        for file in files {
                                            let sep = if path == "/" { "" } else { "/" };
                                            let path = format!("{}{}{}", path, sep, file.name());
                                            body.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", path, file.name()));
                                        }
                                        mime = "text/html";
                                    } else {
                                        code = 404;
                                        res.push_str("HTTP/1.0 404 Not Found\r\n");
                                        body = format!("<h1>Not Found</h1>\r\n");
                                        mime = "text/plain";
                                    }
                                },
                                "PUT" => {
                                    let maybe_file = match kernel::fs::File::open(path) {
                                        Some(file) => Some(file),
                                        None => kernel::fs::File::create(path),
                                    };
                                    match maybe_file {
                                        Some(mut file) => {
                                            if file.write(&contents.as_bytes()).is_ok() {
                                                code = 200;
                                                res.push_str("HTTP/1.0 200 OK\r\n");
                                            } else {
                                                code = 500;
                                                res.push_str("HTTP/1.0 500 Internal Server Error\r\n");
                                            }
                                        },
                                        None => {
                                            code = 403;
                                            res.push_str("HTTP/1.0 403 Forbidden\r\n");
                                        }
                                    }
                                    body = format!("");
                                    mime = "text/plain";
                                },
                                "DELETE" => {
                                    if kernel::fs::File::open(path).is_some() {
                                        if kernel::fs::File::delete(path).is_ok() {
                                            code = 200;
                                            res.push_str("HTTP/1.0 200 OK\r\n");
                                        } else {
                                            code = 500;
                                            res.push_str("HTTP/1.0 500 Internal Server Error\r\n");
                                        }
                                    } else {
                                        code = 404;
                                        res.push_str("HTTP/1.0 404 Not Found\r\n");
                                    }
                                    body = format!("");
                                    mime = "text/plain";
                                },
                                _ => {
                                    res.push_str("HTTP/1.0 400 Bad Request\r\n");
                                    code = 400;
                                    body = format!("<h1>Bad Request</h1>\r\n");
                                    mime = "text/plain";
                                },
                            }
                            let size = body.len();
                            res.push_str(&format!("Server: MOROS/{}\r\n", env!("CARGO_PKG_VERSION")));
                            res.push_str(&format!("Date: {}\r\n", strftime("%a, %d %b %Y %H:%M:%S GMT")));
                            res.push_str(&format!("Content-Type: {}; charset=utf-8\r\n", mime));
                            res.push_str(&format!("Content-Length: {}\r\n", size));
                            res.push_str("Connection: close\r\n");
                            res.push_str("\r\n");
                            res.push_str(&body);
                            print!("{} - - [{}] \"{} {}\" {} {}\n", addr, date, verb, path, code, size);
                        }
                        (buffer.len(), res)
                    }).unwrap();

                    //print!("tcp:80 recv {}\n", res.len());
                    for chunk in res.as_bytes().chunks(1024) {
                        send_queue.push_back(chunk.to_vec());
                        //print!("tcp:80 queue ({} items)\n", send_queue.len());
                    }

                    if socket.can_send() {
                        //print!("tcp:80 {}:{} can send\n", addr, port);
                        if let Some(chunk) = send_queue.pop_front() {
                            //print!("tcp:80 send ({} left in queue)\n", send_queue.len());
                            socket.send_slice(&chunk).unwrap();
                        }
                    }
                } else if socket.may_send() {
                    //print!("tcp:80 {}:{} may send\n", addr, port);
                    socket.close();
                    send_queue.clear();
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
